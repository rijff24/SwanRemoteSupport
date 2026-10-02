//! Company-approved endpoint updates. No package code executes before metadata,
//! bytes, publisher and session exclusion have been checked.
use crate::*;
use anyhow::{ensure,Context,Result};
use serde::{Deserialize,Serialize};
use std::path::Path;

#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt { release:SignedEnvelope, previous_sequence:u64, phase:String }

fn require_existing_application(target:&Path)->Result<()> {
    ensure!(target.is_file(),"Application was removed; automatic updates cannot reinstall it. Run explicit company setup or repair.");
    Ok(())
}

#[cfg(windows)]
fn installed_target(directory:&Path,edition:&Edition)->Result<std::path::PathBuf> {
    if *edition==Edition::Technician {Ok(directory.join("SwanRemoteSupport-Technician.exe"))}else{
        Ok(std::path::PathBuf::from(std::env::var_os("ProgramFiles").context("ProgramFiles unavailable")?).join("Swan Remote Support/Swan Remote Support.exe"))
    }
}

pub fn validate_release(state:&AgentState,envelope:&SignedEnvelope)->Result<Release> {
    let profile=state.company_profile()?;
    let release:Release=envelope.verify(&public_key(&state.bootstrap.release_public_key)?)?;
    release.validate(&state.bootstrap.edition,state.last_release_sequence,now())?;
    ensure!(release.channel==profile.update_channel,"Wrong update channel");
    Ok(release)
}

pub fn validate_repair_release(state:&AgentState,envelope:&SignedEnvelope,installed:&SignedEnvelope)->Result<Release> {
    state.company_profile()?;
    ensure!(state.last_release_sequence>0,"Repair requires a previously recorded installation");
    let key=public_key(&state.bootstrap.release_public_key)?;
    let release:Release=envelope.verify(&key)?;
    let previous:Release=installed.verify(&key)?;
    ensure!(envelope.payload==installed.payload && envelope.signature==installed.signature && release.sequence==previous.sequence && release.sequence==state.last_release_sequence,"Repair must use the exact currently pinned release");
    release.validate(&state.bootstrap.edition,state.last_release_sequence-1,now())?;
    Ok(release)
}

#[cfg(any(windows,test))]
pub(crate) fn validate_recovery_release(state:&AgentState,envelope:&SignedEnvelope,previous_sequence:u64,phase:&str)->Result<Release> {
    ensure!(phase=="installing","Unknown update recovery phase");
    let release:Release=envelope.verify(&public_key(&state.bootstrap.release_public_key)?)?;
    release.validate(&state.bootstrap.edition,previous_sequence,now().min(release.expires_at.saturating_sub(1)))?;
    ensure!(state.last_release_sequence==previous_sequence || state.last_release_sequence==release.sequence,"Recovery sequence conflict");
    Ok(release)
}

#[cfg(windows)]
fn installation_release(state:&AgentState,directory:&Path,envelope:&SignedEnvelope,repair:bool)->Result<Release> {
    ensure!(!directory.join("pending-update.json").exists(),"Resolve pending update recovery before setup or repair");
    if repair {
        let installed:SignedEnvelope=serde_json::from_slice(&std::fs::read(directory.join("installed-release.json"))?)?;
        validate_repair_release(state,envelope,&installed)
    }else{validate_release(state,envelope)}
}

async fn download(url:&str,hash:&str,path:&Path)->Result<()> {
    https_url(url)?;
    // Artifact endpoints may redirect to HTTPS object storage. No credentials
    // are attached, and reqwest rejects any redirect to plain HTTP.
    let client=http_client(600,true)?;
    let mut response=client.get(url).send().await?.error_for_status()?;
    ensure!(response.content_length().unwrap_or(0)<=512*1024*1024,"Package too large");
    let mut bytes=Vec::new();
    while let Some(chunk)=response.chunk().await? {ensure!(bytes.len()+chunk.len()<=512*1024*1024,"Package too large");bytes.extend_from_slice(&chunk);}
    ensure!(digest(&bytes).eq_ignore_ascii_case(hash),"Package hash mismatch");
    std::fs::write(path,bytes)?;Ok(())
}

#[cfg(windows)]
fn verify_publisher(path:&Path,release:&Release)->Result<()> {
    let script=path.parent().context("Missing update directory")?.join("Verify-Package.ps1");
    std::fs::write(&script,include_str!("../../../deployment/windows/Verify-Package.ps1"))?;
    let status=std::process::Command::new("powershell.exe").args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"])
        .arg(script).arg("-Path").arg(path).arg("-Publisher").arg(&release.publisher).arg("-CertificateSha256").arg(&release.publisher_certificate_sha256).arg("-Sha256").arg(&release.sha256).status()?;
    ensure!(status.success(),"Package publisher verification failed");Ok(())
}

#[cfg(windows)]
fn msi_install_command(package:&Path)->Result<std::process::Command> {
    let script=package.parent().context("Missing update directory")?.join("Get-MsiInstallMode.ps1");
    std::fs::write(&script,include_str!("../../../deployment/windows/Get-MsiInstallMode.ps1"))?;
    let output=std::process::Command::new("powershell.exe")
        .args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"])
        .arg(script).arg("-Package").arg(package).output()?;
    ensure!(output.status.success(),"Unable to determine MSI installation or repair mode");
    let mode=std::str::from_utf8(&output.stdout)?.trim();
    ensure!(matches!(mode,"/i"|"/fvamus"),"Unexpected MSI installation mode");
    let mut command=std::process::Command::new("msiexec.exe");
    command.arg(mode).arg(package).args(["/qn","/norestart"]);
    Ok(command)
}

#[cfg(windows)]
pub fn verify_package(state:&AgentState,directory:&Path,envelope:&SignedEnvelope,path:&Path)->Result<()> {
    let release=installation_release(state,directory,envelope,false)?;
    verify_compatibility(path.parent().context("Missing package directory")?,&release)?;
    ensure!(digest(std::fs::read(path)?).eq_ignore_ascii_case(&release.sha256),"Installer hash differs from signed metadata");
    verify_publisher(path,&release)
}

#[cfg(windows)]
pub fn verify_repair_package(state:&AgentState,directory:&Path,envelope:&SignedEnvelope,path:&Path)->Result<()> {
    let release=installation_release(state,directory,envelope,true)?;
    verify_compatibility(path.parent().context("Missing package directory")?,&release)?;
    ensure!(digest(std::fs::read(path)?).eq_ignore_ascii_case(&release.sha256),"Repair installer hash differs from signed metadata");
    verify_publisher(path,&release)
}

#[cfg(windows)]
fn verify_compatibility(directory:&Path,release:&Release)->Result<()> {
    let script=directory.join("Get-WindowsCompatibility.ps1");
    std::fs::write(&script,include_str!("../../../deployment/windows/Get-WindowsCompatibility.ps1"))?;
    let output=std::process::Command::new("powershell.exe").args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(script).output()?;
    ensure!(output.status.success(),"Unsupported Windows version or installation type");
    let platform=String::from_utf8(output.stdout)?;
    ensure!(release.windows_versions.iter().any(|v|v==platform.trim()),"Release does not support this Windows version");Ok(())
}

#[cfg(windows)]
pub fn verify_installed_metadata(state:&AgentState,directory:&Path)->Result<()> {
    let envelope:SignedEnvelope=serde_json::from_slice(&std::fs::read(directory.join("installed-release.json"))?)?;
    let release:Release=envelope.verify(&public_key(&state.bootstrap.release_public_key)?)?;
    // Previously installed software may start after metadata expiry to show
    // cached branding. This never authorizes a new install or remote session.
    release.validate(&state.bootstrap.edition,state.last_release_sequence.saturating_sub(1),now().min(release.expires_at.saturating_sub(1)))?;
    ensure!(state.last_release_sequence==0 || release.sequence==state.last_release_sequence,"Installed release sequence differs from state");
    verify_compatibility(directory,&release)?;
    verify_installed(directory,&release)
}

#[cfg(windows)]
fn save_installed_metadata(directory:&Path,envelope:&SignedEnvelope)->Result<()> {
    use std::io::Write;
    let temporary=directory.join(format!("installed-release-{}.tmp",random_token()));
    let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(&temporary)?;
    file.write_all(&serde_json::to_vec(envelope)?)?;file.sync_all()?;drop(file);
    crate::replace_state(&temporary,&directory.join("installed-release.json"))
}

#[cfg(windows)]
fn launch_update_helper(directory:&Path,release:&Release)->Result<()> {
    use std::os::windows::process::CommandExt;
    let folder=directory.join("updates").join(release.sequence.to_string());
    let helper=folder.join("swan-agent.exe");
    ensure!(digest(std::fs::read(&helper)?).eq_ignore_ascii_case(&release.agent_sha256),"Staged update helper hash mismatch");
    let mut agent_release=release.clone();agent_release.sha256=release.agent_sha256.clone();verify_publisher(&helper,&agent_release)?;
    use std::io::Write;
    let attempt=folder.join(format!("last-attempt-{}.tmp",random_token()));
    let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(&attempt)?;
    file.write_all(now().to_string().as_bytes())?;file.sync_all()?;drop(file);
    crate::replace_state(&attempt,&folder.join("last-attempt.txt"))?;
    let log=std::fs::OpenOptions::new().create(true).append(true).open(folder.join("apply-update.log"))?;
    std::process::Command::new(helper).arg("apply-update").env("SWAN_STATE_DIR",directory).env_remove("SWAN_TECHNICIAN_TOKEN")
        .creation_flags(0x08000000).stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::from(log.try_clone()?)).stderr(std::process::Stdio::from(log)).spawn()?;
    Ok(())
}

impl AgentState {
    #[cfg(windows)]
    pub fn record_installation(&mut self,directory:&Path,envelope:&SignedEnvelope,repair:bool)->Result<()> {
        let activity=activity_file(directory)?;fs2::FileExt::try_lock_exclusive(&activity).context("Installation verification requires all sessions to close")?;
        let mut latest=AgentState::load(directory)?;
        let release=installation_release(&latest,directory,envelope,repair)?;
        let setup_marker=directory.join("pending-install.json");
        if setup_marker.exists(){
            let pending:SignedEnvelope=serde_json::from_slice(&std::fs::read(&setup_marker)?)?;
            ensure!(pending.payload==envelope.payload && pending.signature==envelope.signature,"Another company installation must be recovered first");
        }
        verify_compatibility(directory,&release)?;
        verify_installed(directory,&release)?;
        save_installed_metadata(directory,envelope)?;
        latest.last_release_sequence=release.sequence;latest.save(directory)?;
        if setup_marker.exists(){std::fs::remove_file(setup_marker)?;}
        *self=latest;Ok(())
    }
    #[cfg(windows)]
    pub fn recover_update(&mut self,directory:&Path)->Result<()> {
        let activity=activity_file(directory)?;fs2::FileExt::try_lock_exclusive(&activity).context("Recovery requires all sessions to close")?;
        let receipt_path=directory.join("pending-update.json");
        let receipt:Receipt=serde_json::from_slice(&std::fs::read(&receipt_path)?)?;
        let release=validate_recovery_release(self,&receipt.release,receipt.previous_sequence,&receipt.phase)?;
        verify_compatibility(directory,&release)?;
        verify_installed(directory,&release)?;
        save_installed_metadata(directory,&receipt.release)?;
        // A crashed updater may have installed successfully or saved state before
        // removing the receipt. Exact signed executable identity proves either case.
        self.last_release_sequence=release.sequence;self.save(directory)?;
        std::fs::remove_file(receipt_path)?;Ok(())
    }
    #[cfg(windows)]
    pub fn resume_pending_update(&self,directory:&Path,retry_now:bool)->Result<bool> {
        let activity=activity_file(directory)?;fs2::FileExt::try_lock_exclusive(&activity).context("Update recovery already running or sessions remain active")?;
        let receipt_path=directory.join("pending-update.json");
        if !receipt_path.exists(){return Ok(false);}
        ensure!(!directory.join("pending-install.json").exists(),"Company setup recovery must finish first");
        let latest=AgentState::load_for_refresh(directory)?;
        let receipt:Receipt=serde_json::from_slice(&std::fs::read(&receipt_path)?)?;
        let release=validate_recovery_release(&latest,&receipt.release,receipt.previous_sequence,&receipt.phase)?;
        verify_compatibility(directory,&release)?;
        let attempt=directory.join("updates").join(release.sequence.to_string()).join("last-attempt.txt");
        if !retry_now && attempt.exists(){
            let previous=std::fs::read_to_string(attempt)?.parse::<u64>().context("Invalid update retry timestamp")?;
            if now().saturating_sub(previous)<300{return Ok(false);}
        }
        launch_update_helper(directory,&release)?;Ok(true)
    }
    #[cfg(windows)]
    pub fn apply_pending_update(&mut self,directory:&Path)->Result<bool> {
        // This method performs blocking Windows installation and is called from
        // spawn_blocking. A bounded wait lets the preparing process release its lock.
        let activity=activity_file(directory)?;
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {
            match fs2::FileExt::try_lock_exclusive(&activity) {
                Ok(())=>break,
                Err(error)=>{ensure!(std::time::Instant::now()<deadline,"Update handoff could not acquire session exclusion: {error}");std::thread::sleep(std::time::Duration::from_millis(100));}
            }
        }
        let receipt_path=directory.join("pending-update.json");
        if !receipt_path.exists(){return Ok(false);}
        ensure!(!directory.join("pending-install.json").exists(),"Company setup recovery must finish first");
        let receipt:Receipt=serde_json::from_slice(&std::fs::read(&receipt_path)?)?;
        let mut latest=AgentState::load_for_refresh(directory)?;
        let release=validate_recovery_release(&latest,&receipt.release,receipt.previous_sequence,&receipt.phase)?;
        verify_compatibility(directory,&release)?;
        let helper=std::env::current_exe()?;
        let mut agent_release=release.clone();agent_release.sha256=release.agent_sha256.clone();
        ensure!(digest(std::fs::read(&helper)?).eq_ignore_ascii_case(&release.agent_sha256),"Update helper differs from signed release");
        verify_publisher(&helper,&agent_release)?;
        let folder=directory.join("updates").join(release.sequence.to_string());
        let package=folder.join(format!("SwanRemoteSupport-install.{}",release.format));
        ensure!(digest(std::fs::read(&package)?).eq_ignore_ascii_case(&release.sha256),"Staged installer hash mismatch");
        verify_publisher(&package,&release)?;
        if let Err(error)=verify_installed(directory,&release) {
            eprintln!("Installed payload requires recovery: {error:#}");
            if latest.bootstrap.edition==Edition::Technician {
                ensure!(release.format=="msi" || (release.format=="exe" && release.sha256.eq_ignore_ascii_case(&release.installed_sha256)),"Technician updates require an MSI or a portable EXE");
                let target=installed_target(directory,&latest.bootstrap.edition)?;
                require_existing_application(&target)?;
                let previous=folder.join("previous-technician.exe");
                if !previous.exists(){std::fs::copy(&target,&previous)?;}
                if release.format=="msi" {
                    let status=msi_install_command(&package)?.arg(format!("INSTALLFOLDER={}",directory.display())).status()?;
                    ensure!(matches!(status.code(),Some(0|3010)),"Technician MSI installation failed; retain signed recovery receipt");
                }else{std::fs::copy(&package,&target)?;}
            }else{
                require_existing_application(&installed_target(directory,&latest.bootstrap.edition)?)?;
                let mut command=if release.format=="msi" {msi_install_command(&package)?}else{let mut c=std::process::Command::new(&package);c.args(["--silent-install","printer=0"]);c};
                let status=command.status()?;
                ensure!(matches!(status.code(),Some(0|3010)),"Installation failed; retain signed recovery receipt");
            }
            let installed_agent=directory.join("swan-agent.exe");
            let agent_already_current=installed_agent.is_file() && digest(std::fs::read(&installed_agent)?).eq_ignore_ascii_case(&release.agent_sha256);
            if !agent_already_current {
                let previous=folder.join("previous-agent.exe");
                if installed_agent.exists() && !previous.exists(){std::fs::copy(&installed_agent,&previous)?;}
                let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
                loop {
                    match std::fs::copy(&helper,&installed_agent) {
                        Ok(_)=>break,
                        Err(error) if matches!(error.raw_os_error(),Some(5|32|33)) && std::time::Instant::now()<deadline=>std::thread::sleep(std::time::Duration::from_millis(100)),
                        Err(error)=>return Err(error).context("Agent replacement failed; retain update receipt"),
                    }
                }
            }
        }
        verify_installed(directory,&release)?;
        save_installed_metadata(directory,&receipt.release)?;
        latest.last_release_sequence=release.sequence;latest.save(directory)?;
        std::fs::remove_file(receipt_path)?;*self=latest;Ok(true)
    }
    pub async fn update(&mut self,directory:&Path,technician_token:Option<&str>)->Result<bool> {
        #[cfg(not(windows))] {let _=(directory,technician_token);anyhow::bail!("Endpoint updates require Windows");}
        #[cfg(windows)] {
            ensure!(!directory.join("pending-install.json").exists(),"Company setup recovery is required before automatic updates");
            if directory.join("pending-update.json").exists(){return self.resume_pending_update(directory,false);}
            require_existing_application(&installed_target(directory,&self.bootstrap.edition)?)?;
            let profile=self.company_profile()?;
            if profile.updates_paused || !maintenance_open(profile.maintenance_start_utc,profile.maintenance_end_utc,now()){return Ok(false);}
            let receipt_path=directory.join("pending-update.json");
            // An interrupted installation is deliberately not guessed successful.
            // Keep the signed receipt so recovery can verify the installed build.
            ensure!(!receipt_path.exists(),"Interrupted update requires recovery before another installation");
            let (path,token)=if self.bootstrap.edition==Edition::Customer {("device/update",self.device_token.as_deref().context("Not enrolled")?)}else{("user/update",technician_token.context("Technician authentication required")?)};
            let value:Value=self.client()?.get(self.endpoint(path)?).bearer_auth(token).send().await?.error_for_status()?.json().await?;
            if value.is_null(){return Ok(false);}
            let envelope:SignedEnvelope=serde_json::from_value(value)?;
            let release=validate_release(self,&envelope)?;
            ensure!(release.edition!=Edition::Technician || (release.format=="msi" || (release.format=="exe" && release.installed_sha256.eq_ignore_ascii_case(&release.sha256))),"Technician updates require an MSI or a portable EXE with matching installed identity");
            let folder=directory.join("updates").join(release.sequence.to_string());std::fs::create_dir_all(&folder)?;
            verify_compatibility(&folder,&release)?;
            let package=folder.join(format!("SwanRemoteSupport-install.{}",release.format));
            download(&release.artifact_url,&release.sha256,&package).await?;
            verify_publisher(&package,&release)?;
            let replacement_agent=folder.join("swan-agent.exe");
            download(&release.agent_url,&release.agent_sha256,&replacement_agent).await?;
            let mut agent_release=release.clone();agent_release.sha256=release.agent_sha256.clone();
            verify_publisher(&replacement_agent,&agent_release)?;
            let activity=activity_file(directory)?;
            fs2::FileExt::try_lock_exclusive(&activity).context("Update deferred while a session or technician app is active")?;
            ensure!(!directory.join("pending-install.json").exists(),"Company setup started while the update was downloading");
            // Re-read consent and enrollment immediately before installation.
            let latest=AgentState::load(directory)?;
            // Removal during download must not be turned into a fresh install.
            require_existing_application(&installed_target(directory,&latest.bootstrap.edition)?)?;
            validate_release(&latest,&envelope)?;
            let receipt=Receipt{release:envelope,previous_sequence:latest.last_release_sequence,phase:"installing".into()};
            let mut options=std::fs::OpenOptions::new();options.create_new(true).write(true);
            use std::io::Write;
            let mut file=options.open(&receipt_path)?;file.write_all(&serde_json::to_vec(&receipt)?)?;file.sync_all()?;drop(file);
            // The signed replacement runs from staging. This caller must exit so
            // Windows releases the old executable before it is overwritten.
            launch_update_helper(directory,&release)?;
            *self=latest;Ok(true)
        }
    }
}

#[cfg(windows)]
fn verify_installed(directory:&Path,release:&Release)->Result<()> {
    let agent=directory.join("swan-agent.exe");
    ensure!(digest(std::fs::read(&agent)?).eq_ignore_ascii_case(&release.agent_sha256),"Installed configuration agent differs from signed release; recovery remains pending");
    let mut agent_release=release.clone();agent_release.sha256=release.agent_sha256.clone();verify_publisher(&agent,&agent_release)?;
    let target=installed_target(directory,&release.edition)?;
    // Portable technician state also contains the agent and updater. Its single
    // packed EXE is verified separately; customer files share an install root.
    if release.edition==Edition::Customer {
        let mut files=release.installed_files.clone();
        // RustDesk copies this Windows-owned process for input/privacy support.
        // It varies with Windows updates and is not a project release artifact.
        let mut system_directory=[0u16;32768];
        let length=unsafe{windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW(system_directory.as_mut_ptr(),system_directory.len() as u32)} as usize;
        ensure!(length>0 && length<system_directory.len(),"Cannot resolve trusted Windows system directory");
        let system_directory=std::path::PathBuf::from(String::from_utf16(&system_directory[..length])?);
        files.push(InstalledFile{path:"RuntimeBroker_rustdesk.exe".into(),sha256:digest(std::fs::read(system_directory.join("RuntimeBroker.exe"))?)});
        crate::payload::verify(target.parent().context("Missing installation directory")?,&files)?;
    }
    ensure!(digest(std::fs::read(&target)?).eq_ignore_ascii_case(&release.installed_sha256),"Installed executable differs from signed release; recovery remains pending");
    let mut installed=release.clone();installed.sha256=release.installed_sha256.clone();verify_publisher(&target,&installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn interrupted_setup_blocks_sessions_until_recovered(){
        let folder=std::env::temp_dir().join(format!("swan-pending-install-{}",random_token()));
        std::fs::create_dir(&folder).unwrap();std::fs::write(folder.join("pending-install.json"),b"interrupted setup fixture").unwrap();
        assert!(lock_session(&folder).is_err());
        std::fs::remove_file(folder.join("pending-install.json")).unwrap();
        assert!(lock_session(&folder).is_ok());std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn interrupted_update_blocks_sessions_after_process_lock_is_released(){
        let folder=std::env::temp_dir().join(format!("swan-pending-update-{}",random_token()));
        std::fs::create_dir(&folder).unwrap();
        let updater=activity_file(&folder).unwrap();fs2::FileExt::try_lock_exclusive(&updater).unwrap();
        std::fs::write(folder.join("pending-update.json"),b"interrupted update fixture").unwrap();
        drop(updater);
        assert!(lock_session(&folder).is_err());
        std::fs::remove_file(folder.join("pending-update.json")).unwrap();
        assert!(lock_session(&folder).is_ok());std::fs::remove_dir_all(folder).unwrap();
    }
    #[cfg(windows)]
    #[test]
    fn powershell_installer_byte_lock_excludes_native_sessions(){
        use std::io::{BufRead,Write};use std::os::windows::process::CommandExt;
        let folder=std::env::temp_dir().join(format!("swan-installer-lock-{}",random_token()));std::fs::create_dir(&folder).unwrap();
        let script=folder.join("hold-install-lock.ps1");
        std::fs::write(&script,r#"param([string]$Directory)
$ErrorActionPreference='Stop'
$stream=[IO.File]::Open((Join-Path $Directory 'activity.lock'),[IO.FileMode]::OpenOrCreate,[IO.FileAccess]::ReadWrite,[IO.FileShare]::ReadWrite)
try {$stream.Lock(0,1);[Console]::WriteLine('locked');[Console]::ReadLine()|Out-Null}finally{$stream.Dispose()}
"#).unwrap();
        let mut child=std::process::Command::new("powershell.exe").args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(&script).arg(&folder).creation_flags(0x08000000).stdin(std::process::Stdio::piped()).stdout(std::process::Stdio::piped()).spawn().unwrap();
        let mut output=std::io::BufReader::new(child.stdout.take().unwrap());let mut line=String::new();output.read_line(&mut line).unwrap();
        let ready=line.trim()=="locked";let blocked=lock_session(&folder).is_err();
        if let Some(mut input)=child.stdin.take(){input.write_all(b"release\n").unwrap();}
        let status=child.wait().unwrap();assert!(ready && blocked && status.success());
        assert!(lock_session(&folder).is_ok());std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn removed_application_requires_explicit_setup_instead_of_automatic_reinstall() {
        let folder=std::env::temp_dir().join(format!("swan-removed-{}",random_token()));
        std::fs::create_dir(&folder).unwrap();let target=folder.join("endpoint.exe");
        assert!(require_existing_application(&target).is_err());
        std::fs::write(&target,b"installed test fixture").unwrap();
        assert!(require_existing_application(&target).is_ok());
        std::fs::remove_file(&target).unwrap();
        assert!(require_existing_application(&target).is_err());
        std::fs::remove_dir(folder).unwrap();
    }
    #[test]
    fn update_lock_excludes_sessions_and_releases_on_close() {
        let folder=std::env::temp_dir().join(format!("swan-lock-{}",random_token()));
        let session=lock_session(&folder).unwrap();let update=activity_file(&folder).unwrap();
        assert!(fs2::FileExt::try_lock_exclusive(&update).is_err());drop(session);
        fs2::FileExt::try_lock_exclusive(&update).unwrap();assert!(lock_session(&folder).is_err());drop(update);
        assert!(lock_session(&folder).is_ok());std::fs::remove_dir_all(folder).unwrap();
    }
}
