//! Best-effort installation diagnostics. No credentials are persisted here and
//! no reporting result participates in authorization or installation decisions.
use crate::*;
use fs2::FileExt;

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Cache {
    company_id: String,
    report: InstallationReport,
}

fn uuid() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 16];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    bytes[6] = (bytes[6] & 15) | 64;
    bytes[8] = (bytes[8] & 63) | 128;
    let hex = hex::encode(bytes);
    format!("{}-{}-{}-{}-{}", &hex[..8], &hex[8..12], &hex[12..16], &hex[16..20], &hex[20..])
}

fn locked(directory: &Path) -> Result<std::fs::File> {
    // The existing installed state directory supplies the Windows ACL. Never
    // create an alternate directory with weaker permissions for diagnostics.
    ensure!(directory.join("managed-state.json").is_file(), "Installed company state required");
    let mut options = std::fs::OpenOptions::new();
    options.create(true).read(true).write(true);
    #[cfg(unix)] {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let file = options.open(directory.join("installation-report.lock"))?;
    file.try_lock_exclusive().context("Another installation reporter is busy")?;
    Ok(file)
}

fn read(directory: &Path) -> Result<Option<Cache>> {
    match std::fs::read(directory.join("installation-report.json")) {
        Ok(bytes) => {
            let cache: Cache = serde_json::from_slice(&bytes)?;
            cache.report.validate()?;
            Ok(Some(cache))
        }
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn save(directory: &Path, cache: &Cache) -> Result<()> {
    use std::io::Write;
    let temporary = directory.join(format!("installation-report-{}.tmp", random_token()));
    let mut options = std::fs::OpenOptions::new();
    options.create_new(true).write(true);
    #[cfg(unix)] {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> Result<()> {
        let mut file = options.open(&temporary)?;
        file.write_all(&serde_json::to_vec(cache)?)?;
        file.sync_all()?;
        drop(file);
        crate::replace_state(&temporary, &directory.join("installation-report.json"))
    })();
    if result.is_err() && temporary.exists() { std::fs::remove_file(temporary)?; }
    result
}

fn prepare(directory: &Path, phase: Option<UpdatePhase>, target: Option<u64>, downloaded: u64, total: Option<u64>) -> Result<(AgentState, InstallationReport)> {
    let _lock = locked(directory)?;
    ensure!(!directory.join("pending-uninstall").exists(), "Uninstall in progress");
    let state = AgentState::load(directory)?;
    let old = read(directory)?;
    if let Some(old) = &old {
        ensure!(old.company_id == state.bootstrap.company_id && old.report.identity.edition == state.bootstrap.edition, "Installation report ownership changed");
    }
    let installation_id = match state.bootstrap.edition {
        Edition::Customer => state.device_id.clone().context("Device enrollment required")?,
        Edition::Technician => old.as_ref().map(|cache| cache.report.identity.installation_id.clone()).unwrap_or_else(uuid),
    };
    if let Some(old) = &old { ensure!(old.report.identity.installation_id == installation_id, "Installation identity changed"); }
    let installed_version = if state.last_release_sequence > 0 {
        let envelope = serde_json::from_slice(&std::fs::read(directory.join("installed-release.json"))?)?;
        Some(update::validate_previous_release(&state, &envelope, state.last_release_sequence)?.version)
    } else { None };
    let (phase, target) = if let Some(phase) = phase { (phase, target) }
    else if let Some(sequence) = update::pending_report_target(&state, directory)? { (UpdatePhase::Recovering, Some(sequence)) }
    else { (if state.last_release_sequence > 0 { UpdatePhase::Current } else { UpdatePhase::Checking }, None) };
    let report = InstallationReport {
        schema: 1,
        identity: InstallationIdentity { edition: state.bootstrap.edition.clone(), installation_id },
        revision: old.map(|cache| cache.report.revision).unwrap_or(0).checked_add(1).context("Report revision exhausted")?,
        installed_sequence: state.last_release_sequence,
        installed_version,
        phase,
        target_sequence: target, downloaded_bytes: downloaded, total_bytes: total,
    };
    report.validate()?;
    save(directory, &Cache { company_id: state.bootstrap.company_id.clone(), report: report.clone() })?;
    Ok((state, report))
}

fn rebase(directory: &Path, company: &str, report: &InstallationReport, remote_revision: u64) -> Result<Option<InstallationReport>> {
    let _lock = locked(directory)?;
    let Some(mut cache) = read(directory)? else { return Ok(None); };
    // An old asynchronous response cannot replace a newer download/install phase.
    if cache.company_id != company || cache.report != *report { return Ok(None); }
    cache.report.revision = remote_revision.max(report.revision).checked_add(1).context("Report revision exhausted")?;
    cache.report.validate()?;
    save(directory, &cache)?;
    Ok(Some(cache.report))
}

/// The caller may log a generic failure, but must continue its normal work.
pub async fn report(directory: &Path, technician_token: Option<&str>, phase: Option<UpdatePhase>, target: Option<u64>, downloaded: u64, total: Option<u64>) -> Result<()> {
    let folder = directory.to_owned();
    let (state, report) = tokio::task::spawn_blocking(move || prepare(&folder, phase, target, downloaded, total)).await.context("Report preparation task failed")??;
    let token = match state.bootstrap.edition {
        Edition::Customer => state.device_token.as_deref().context("Device enrollment required")?,
        Edition::Technician => technician_token.context("Current technician login required")?,
    };
    let client = crate::http_client(3, false)?;
    let endpoint = state.endpoint("installations/report")?;
    transmit(directory, &state.bootstrap.company_id, report, token, client, &endpoint).await
}

async fn transmit(directory: &Path, company: &str, mut report: InstallationReport, token: &str, client: reqwest::Client, endpoint: &str) -> Result<()> {
    let response = client.post(endpoint).bearer_auth(token).json(&report).send().await?;
    if response.status() != reqwest::StatusCode::CONFLICT { response.error_for_status()?; return Ok(()); }
    let remote: Value = client.get(endpoint).query(&report.identity).bearer_auth(token).send().await?.error_for_status()?.json().await?;
    let revision = remote["revision"].as_u64().context("Missing remote report revision")?;
    let folder = directory.to_owned(); let company = company.to_owned(); let previous = report.clone();
    let Some(rebased) = tokio::task::spawn_blocking(move || rebase(&folder, &company, &previous, revision)).await.context("Report revision recovery task failed")?? else { return Ok(()); };
    report = rebased;
    // One bounded retry; a concurrent reporter wins naturally on the server.
    client.post(endpoint).bearer_auth(token).json(&report).send().await?.error_for_status()?;
    Ok(())
}

pub async fn heartbeat(directory: &Path, technician_token: Option<&str>) {
    if report(directory, technician_token, None, None, 0, None).await.is_err() {
        eprintln!("Installation status reporting deferred");
    }
}

pub async fn progress(directory: &Path, technician_token: Option<&str>, phase: UpdatePhase, target: Option<u64>, downloaded: u64, total: Option<u64>) {
    if report(directory, technician_token, Some(phase), target, downloaded, total).await.is_err() {
        eprintln!("Installation status reporting deferred");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn identity_and_revision_survive_restart_without_credentials_and_stale_callbacks_lose() {
        let directory = std::env::temp_dir().join(format!("swan-report-{}", random_token()));
        std::fs::create_dir(&directory).unwrap();
        std::fs::write(directory.join("managed-state.json"), b"test lock prerequisite").unwrap();
        let report = InstallationReport { schema: 1, identity: InstallationIdentity { edition: Edition::Technician, installation_id: uuid() }, revision: 1, installed_sequence: 0, installed_version: None, phase: UpdatePhase::Checking, target_sequence: None, downloaded_bytes: 0, total_bytes: None };
        report.validate().unwrap();
        { let _lock = locked(&directory).unwrap(); save(&directory, &Cache { company_id: "company".into(), report: report.clone() }).unwrap(); }
        assert_eq!(read(&directory).unwrap().unwrap().report, report);
        let next = rebase(&directory, "company", &report, 81).unwrap().unwrap();
        assert_eq!(next.revision, 82);
        assert_eq!(next.identity, report.identity);
        assert!(rebase(&directory, "company", &report, 100).unwrap().is_none());
        assert!(rebase(&directory, "other-company", &next, 100).unwrap().is_none());
        let bytes = std::fs::read_to_string(directory.join("installation-report.json")).unwrap();
        for secret in ["token", "password", "totp", "private_key"] { assert!(!bytes.contains(secret)); }
        #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; assert_eq!(std::fs::metadata(directory.join("installation-report.json")).unwrap().permissions().mode() & 0o777, 0o600); }
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn preparation_uses_enrolled_identity_and_never_guesses_installed_version() {
        let directory = std::env::temp_dir().join(format!("swan-report-state-{}", random_token()));
        let mut state = crate::tests::fixture();
        state.device_id = Some(uuid());
        state.save(&directory).unwrap();
        let (_, first) = prepare(&directory, None, None, 0, None).unwrap();
        assert_eq!(Some(&first.identity.installation_id), state.device_id.as_ref());
        assert_eq!(first.installed_version, None);
        assert_eq!(first.phase, UpdatePhase::Checking);
        let (_, second) = prepare(&directory, Some(UpdatePhase::Downloading), Some(10), 50, Some(100)).unwrap();
        assert_eq!(second.revision, first.revision + 1);
        assert_eq!(second.identity, first.identity);
        assert_eq!(second.downloaded_bytes, 50);
        state.last_release_sequence = 9; state.save(&directory).unwrap();
        assert!(prepare(&directory, None, None, 0, None).is_err(), "Missing signed installation must not be guessed from agent version");
        assert_eq!(read(&directory).unwrap().unwrap().report, second);
        std::fs::write(directory.join("installed-release.json"), b"{}").unwrap();
        assert!(prepare(&directory, None, None, 0, None).is_err());
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn technician_restarts_and_concurrent_reporters_keep_one_identity_and_unique_revisions() {
        let directory = std::env::temp_dir().join(format!("swan-report-tech-{}", random_token()));
        let mut state = crate::tests::fixture();
        state.bootstrap.edition = Edition::Technician;
        state.device_id = None; state.device_token = None;
        state.save(&directory).unwrap();
        let (_, first) = prepare(&directory, None, None, 0, None).unwrap();
        {
            let _busy = locked(&directory).unwrap();
            assert!(prepare(&directory, None, None, 0, None).is_err(), "Busy reporting must defer instead of waiting on another process");
        }
        let handles: Vec<_> = (0..8).map(|_| {
            let folder = directory.clone();
            std::thread::spawn(move || {
                let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
                loop {
                    if let Ok((_, report)) = prepare(&folder, None, None, 0, None) { break report; }
                    assert!(std::time::Instant::now() < deadline, "Concurrent reporter never completed");
                    std::thread::sleep(std::time::Duration::from_millis(1));
                }
            })
        }).collect();
        let mut revisions = Vec::new();
        for handle in handles {
            let report = handle.join().unwrap();
            assert_eq!(report.identity, first.identity);
            revisions.push(report.revision);
        }
        revisions.sort_unstable(); assert_eq!(revisions, (2..=9).collect::<Vec<_>>());
        std::fs::write(directory.join("installation-report.json"), b"corrupt cache").unwrap();
        assert!(prepare(&directory, None, None, 0, None).is_err(), "Corrupt state cannot reset identity and counters");
        std::fs::remove_dir_all(directory).unwrap();
    }

    #[tokio::test]
    async fn http_conflict_recovers_once_and_a_newer_local_phase_cancels_old_retry() {
        use axum::{extract::State, http::{HeaderMap, StatusCode}, routing::post, Json, Router};
        use std::sync::{Arc, Mutex};
        #[derive(Clone)]
        struct Fixture { reports: Arc<Mutex<Vec<InstallationReport>>>, directory: PathBuf, newer: bool }
        async fn receive(State(fixture): State<Fixture>, headers: HeaderMap, Json(report): Json<InstallationReport>) -> StatusCode {
            assert_eq!(headers["authorization"], "Bearer test-session-memory-only");
            let mut reports = fixture.reports.lock().unwrap(); reports.push(report);
            if reports.len() == 1 { StatusCode::CONFLICT } else { StatusCode::OK }
        }
        async fn previous(State(fixture): State<Fixture>, headers: HeaderMap) -> Json<Value> {
            assert_eq!(headers["authorization"], "Bearer test-session-memory-only");
            if fixture.newer {
                let folder = fixture.directory.clone();
                tokio::task::spawn_blocking(move || {
                    let _lock = locked(&folder).unwrap();
                    let mut cache = read(&folder).unwrap().unwrap();
                    cache.report.revision += 1; cache.report.phase = UpdatePhase::Preparing;
                    cache.report.target_sequence = Some(10);
                    save(&folder, &cache).unwrap();
                }).await.unwrap();
            }
            Json(json!({"revision": 40}))
        }
        for newer in [false, true] {
            let directory = std::env::temp_dir().join(format!("swan-report-http-{}", random_token()));
            let mut state = crate::tests::fixture(); state.device_id = Some(uuid()); state.save(&directory).unwrap();
            let (_, report) = prepare(&directory, None, None, 0, None).unwrap();
            let fixture = Fixture { reports: Arc::new(Mutex::new(Vec::new())), directory: directory.clone(), newer };
            let app = Router::new().route("/report", post(receive).get(previous)).with_state(fixture.clone());
            let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
            let endpoint = format!("http://{}/report", listener.local_addr().unwrap());
            let server = tokio::spawn(async move { axum::serve(listener, app).await.unwrap(); });
            // Only this private transport fixture accepts HTTP. Production builds
            // derive a validated HTTPS company endpoint and forbid redirects.
            transmit(&directory, &state.bootstrap.company_id, report, "test-session-memory-only", reqwest::Client::new(), &endpoint).await.unwrap();
            let reports = fixture.reports.lock().unwrap().clone();
            assert_eq!(reports.len(), if newer { 1 } else { 2 });
            let saved = read(&directory).unwrap().unwrap().report;
            if newer { assert_eq!(saved.phase, UpdatePhase::Preparing); assert_eq!(saved.revision, 2); }
            else { assert_eq!(reports[1].revision, 41); assert_eq!(saved, reports[1]); }
            server.abort(); let _ = server.await;
            std::fs::remove_dir_all(directory).unwrap();
        }
    }
}
