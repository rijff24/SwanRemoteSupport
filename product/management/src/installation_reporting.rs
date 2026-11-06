use super::*;

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Page { #[serde(default)] offset: u64 }

fn owner(s: &Store, headers: &HeaderMap, identity: &InstallationIdentity) -> Result<String, ApiError> {
    match identity.edition {
        Edition::Customer => {
            let device = s.device(headers)?;
            if identity.installation_id != device { return Err(denied()); }
            Ok(device)
        }
        Edition::Technician => s.user(headers, false),
    }
}
fn kind(edition: &Edition) -> &'static str {
    match edition { Edition::Customer => "customer", Edition::Technician => "technician" }
}
async fn blocking(task: impl FnOnce() -> ApiResult + Send + 'static) -> ApiResult {
    tokio::task::spawn_blocking(task).await.map_err(|_| ApiError(StatusCode::INTERNAL_SERVER_ERROR, "Installation reporting unavailable"))?
}

pub(super) async fn report(State(s): State<Shared>, headers: HeaderMap, Json(input): Json<InstallationReport>) -> ApiResult {
    input.validate().map_err(|_| bad("Invalid installation report"))?;
    blocking(move || {
        let actor = owner(&s, &headers, &input.identity)?;
        s.rate(&format!("installation-actor:{actor}"), 240)?;
        s.rate(&format!("installation:{actor}:{}", input.identity.installation_id), 60)?;
        let body = serde_json::to_string(&input)?;
        let hash = digest(token(&headers)?);
        let mut db = s.db.lock().unwrap();
        let tx = db.transaction()?;
        // Recheck inside the write transaction: revocation/disable/logout must
        // not race the earlier ownership lookup.
        let active: bool = match input.identity.edition {
            Edition::Customer => tx.query_row("SELECT EXISTS(SELECT 1 FROM devices WHERE id=?1 AND token_hash=?2 AND state='approved')", params![actor, hash], |row| row.get(0))?,
            Edition::Technician => tx.query_row("SELECT EXISTS(SELECT 1 FROM sessions s JOIN users u ON u.id=s.user_id WHERE u.id=?1 AND s.token_hash=?2 AND s.expires_at>?3 AND u.disabled=0)", params![actor, hash, now()], |row| row.get(0))?,
        };
        if !active { return Err(unauthorized()); }
        let previous: Option<(u64, String)> = tx.query_row("SELECT revision,body FROM installation_reports WHERE owner_kind=?1 AND owner_id=?2 AND installation_id=?3", params![kind(&input.identity.edition), actor, input.identity.installation_id], |row| Ok((row.get(0)?, row.get(1)?))).optional()?;
        if let Some((revision, old)) = &previous {
            if input.revision < *revision || (input.revision == *revision && old != &body) {
                return Err(ApiError(StatusCode::CONFLICT, "Installation report superseded"));
            }
            if input.revision == *revision {
                // Replaying the same message does not refresh its last-seen time.
                return Ok(Json(json!({"accepted":true,"revision":revision})));
            }
        }
        tx.execute("INSERT INTO installation_reports(owner_kind,owner_id,installation_id,revision,received_at,body) VALUES(?1,?2,?3,?4,?5,?6) ON CONFLICT(owner_kind,owner_id,installation_id) DO UPDATE SET revision=excluded.revision,received_at=excluded.received_at,body=excluded.body", params![kind(&input.identity.edition), actor, input.identity.installation_id, input.revision, now(), body])?;
        tx.commit()?;
        // This diagnostic never changes a device, permission, release or grant.
        Ok(Json(json!({"accepted":true,"revision":input.revision})))
    }).await
}

pub(super) async fn own_report(State(s): State<Shared>, headers: HeaderMap, Query(identity): Query<InstallationIdentity>) -> ApiResult {
    identity.validate().map_err(|_| bad("Invalid installation identity"))?;
    blocking(move || {
        let actor = owner(&s, &headers, &identity)?;
        let row: Option<(u64, u64, String)> = s.db.lock().unwrap().query_row("SELECT revision,received_at,body FROM installation_reports WHERE owner_kind=?1 AND owner_id=?2 AND installation_id=?3", params![kind(&identity.edition), actor, identity.installation_id], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?))).optional()?;
        Ok(Json(match row {
            Some((revision, received, body)) => json!({"revision":revision,"received_at":received,"report":serde_json::from_str::<InstallationReport>(&body)?}),
            None => Value::Null,
        }))
    }).await
}

pub(super) async fn reports(State(s): State<Shared>, headers: HeaderMap, Query(page): Query<Page>) -> ApiResult {
    if page.offset > i64::MAX as u64 - 201 { return Err(bad("Invalid report page")); }
    blocking(move || {
        let actor = s.user(&headers, false)?;
        s.rate(&format!("installation-list:{actor}"), 60)?;
        let db = s.db.lock().unwrap();
        let mut statement = db.prepare("SELECT r.owner_kind,r.owner_id,r.installation_id,r.received_at,r.body,COALESCE(d.name,u.username,'Unknown installation'),COALESCE(d.state,CASE WHEN u.disabled=0 THEN 'active' ELSE 'disabled' END) FROM installation_reports r LEFT JOIN devices d ON r.owner_kind='customer' AND d.id=r.owner_id LEFT JOIN users u ON r.owner_kind='technician' AND u.id=r.owner_id WHERE EXISTS(SELECT 1 FROM users a WHERE a.id=?1 AND a.role='admin') OR (r.owner_kind='technician' AND r.owner_id=?1) OR (r.owner_kind='customer' AND d.state='approved' AND EXISTS(SELECT 1 FROM group_access g WHERE g.group_id=d.group_id AND g.user_id=?1)) ORDER BY r.received_at DESC,r.owner_kind,r.owner_id,r.installation_id LIMIT 201 OFFSET ?2")?;
        let rows = statement.query_map(params![actor, page.offset as i64], |row| Ok((row.get::<_,String>(0)?, row.get::<_,String>(1)?, row.get::<_,String>(2)?, row.get::<_,u64>(3)?, row.get::<_,String>(4)?, row.get::<_,String>(5)?, row.get::<_,String>(6)?)))?;
        let mut reports = Vec::new();
        for row in rows {
            let (edition, owner, installation, received, body, name, state) = row?;
            let report: InstallationReport = serde_json::from_str(&body)?;
            reports.push(json!({"edition":edition,"owner_id":owner,"installation_id":installation,"owner_name":name,"owner_state":state,"received_at":received,"report":report}));
        }
        let next = if reports.len() > 200 { reports.truncate(200); Some(page.offset + 200) } else { None };
        Ok(Json(json!({"reports":reports,"next_offset":next})))
    }).await
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::{body::Body, http::Request};
    use tower::ServiceExt;

    async fn call(app: &Router, path: &str, method: &str, credential: Option<&str>, body: Value) -> (StatusCode, Value) {
        let mut request = Request::builder().method(method).uri(path).header(header::CONTENT_TYPE, "application/json");
        if let Some(credential) = credential { request = request.header(header::AUTHORIZATION, format!("Bearer {credential}")); }
        let response = app.clone().oneshot(request.body(Body::from(body.to_string())).unwrap()).await.unwrap();
        let status = response.status();
        let bytes = axum::body::to_bytes(response.into_body(), 2 * 1024 * 1024).await.unwrap();
        (status, serde_json::from_slice(&bytes).unwrap())
    }
    fn report(edition: Edition, id: &str) -> InstallationReport {
        InstallationReport { schema: 1, identity: InstallationIdentity { edition, installation_id: id.into() }, revision: 1, installed_sequence: 9, installed_version: Some("1.5.7".into()), phase: UpdatePhase::Downloading, target_sequence: Some(10), downloaded_bytes: 25, total_bytes: Some(100) }
    }

    #[tokio::test]
    async fn reports_are_owned_permission_filtered_and_never_authorize_sessions() {
        let directory = std::env::temp_dir().join(format!("swan-installation-report-{}", random_token()));
        let store = Arc::new(Store::open(&directory).unwrap());
        let credentials: Vec<_> = (0..6).map(|_| random_token()).collect();
        let device = "a0000000-0000-4000-8000-000000000001";
        let other_device = "a0000000-0000-4000-8000-000000000002";
        let installation = "b0000000-0000-4000-8000-000000000001";
        {
            let db = store.db.lock().unwrap();
            for (index, role) in ["admin", "technician", "technician", "technician"].into_iter().enumerate() {
                let user = format!("user-{index}");
                db.execute("INSERT INTO users(id,username,password_hash,totp_secret,role) VALUES(?1,?1,'fixture','fixture',?2)", params![user, role]).unwrap();
                db.execute("INSERT INTO sessions VALUES(?1,?2,?3)", params![digest(&credentials[index]), user, now() + 3600]).unwrap();
            }
            db.execute("INSERT INTO group_access VALUES('group-a','user-1')", []).unwrap();
            for (index, id) in [device, other_device].into_iter().enumerate() {
                db.execute("INSERT INTO devices(id,name,rustdesk_id,token_hash,state,group_id) VALUES(?1,?1,?2,?3,'approved',?4)", params![id, format!("peer-{index}"), digest(&credentials[index + 4]), if index == 0 { "group-a" } else { "group-b" }]).unwrap();
            }
        }
        let app = router(store.clone());
        let customer = report(Edition::Customer, device);
        let technician = report(Edition::Technician, installation);
        let path = "/api/v1/installations/report";
        for (credential, body, expected) in [
            (None, &customer, StatusCode::UNAUTHORIZED),
            (Some(credentials[1].as_str()), &customer, StatusCode::UNAUTHORIZED),
            (Some(credentials[4].as_str()), &technician, StatusCode::UNAUTHORIZED),
            (Some(credentials[5].as_str()), &customer, StatusCode::FORBIDDEN),
            (Some(credentials[4].as_str()), &customer, StatusCode::OK),
            (Some(credentials[1].as_str()), &technician, StatusCode::OK),
            (Some(credentials[2].as_str()), &technician, StatusCode::OK),
        ] {
            assert_eq!(call(&app, path, "POST", credential, serde_json::to_value(body).unwrap()).await.0, expected);
        }
        for (index, count) in [(0, 3), (1, 2), (2, 1), (3, 0)] {
            let (status, rows) = call(&app, "/api/v1/installations", "GET", Some(&credentials[index]), Value::Null).await;
            assert_eq!(status, StatusCode::OK); assert_eq!(rows["reports"].as_array().unwrap().len(), count);
        }
        let own = format!("{path}?edition=customer&installation_id={device}");
        store.db.lock().unwrap().execute("UPDATE installation_reports SET received_at=1 WHERE owner_kind='customer'", []).unwrap();
        assert_eq!(call(&app, path, "POST", Some(&credentials[4]), serde_json::to_value(&customer).unwrap()).await.0, StatusCode::OK);
        assert_eq!(call(&app, &own, "GET", Some(&credentials[4]), Value::Null).await.1["received_at"], 1, "Replay must not claim a fresh heartbeat");
        let mut newer = customer.clone(); newer.revision = 2; newer.downloaded_bytes = 50;
        assert_eq!(call(&app, path, "POST", Some(&credentials[4]), serde_json::to_value(&newer).unwrap()).await.0, StatusCode::OK);
        assert_eq!(call(&app, path, "POST", Some(&credentials[4]), serde_json::to_value(&customer).unwrap()).await.0, StatusCode::CONFLICT);
        newer.downloaded_bytes = 51;
        assert_eq!(call(&app, path, "POST", Some(&credentials[4]), serde_json::to_value(&newer).unwrap()).await.0, StatusCode::CONFLICT);
        assert_eq!(call(&app, &format!("{path}?edition=technician&installation_id={installation}"), "GET", Some(&credentials[3]), Value::Null).await.1, Value::Null, "Other technicians cannot retrieve this installation report");
        {
            let db = store.db.lock().unwrap();
            assert_eq!(db.query_row("SELECT COUNT(*) FROM grants", [], |row| row.get::<_,u64>(0)).unwrap(), 0);
            assert!(!db.query_row("SELECT unattended FROM devices WHERE id=?1", [device], |row| row.get::<_,bool>(0)).unwrap());
            db.execute("UPDATE devices SET state='revoked' WHERE id=?1", [device]).unwrap();
            db.execute("UPDATE users SET disabled=1 WHERE id='user-1'", []).unwrap();
        }
        assert_eq!(call(&app, path, "POST", Some(&credentials[4]), serde_json::to_value(&customer).unwrap()).await.0, StatusCode::UNAUTHORIZED);
        assert_eq!(call(&app, &own, "GET", Some(&credentials[4]), Value::Null).await.0, StatusCode::UNAUTHORIZED);
        assert_eq!(call(&app, path, "POST", Some(&credentials[1]), serde_json::to_value(&technician).unwrap()).await.0, StatusCode::UNAUTHORIZED);
        assert_eq!(call(&app, "/api/v1/installations", "GET", Some(&credentials[1]), Value::Null).await.0, StatusCode::UNAUTHORIZED);
        store.db.lock().unwrap().execute("UPDATE devices SET state='pending' WHERE id=?1", [other_device]).unwrap();
        assert_eq!(call(&app, path, "POST", Some(&credentials[5]), serde_json::to_value(report(Edition::Customer, other_device)).unwrap()).await.0, StatusCode::UNAUTHORIZED);
        // Pagination fixtures use valid report rows in one transaction. They
        // exercise the real HTTP reader, not receipt ingestion or remote MFA.
        {
            let mut db = store.db.lock().unwrap(); let tx = db.transaction().unwrap();
            for _ in 0..210 {
                let input = report(Edition::Technician, &Uuid::new_v4().to_string());
                tx.execute("INSERT INTO installation_reports VALUES('technician','user-2',?1,1,?2,?3)", params![input.identity.installation_id, now(), serde_json::to_string(&input).unwrap()]).unwrap();
            }
            tx.commit().unwrap();
        }
        let first = call(&app, "/api/v1/installations", "GET", Some(&credentials[0]), Value::Null).await.1;
        let second = call(&app, "/api/v1/installations?offset=200", "GET", Some(&credentials[0]), Value::Null).await.1;
        assert_eq!(first["reports"].as_array().unwrap().len(), 200); assert_eq!(first["next_offset"], 200);
        assert_eq!(second["reports"].as_array().unwrap().len(), 13); assert!(second["next_offset"].is_null());
        let own_page = call(&app, "/api/v1/installations?offset=200", "GET", Some(&credentials[2]), Value::Null).await.1;
        assert_eq!(own_page["reports"].as_array().unwrap().len(), 11);
        assert_eq!(call(&app, "/api/v1/installations?offset=18446744073709551615", "GET", Some(&credentials[0]), Value::Null).await.0, StatusCode::BAD_REQUEST);
        let foreign_directory = directory.with_extension("other-company");
        let foreign_store = Arc::new(Store::open(&foreign_directory).unwrap()); let foreign_app = router(foreign_store.clone());
        assert_eq!(call(&foreign_app, path, "POST", Some(&credentials[4]), serde_json::to_value(&customer).unwrap()).await.0, StatusCode::UNAUTHORIZED);
        assert_eq!(foreign_store.db.lock().unwrap().query_row("SELECT COUNT(*) FROM installation_reports", [], |row| row.get::<_,u64>(0)).unwrap(), 0);
        drop(foreign_app); drop(foreign_store); std::fs::remove_dir_all(foreign_directory).unwrap();
        drop(app); drop(store); std::fs::remove_dir_all(directory).unwrap();
    }

    #[test]
    fn schema_two_upgrade_requires_backup_and_restores_original_schema_and_keys() {
        let directory = std::env::temp_dir().join(format!("swan-report-migration-{}", random_token()));
        let original = Store::open(&directory).unwrap(); let public = original.key.verifying_key();
        original.db.lock().unwrap().execute_batch("DROP TABLE installation_reports; PRAGMA user_version=2;").unwrap(); drop(original);
        assert!(Store::open_with_backup_password(&directory, None).is_err());
        let upgraded = Store::open_with_backup_password(&directory, Some("schema three backup fixture password")).unwrap();
        assert_eq!(upgraded.key.verifying_key(), public);
        assert_eq!(upgraded.db.lock().unwrap().pragma_query_value(None, "user_version", |row| row.get::<_,u32>(0)).unwrap(), 3);
        let archive = std::fs::read_dir(&directory).unwrap().map(|entry| entry.unwrap().path()).find(|path| path.file_name().unwrap().to_string_lossy().starts_with("pre-upgrade-v2-")).unwrap();
        drop(upgraded);
        let restored = directory.with_extension("restored");
        crate::backup::restore(&restored, &archive, "schema three backup fixture password").unwrap();
        let db = Connection::open(restored.join("management.sqlite3")).unwrap();
        assert_eq!(db.pragma_query_value(None, "user_version", |row| row.get::<_,u32>(0)).unwrap(), 2);
        assert_eq!(db.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='installation_reports'", [], |row| row.get::<_,u64>(0)).unwrap(), 0);
        assert_eq!(std::fs::read(restored.join("profile-key.hex")).unwrap(), std::fs::read(directory.join("profile-key.hex")).unwrap());
        drop(db); std::fs::remove_dir_all(directory).unwrap(); std::fs::remove_dir_all(restored).unwrap();
    }

    #[test]
    fn failed_schema_upgrade_rolls_back_all_ddl() {
        let directory = std::env::temp_dir().join(format!("swan-report-migration-failure-{}", random_token()));
        let original = Store::open(&directory).unwrap();
        original.db.lock().unwrap().execute_batch("DROP TABLE profile_signing_keys; DROP TABLE installation_reports; CREATE TABLE installation_reports(invalid TEXT); PRAGMA user_version=1;").unwrap();
        drop(original);
        assert!(Store::open_with_backup_password(&directory, Some("failed migration backup password")).is_err());
        let db = Connection::open(directory.join("management.sqlite3")).unwrap();
        assert_eq!(db.pragma_query_value(None, "user_version", |row| row.get::<_,u32>(0)).unwrap(), 1);
        assert_eq!(db.query_row("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='profile_signing_keys'", [], |row| row.get::<_,u64>(0)).unwrap(), 0, "DDL before the failing index must not partially migrate the database");
        drop(db); std::fs::remove_dir_all(directory).unwrap();
    }
}
