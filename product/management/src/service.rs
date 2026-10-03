use anyhow::ensure;
use argon2::{password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use axum::{extract::{DefaultBodyLimit, Path, Query, State}, http::{header, HeaderMap, StatusCode}, response::{Html, IntoResponse, Response}, routing::{get, post, put}, Json, Router};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use rusqlite::{params, Connection, OptionalExtension};
use serde::Deserialize;
use serde_json::{json, Value};
use std::{collections::HashMap, path::{Path as FsPath,PathBuf}, sync::{Arc, Mutex}};
use swan_protocol::*;
use totp_rs::{Algorithm, Secret, TOTP};
use uuid::Uuid;

pub struct Store {
    _process_lock: std::fs::File,
    db: Mutex<Connection>,
    key: SigningKey,
    setup_hash: String,
    limits: Mutex<HashMap<String,(u64,u32)>>,
    artifact_dir: PathBuf,
}
type Shared = Arc<Store>;
type ApiResult = Result<Json<Value>, ApiError>;
struct ApiError(StatusCode, &'static str);
impl IntoResponse for ApiError {
    fn into_response(self) -> Response { (self.0,Json(json!({"error":self.1}))).into_response() }
}
impl From<anyhow::Error> for ApiError {
    fn from(error:anyhow::Error)->Self { eprintln!("Management operation failed: {}",error.root_cause()); Self(StatusCode::INTERNAL_SERVER_ERROR,"Operation failed") }
}
impl From<rusqlite::Error> for ApiError {
    fn from(error:rusqlite::Error)->Self { anyhow::Error::from(error).into() }
}
impl From<serde_json::Error> for ApiError {
    fn from(error:serde_json::Error)->Self { anyhow::Error::from(error).into() }
}
fn bad(message:&'static str)->ApiError { ApiError(StatusCode::BAD_REQUEST,message) }
fn denied()->ApiError { ApiError(StatusCode::FORBIDDEN,"Access denied") }
fn unauthorized()->ApiError { ApiError(StatusCode::UNAUTHORIZED,"Authentication required") }
fn token(headers:&HeaderMap)->Result<&str,ApiError> {
    headers.get(header::AUTHORIZATION).and_then(|v|v.to_str().ok()).and_then(|v|v.strip_prefix("Bearer ")).filter(|v|v.len()>=32 && v.len()<=256).ok_or_else(unauthorized)
}
fn password_hash(password:&str)->anyhow::Result<String> {
    ensure!(password.len()>=14 && password.len()<=256,"Password must contain 14–256 characters");
    Ok(Argon2::default().hash_password(password.as_bytes(),&SaltString::generate(&mut OsRng)).map_err(|_|anyhow::anyhow!("Password hashing failed"))?.to_string())
}
fn totp(secret:&str)->anyhow::Result<TOTP> {
    Ok(TOTP::new(Algorithm::SHA1,6,1,30,Secret::Encoded(secret.into()).to_bytes().map_err(|_|anyhow::anyhow!("Invalid TOTP secret"))?).map_err(|_|anyhow::anyhow!("Invalid TOTP parameters"))?)
}
fn totp_step(secret:&str,code:&str,time:u64)->anyhow::Result<u64> {
    ensure!(code.len()==6 && code.bytes().all(|b|b.is_ascii_digit()),"Invalid code");
    let generator=totp(secret)?;
    for step in [(time/30).saturating_sub(1),time/30,time/30+1] {
        if generator.generate(step*30)==code { return Ok(step); }
    }
    anyhow::bail!("Invalid code")
}

impl Store {
    pub fn open(directory:&FsPath)->anyhow::Result<Self> {
        Self::open_with_backup_password(directory,std::env::var("SWAN_BACKUP_PASSPHRASE").ok().as_deref())
    }
    fn open_with_backup_password(directory:&FsPath,backup_password:Option<&str>)->anyhow::Result<Self> {
        std::fs::create_dir_all(directory)?;
        #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(directory,std::fs::Permissions::from_mode(0o700))?; }
        let mut lock_options=std::fs::OpenOptions::new();lock_options.create(true).read(true).write(true);
        #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;lock_options.mode(0o600);}
        let process_lock=lock_options.open(directory.join("management.lock"))?;
        fs2::FileExt::try_lock_exclusive(&process_lock).map_err(|_|anyhow::anyhow!("Company management data is in use; stop the service before offline operations"))?;
        let key_path=directory.join("profile-key.hex");
        if !key_path.exists() { write_secret(&key_path,&hex::encode(SigningKey::generate(&mut OsRng).to_bytes()))?; }
        let key_bytes:[u8;32]=hex::decode(std::fs::read_to_string(&key_path)?.trim())?.try_into().map_err(|_|anyhow::anyhow!("Invalid signing key"))?;
        let setup_path=directory.join("setup-token.txt");
        if !setup_path.exists() { write_secret(&setup_path,&random_token())?; }
        let setup_hash=digest(std::fs::read_to_string(setup_path)?.trim());
        let db=Connection::open(directory.join("management.sqlite3"))?;
        db.pragma_update(None,"journal_mode","WAL")?;
        db.pragma_update(None,"foreign_keys","ON")?;
        db.busy_timeout(std::time::Duration::from_secs(5))?;
        let version:u32=db.pragma_query_value(None,"user_version",|r|r.get(0))?;
        ensure!(version<=2,"Database was created by a newer server; restore the pre-upgrade backup rather than downgrading it");
        if version==1 {
            let password=backup_password.ok_or_else(||anyhow::anyhow!("Schema upgrade requires SWAN_BACKUP_PASSPHRASE in private environment configuration for an encrypted pre-upgrade backup"))?;
            crate::backup::export(directory,&directory.join(format!("pre-upgrade-v1-{}.swan-backup",random_token())),password)?;
        }
        db.execute_batch(include_str!("schema.sql"))?;
        db.execute("INSERT OR IGNORE INTO profile_signing_keys(id,active_hex) VALUES(1,?1)",[hex::encode(key_bytes)])?;
        let active_hex:String=db.query_row("SELECT active_hex FROM profile_signing_keys WHERE id=1",[],|row|row.get(0))?;
        let key=signing_key_from_hex(&active_hex)?;
        let artifact_dir=directory.join("artifacts");std::fs::create_dir_all(&artifact_dir)?;
        Ok(Self {_process_lock:process_lock,db:Mutex::new(db),key,setup_hash,limits:Mutex::new(HashMap::new()),artifact_dir})
    }
    pub fn prepare_profile_rotation(&self)->anyhow::Result<String> {
        let mut db=self.db.lock().unwrap();let tx=db.transaction()?;
        let pending:Option<String>=tx.query_row("SELECT pending_hex FROM profile_signing_keys WHERE id=1",[],|row|row.get(0))?;
        ensure!(pending.is_none(),"A profile key rotation is already pending");
        let raw:String=tx.query_row("SELECT body FROM profile WHERE id=1",[],|row|row.get(0))?;
        let mut profile:CompanyProfile=serde_json::from_str(&raw)?;
        ensure!(profile.next_profile_public_key.is_none(),"Profile already declares a replacement key");
        let next=SigningKey::generate(&mut OsRng);let public=STANDARD.encode(next.verifying_key().as_bytes());
        profile.next_profile_public_key=Some(public.clone());profile.revision=profile.revision.checked_add(1).ok_or_else(||anyhow::anyhow!("Profile revision exhausted"))?;
        profile.issued_at=now();profile.expires_at=now()+30*86400;profile.validate(&profile.company_id,profile.revision,now())?;
        tx.execute("UPDATE profile_signing_keys SET pending_hex=?1 WHERE id=1",[hex::encode(next.to_bytes())])?;
        tx.execute("UPDATE profile SET body=?1 WHERE id=1",[serde_json::to_string(&profile)?])?;
        tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,'local-operator','profile.key.prepared',?2)",params![now(),public])?;
        tx.commit()?;Ok(public)
    }
    pub fn activate_profile_rotation(&self)->anyhow::Result<String> {
        let mut db=self.db.lock().unwrap();let tx=db.transaction()?;
        let pending:Option<String>=tx.query_row("SELECT pending_hex FROM profile_signing_keys WHERE id=1",[],|row|row.get(0))?;
        let pending=pending.ok_or_else(||anyhow::anyhow!("No pending profile key rotation"))?;
        let next=signing_key_from_hex(&pending)?;let public=STANDARD.encode(next.verifying_key().as_bytes());
        let raw:String=tx.query_row("SELECT body FROM profile WHERE id=1",[],|row|row.get(0))?;let mut profile:CompanyProfile=serde_json::from_str(&raw)?;
        ensure!(profile.next_profile_public_key.as_ref()==Some(&public),"Published profile does not authorize the pending key");
        profile.next_profile_public_key=None;profile.revision=profile.revision.checked_add(1).ok_or_else(||anyhow::anyhow!("Profile revision exhausted"))?;
        profile.issued_at=now();profile.expires_at=now()+30*86400;profile.validate(&profile.company_id,profile.revision,now())?;
        tx.execute("UPDATE profile_signing_keys SET active_hex=?1,pending_hex=NULL WHERE id=1",[pending])?;
        tx.execute("UPDATE profile SET body=?1 WHERE id=1",[serde_json::to_string(&profile)?])?;
        tx.execute("UPDATE grants SET closed=1,lease_until=0 WHERE closed=0",[])?;
        tx.execute("UPDATE builds SET state='failed',result=?1 WHERE state IN ('queued','running','uploaded','completed')",[json!({"ok":false,"error":"Company signing key rotated; regenerate company bundles"}).to_string()])?;
        tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,'local-operator','profile.key.activated',?2)",params![now(),public])?;
        tx.commit()?;Ok(public)
    }
    fn profile(&self)->anyhow::Result<CompanyProfile> {
        let raw:String=self.db.lock().unwrap().query_row("SELECT body FROM profile WHERE id=1",[],|r|r.get(0))?;
        Ok(serde_json::from_str(&raw)?)
    }
    fn user(&self,headers:&HeaderMap,admin:bool)->Result<String,ApiError> {
        let hash=digest(token(headers)?);
        let result:Option<(String,String)>=self.db.lock().unwrap().query_row("SELECT u.id,u.role FROM sessions s JOIN users u ON u.id=s.user_id WHERE s.token_hash=?1 AND s.expires_at>?2 AND u.disabled=0",params![hash,now()],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
        match result { Some((id,role)) if !admin || role=="admin" => Ok(id),_=>Err(unauthorized()) }
    }
    fn device(&self,headers:&HeaderMap)->Result<String,ApiError> {
        self.db.lock().unwrap().query_row("SELECT id FROM devices WHERE token_hash=?1 AND state='approved'",[digest(token(headers)?)],|r|r.get(0)).optional()?.ok_or_else(unauthorized)
    }
    fn worker(&self,headers:&HeaderMap)->Result<(),ApiError> {
        let found:bool=self.db.lock().unwrap().query_row("SELECT EXISTS(SELECT 1 FROM workers WHERE token_hash=?1 AND disabled=0)",[digest(token(headers)?)],|r|r.get(0))?;
        if found {Ok(())} else {Err(unauthorized())}
    }
    fn audit(&self,actor:&str,event:&str,target:&str)->Result<(),ApiError> {
        self.db.lock().unwrap().execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,?3,?4)",params![now(),actor,event,target])?;
        Ok(())
    }
    fn rate(&self,key:&str,maximum:u32)->Result<(),ApiError> {
        let mut limits=self.limits.lock().unwrap();
        let time=now(); limits.retain(|_,v|time.saturating_sub(v.0)<300);
        if limits.len()>10000 { return Err(ApiError(StatusCode::TOO_MANY_REQUESTS,"Retry later")); }
        let value=limits.entry(key.into()).or_insert((time,0));
        if time.saturating_sub(value.0)>=60 { *value=(time,0); }
        value.1+=1;
        if value.1>maximum {Err(ApiError(StatusCode::TOO_MANY_REQUESTS,"Retry later"))} else {Ok(())}
    }
}
fn write_secret(path:&FsPath,value:&str)->anyhow::Result<()> {
    use std::io::Write;
    let mut options=std::fs::OpenOptions::new(); options.write(true).create_new(true);
    #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
    let mut file=options.open(path)?; file.write_all(value.as_bytes())?;file.sync_all()?;Ok(())
}

pub fn router(store:Shared)->Router {
    Router::new()
        .route("/",get(||async{Html(include_str!("../web/index.html"))}))
        .route("/app.js",get(||async{([(header::CONTENT_TYPE,"text/javascript; charset=utf-8")],include_str!("../web/app.js"))}))
        .route("/style.css",get(||async{([(header::CONTENT_TYPE,"text/css")],include_str!("../web/style.css"))}))
        .route("/health",get(||async{Json(json!({"status":"ok","product":PRODUCT}))}))
        .route("/swan-support-mark.svg",get(||async{([(header::CONTENT_TYPE,"image/svg+xml")],include_str!("../../../branding/swan-support-mark.svg"))}))
        .route("/api/v1/default-branding",get(||async{Json(Branding::default())}))
        .route("/api/v1/status",get(status))
        .route("/api/v1/setup",post(setup))
        .route("/api/v1/login",post(login))
        .route("/api/v1/logout",post(logout))
        .route("/api/v1/profile",get(profile).put(save_profile))
        .route("/api/v1/network/check",post(network_check))
        .route("/api/v1/users",get(users).post(create_user))
        .route("/api/v1/users/{id}/disable",post(disable_user))
        .route("/api/v1/enroll",post(enroll))
        .route("/api/v1/devices",get(devices))
        .route("/api/v1/devices/{id}/state",put(device_state))
        .route("/api/v1/device/consent",put(consent))
        .route("/api/v1/device/status",get(device_status))
        .route("/api/v1/groups/{group}/permissions",get(group_permissions).put(set_group_permissions))
        .route("/api/v1/groups/{group}/users/{user}",put(group_access).delete(revoke_group_access))
        .route("/api/v1/grants",post(grant))
        .route("/api/v1/grants/claim",post(claim))
        .route("/api/v1/grants/{id}/renew",post(renew))
        .route("/api/v1/grants/{id}/close",post(close_grant))
        .route("/api/v1/audit",get(audit))
        .route("/api/v1/sessions",get(session_history))
        .route("/api/v1/releases",get(releases).post(import_release))
        .route("/api/v1/releases/{id}/approve",post(approve_release).delete(withdraw_release))
        .route("/api/v1/device/update",get(device_update))
        .route("/api/v1/user/update",get(technician_update))
        .route("/api/v1/workers",get(workers).post(create_worker))
        .route("/api/v1/workers/{id}",axum::routing::delete(revoke_worker))
        .route("/api/v1/builds",get(builds).post(create_build))
        .route("/api/v1/worker/claim",post(claim_build))
        .route("/api/v1/worker/builds/{id}",put(finish_build))
        .route("/api/v1/worker/builds/{id}/artifact",put(upload_artifact).layer(DefaultBodyLimit::max(512*1024*1024)))
        .route("/api/v1/downloads/{id}",get(download_artifact))
        .layer(DefaultBodyLimit::max(1024*1024))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(header::HeaderName::from_static("x-content-type-options"),header::HeaderValue::from_static("nosniff")))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(header::HeaderName::from_static("content-security-policy"),header::HeaderValue::from_static("default-src 'self'; img-src 'self' data:; script-src 'self'; style-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'")))
        .layer(tower_http::set_header::SetResponseHeaderLayer::overriding(header::CACHE_CONTROL,header::HeaderValue::from_static("no-store")))
        .with_state(store)
}
async fn status(State(s):State<Shared>)->ApiResult {
    let configured=s.db.lock().unwrap().query_row("SELECT EXISTS(SELECT 1 FROM profile)",[],|r|r.get::<_,bool>(0))?;
    Ok(Json(json!({"configured":configured,"profile_public_key":STANDARD.encode(s.key.verifying_key().as_bytes())})))
}
async fn network_check(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    let actor=s.user(&headers,true)?;s.rate("network-check",10)?;
    let profile=s.profile()?;
    let result=crate::network::check(&profile,&s.key.verifying_key()).await?;
    s.audit(&actor,"network.checked",&profile.company_id)?;Ok(Json(result))
}
#[derive(Deserialize)]
struct Setup { profile:CompanyProfile, username:String,password:String,totp_secret:String,totp_code:String }
async fn setup(State(s):State<Shared>,headers:HeaderMap,Json(mut input):Json<Setup>)->ApiResult {
    s.rate("setup",5)?;
    if digest(token(&headers)?)!=s.setup_hash {return Err(denied());}
    validate_username(&input.username)?;
    let step=totp_step(&input.totp_secret,&input.totp_code,now()).map_err(|_|bad("Invalid authenticator code"))?;
    let hash=password_hash(&input.password).map_err(|_|bad("Password must contain 14–256 characters"))?;
    input.profile.company_id=Uuid::new_v4().to_string(); input.profile.revision=1;
    input.profile.issued_at=now();input.profile.expires_at=now()+86400*30;
    input.profile.validate(&input.profile.company_id,0,now()).map_err(|_|bad("Invalid company profile"))?;
    let mut db=s.db.lock().unwrap();let transaction=db.transaction()?;
    let exists:bool=transaction.query_row("SELECT EXISTS(SELECT 1 FROM profile)",[],|r|r.get(0))?;
    if exists {return Err(ApiError(StatusCode::CONFLICT,"Already configured"));}
    let user=Uuid::new_v4().to_string();
    transaction.execute("INSERT INTO profile(id,body) VALUES(1,?1)",[serde_json::to_string(&input.profile)?])?;
    transaction.execute("INSERT INTO users(id,username,password_hash,totp_secret,last_totp_step,role) VALUES(?1,?2,?3,?4,?5,'admin')",params![user,input.username,hash,input.totp_secret,step])?;
    transaction.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,'company.setup',?3)",params![now(),user,input.profile.company_id])?;
    transaction.commit()?;
    Ok(Json(json!({"company_id":input.profile.company_id})))
}
fn validate_username(name:&str)->Result<(),ApiError> {
    if name.len()<3 || name.len()>64 || !name.bytes().all(|b|b.is_ascii_alphanumeric() || b"._-@".contains(&b)) {Err(bad("Invalid username"))} else {Ok(())}
}
#[derive(Deserialize)]
struct Login { username:String,password:String,totp_code:String }
async fn login(State(s):State<Shared>,Json(input):Json<Login>)->ApiResult {
    s.rate("login-global",60)?;s.rate(&format!("login:{}",input.username),5)?;
    validate_username(&input.username)?;
    let record:Option<(String,String,String,u64)>=s.db.lock().unwrap().query_row("SELECT id,password_hash,totp_secret,last_totp_step FROM users WHERE username=?1 AND disabled=0",[&input.username],|r|Ok((r.get(0)?,r.get(1)?,r.get(2)?,r.get(3)?))).optional()?;
    let (id,hash,secret,last)=record.ok_or_else(unauthorized)?;
    let parsed=PasswordHash::new(&hash).map_err(|_|unauthorized())?;
    Argon2::default().verify_password(input.password.as_bytes(),&parsed).map_err(|_|unauthorized())?;
    let step=totp_step(&secret,&input.totp_code,now()).map_err(|_|unauthorized())?;
    if step<=last {return Err(unauthorized());}
    let value=random_token();
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    if tx.execute("UPDATE users SET last_totp_step=?1 WHERE id=?2 AND last_totp_step<?1",params![step,id])?!=1 {return Err(unauthorized());}
    tx.execute("DELETE FROM sessions WHERE expires_at<=?1",[now()])?;
    tx.execute("INSERT INTO sessions(token_hash,user_id,expires_at) VALUES(?1,?2,?3)",params![digest(&value),id,now()+3600])?;
    tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,'user.login',?2)",params![now(),id])?;tx.commit()?;
    Ok(Json(json!({"token":value,"expires_at":now()+3600,"user_id":id})))
}
async fn logout(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    let user=s.user(&headers,false)?;
    s.db.lock().unwrap().execute("DELETE FROM sessions WHERE token_hash=?1",[digest(token(&headers)?)])?;
    s.audit(&user,"user.logout",&user)?;Ok(Json(json!({"ok":true})))
}
async fn profile(State(s):State<Shared>)->ApiResult {
    let mut profile=s.profile().map_err(|_|ApiError(StatusCode::SERVICE_UNAVAILABLE,"Company not configured"))?;
    // Renew the signed policy lifetime without changing the administrator's revision.
    profile.issued_at=now();profile.expires_at=now()+7*86400;
    Ok(Json(serde_json::to_value(SignedEnvelope::sign(&profile,&s.key)?)?))
}
async fn save_profile(State(s):State<Shared>,headers:HeaderMap,Json(mut input):Json<CompanyProfile>)->ApiResult {
    let user=s.user(&headers,true)?;
    let current=s.profile()?;
    input.company_id=current.company_id.clone();input.revision=current.revision+1;
    input.issued_at=now();input.expires_at=now()+30*86400;
    input.validate(&current.company_id,current.revision,now()).map_err(|_|bad("Invalid profile"))?;
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    let raw:String=tx.query_row("SELECT body FROM profile WHERE id=1",[],|r|r.get(0))?;
    if serde_json::from_str::<CompanyProfile>(&raw)?.revision!=current.revision {return Err(ApiError(StatusCode::CONFLICT,"Profile changed; reload"));}
    tx.execute("UPDATE profile SET body=?1 WHERE id=1",[serde_json::to_string(&input)?])?;
    if !input.allow_unattended {close_unattended_grants(&tx,None)?;}
    tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,'profile.updated',?3)",params![now(),user,input.revision.to_string()])?;
    tx.commit()?;Ok(Json(json!({"revision":input.revision})))
}
#[derive(Deserialize)]
struct CreateUser { username:String,password:String,role:String }
async fn create_user(State(s):State<Shared>,headers:HeaderMap,Json(input):Json<CreateUser>)->ApiResult {
    let actor=s.user(&headers,true)?;validate_username(&input.username)?;
    if !["admin","technician"].contains(&input.role.as_str()) {return Err(bad("Invalid role"));}
    let hash=password_hash(&input.password).map_err(|_|bad("Password must contain 14–256 characters"))?;
    let secret=Secret::generate_secret().to_encoded().to_string();let id=Uuid::new_v4().to_string();
    s.db.lock().unwrap().execute("INSERT INTO users(id,username,password_hash,totp_secret,role) VALUES(?1,?2,?3,?4,?5)",params![id,input.username,hash,secret,input.role])?;
    s.audit(&actor,"user.created",&id)?;
    Ok(Json(json!({"id":id,"totp_secret":secret,"message":"Transfer authenticator secret privately; it will not be returned again."})))
}
async fn users(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    s.user(&headers,true)?;let db=s.db.lock().unwrap();let mut statement=db.prepare("SELECT id,username,role,disabled FROM users ORDER BY username")?;
    let rows=statement.query_map([],|r|Ok(json!({"id":r.get::<_,String>(0)?,"username":r.get::<_,String>(1)?,"role":r.get::<_,String>(2)?,"disabled":r.get::<_,bool>(3)?})))?.collect::<Result<Vec<_>,_>>()?;
    Ok(Json(json!(rows)))
}
async fn disable_user(State(s):State<Shared>,headers:HeaderMap,Path(id):Path<String>)->ApiResult {
    let actor=s.user(&headers,true)?;if actor==id {return Err(bad("Cannot disable your own administrator account"));}
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    tx.execute("UPDATE users SET disabled=1 WHERE id=?1",[&id])?;
    tx.execute("DELETE FROM sessions WHERE user_id=?1",[&id])?;
    tx.execute("UPDATE grants SET closed=1,lease_until=0 WHERE user_id=?1",[&id])?;
    tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,'user.disabled',?3)",params![now(),actor,id])?;
    tx.commit()?;Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
struct Enrollment { name:String,rustdesk_id:String,unattended_consent:bool }
async fn enroll(State(s):State<Shared>,Json(input):Json<Enrollment>)->ApiResult {
    s.rate("enrollment",10)?;
    if input.name.trim().is_empty() || input.name.len()>100 || input.rustdesk_id.len()>20 || input.rustdesk_id.len()<6 || !input.rustdesk_id.bytes().all(|b|b.is_ascii_digit()) {return Err(bad("Invalid device identity"));}
    let profile=s.profile()?;
    if input.unattended_consent && !profile.allow_unattended {return Err(denied());}
    let id=Uuid::new_v4().to_string();let credential=random_token();
    s.db.lock().unwrap().execute("INSERT INTO devices(id,name,rustdesk_id,token_hash,unattended,state,group_id) VALUES(?1,?2,?3,?4,?5,'pending','default')",params![id,input.name,input.rustdesk_id,digest(&credential),input.unattended_consent])?;
    s.audit(&id,"device.enrolled",&id)?;
    Ok(Json(json!({"device_id":id,"device_token":credential,"state":"pending","company_id":profile.company_id})))
}
async fn devices(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    let user=s.user(&headers,false)?;let db=s.db.lock().unwrap();
    let mut statement=db.prepare("SELECT d.id,d.name,d.rustdesk_id,d.state,d.group_id,d.unattended FROM devices d WHERE EXISTS(SELECT 1 FROM users u WHERE u.id=?1 AND u.role='admin') OR (d.state='approved' AND EXISTS(SELECT 1 FROM group_access g WHERE g.group_id=d.group_id AND g.user_id=?1)) ORDER BY d.name")?;
    let rows=statement.query_map([user],|r|Ok(json!({"id":r.get::<_,String>(0)?,"name":r.get::<_,String>(1)?,"rustdesk_id":r.get::<_,String>(2)?,"state":r.get::<_,String>(3)?,"group":r.get::<_,String>(4)?,"unattended":r.get::<_,bool>(5)?})))?.collect::<Result<Vec<_>,_>>()?;
    Ok(Json(json!(rows)))
}
#[derive(Deserialize)]
struct DeviceState { state:String,group:String }
async fn device_state(State(s):State<Shared>,headers:HeaderMap,Path(id):Path<String>,Json(input):Json<DeviceState>)->ApiResult {
    let actor=s.user(&headers,true)?;
    if !["approved","revoked","pending"].contains(&input.state.as_str()) || input.group.is_empty() || input.group.len()>64 {return Err(bad("Invalid state or group"));}
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    let previous_group:Option<String>=tx.query_row("SELECT group_id FROM devices WHERE id=?1",[&id],|row|row.get(0)).optional()?;
    let previous_group=previous_group.ok_or(ApiError(StatusCode::NOT_FOUND,"Device not found"))?;
    tx.execute("UPDATE devices SET state=?1,group_id=?2 WHERE id=?3",params![input.state,input.group,id])?;
    if input.state!="approved" || input.group!=previous_group {
        tx.execute("UPDATE grants SET closed=1,lease_until=0 WHERE device_id=?1",[&id])?;
    }
    tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,?3,?4)",params![now(),actor,format!("device.{}",input.state),id])?;
    tx.commit()?;Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
struct Consent { unattended:bool,revision:u64 }
fn close_unattended_grants(db:&Connection,device:Option<&str>)->Result<(),ApiError> {
    let ids={
        let mut statement=db.prepare("SELECT id,body FROM grants WHERE closed=0 AND (?1 IS NULL OR device_id=?1)")?;
        let rows=statement.query_map([device],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?)))?;
        let mut ids=Vec::new();
        for row in rows {let (id,body)=row?;let grant:SessionGrant=serde_json::from_str(&body)?;if grant.unattended {ids.push(id);}}
        ids
    };
    for id in ids {db.execute("UPDATE grants SET closed=1,lease_until=0 WHERE id=?1",[id])?;}
    Ok(())
}
async fn device_status(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    // Pending devices may inspect only their own identity, never session APIs.
    let device:String=s.db.lock().unwrap().query_row("SELECT id FROM devices WHERE token_hash=?1 AND state IN ('pending','approved')",[digest(token(&headers)?)],|row|row.get(0)).optional()?.ok_or_else(unauthorized)?;
    let company=s.profile()?.company_id;
    let value=s.db.lock().unwrap().query_row("SELECT rustdesk_id,state FROM devices WHERE id=?1",[&device],|row|Ok(json!({"company_id":company,"device_id":device,"rustdesk_id":row.get::<_,String>(0)?,"state":row.get::<_,String>(1)?})))?;
    Ok(Json(value))
}
async fn consent(State(s):State<Shared>,headers:HeaderMap,Json(input):Json<Consent>)->ApiResult {
    let device=s.device(&headers)?;
    if input.unattended && !s.profile()?.allow_unattended {return Err(denied());}
    if input.revision>i64::MAX as u64 {return Err(bad("Invalid consent revision"));}
    {
        let mut db=s.db.lock().unwrap();let transaction=db.transaction()?;
        let revision:u64=transaction.query_row("SELECT revision FROM device_consent_revisions WHERE device_id=?1",[&device],|row|row.get(0)).optional()?.unwrap_or(0);
        let enabled:bool=transaction.query_row("SELECT unattended FROM devices WHERE id=?1",[&device],|row|row.get(0))?;
        // Equal revisions are idempotent; revocation wins conflicting requests.
        if input.revision<revision || (input.revision==revision && input.unattended && !enabled) {return Err(ApiError(StatusCode::CONFLICT,"Consent request superseded"));}
        if input.revision==revision && input.unattended==enabled {return Ok(Json(json!({"ok":true})));}
        transaction.execute("UPDATE devices SET unattended=?1 WHERE id=?2",params![input.unattended,device])?;
        if !input.unattended {
            close_unattended_grants(&transaction,Some(&device))?;
        }
        transaction.execute("INSERT INTO device_consent_revisions(device_id,revision) VALUES(?1,?2) ON CONFLICT(device_id) DO UPDATE SET revision=excluded.revision",params![device,input.revision])?;
        transaction.commit()?;
    }
    s.audit(&device,"device.consent_changed",&device)?;Ok(Json(json!({"ok":true})))
}
async fn group_access(State(s):State<Shared>,headers:HeaderMap,Path((group,user)):Path<(String,String)>)->ApiResult {
    let actor=s.user(&headers,true)?;if group.len()>64 || group.is_empty(){return Err(bad("Invalid group"));}
    s.db.lock().unwrap().execute("INSERT OR IGNORE INTO group_access(group_id,user_id) VALUES(?1,?2)",params![group,user])?;
    s.audit(&actor,"group.access_granted",&user)?;Ok(Json(json!({"ok":true})))
}
#[derive(Deserialize)]
struct GrantRequest { device_id:String, proof_public_key:String, unattended:bool }
fn device_permissions(db:&Connection,device:&str)->Result<SessionPermissions,ApiError> {
    let raw:Option<String>=db.query_row("SELECT p.body FROM devices d LEFT JOIN group_permissions p ON p.group_id=d.group_id WHERE d.id=?1",[device],|row|row.get(0))?;
    Ok(match raw {Some(value)=>serde_json::from_str(&value)?,None=>SessionPermissions::support_default()})
}
async fn group_permissions(State(s):State<Shared>,headers:HeaderMap,Path(group):Path<String>)->ApiResult {
    s.user(&headers,true)?;
    let raw:Option<String>=s.db.lock().unwrap().query_row("SELECT body FROM group_permissions WHERE group_id=?1",[group],|row|row.get(0)).optional()?;
    Ok(Json(match raw {Some(value)=>serde_json::from_str(&value)?,None=>serde_json::to_value(SessionPermissions::support_default())?}))
}
async fn set_group_permissions(State(s):State<Shared>,headers:HeaderMap,Path(group):Path<String>,Json(policy):Json<SessionPermissions>)->ApiResult {
    let actor=s.user(&headers,true)?;if group.is_empty() || group.len()>64 {return Err(bad("Invalid group"));}
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    tx.execute("INSERT INTO group_permissions(group_id,body) VALUES(?1,?2) ON CONFLICT(group_id) DO UPDATE SET body=excluded.body",params![group,serde_json::to_string(&policy)?])?;
    let invalid={
        let mut statement=tx.prepare("SELECT g.id,g.body FROM grants g JOIN devices d ON d.id=g.device_id WHERE d.group_id=?1 AND g.closed=0")?;
        let rows=statement.query_map([&group],|row|Ok((row.get::<_,String>(0)?,row.get::<_,String>(1)?)))?;
        let mut ids=Vec::new();
        for row in rows {let (id,body)=row?;let grant:SessionGrant=serde_json::from_str(&body)?;if !policy.allows(&grant.permissions){ids.push(id);}}
        ids
    };
    for id in invalid {tx.execute("UPDATE grants SET closed=1,lease_until=0 WHERE id=?1",[id])?;}
    tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,'group.permissions_changed',?3)",params![now(),actor,group])?;
    tx.commit()?;Ok(Json(json!({"ok":true})))
}
async fn grant(State(s):State<Shared>,headers:HeaderMap,Json(input):Json<GrantRequest>)->ApiResult {
    let user=s.user(&headers,false)?;s.rate(&format!("grant:{user}"),30)?;
    public_key(&input.proof_public_key).map_err(|_|bad("Invalid proof key"))?;
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    let raw:String=tx.query_row("SELECT body FROM profile WHERE id=1",[],|row|row.get(0))?;
    let profile:CompanyProfile=serde_json::from_str(&raw)?;
    let peer:Option<String>=tx.query_row("SELECT d.rustdesk_id FROM devices d JOIN users u ON u.id=?2 WHERE d.id=?1 AND d.state='approved' AND u.disabled=0 AND (?3=0 OR d.unattended=1) AND (u.role='admin' OR EXISTS(SELECT 1 FROM group_access g WHERE g.group_id=d.group_id AND g.user_id=u.id))",params![input.device_id,user,input.unattended],|r|r.get(0)).optional()?;
    let peer=peer.ok_or_else(denied)?;
    if input.unattended && !profile.allow_unattended {return Err(denied());}
    let grant=SessionGrant { schema:SCHEMA,company_id:profile.company_id,grant_id:Uuid::new_v4().to_string(),technician_id:user.clone(),device_id:input.device_id.clone(),rustdesk_id:peer,proof_public_key:input.proof_public_key,unattended:input.unattended,permissions:device_permissions(&tx,&input.device_id)?,issued_at:now(),expires_at:now()+60 };
    tx.execute("INSERT INTO grants(id,user_id,device_id,body,expires_at) VALUES(?1,?2,?3,?4,?5)",params![grant.grant_id,user,input.device_id,serde_json::to_string(&grant)?,grant.expires_at])?;
    tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,'session.requested',?3)",params![now(),user,grant.grant_id])?;
    tx.commit()?;
    Ok(Json(serde_json::to_value(SignedEnvelope::sign(&grant,&s.key)?)?))
}
#[derive(Deserialize)]
struct Claim { grant:SignedEnvelope }
fn active_grant(db:&Connection,id:&str,device:&str)->Result<bool,ApiError> {
    Ok(db.query_row("SELECT EXISTS(SELECT 1 FROM grants g JOIN devices d ON d.id=g.device_id JOIN users u ON u.id=g.user_id WHERE g.id=?1 AND g.device_id=?2 AND d.state='approved' AND u.disabled=0 AND g.closed=0 AND (u.role='admin' OR EXISTS(SELECT 1 FROM group_access a WHERE a.user_id=u.id AND a.group_id=d.group_id)))",params![id,device],|r|r.get(0))?)
}
async fn claim(State(s):State<Shared>,headers:HeaderMap,Json(input):Json<Claim>)->ApiResult {
    let device=s.device(&headers)?;let profile=s.profile()?;
    let grant:SessionGrant=input.grant.verify(&s.key.verifying_key()).map_err(|_|denied())?;
    let peer:String=s.db.lock().unwrap().query_row("SELECT rustdesk_id FROM devices WHERE id=?1",[&device],|r|r.get(0))?;
    grant.validate(&profile.company_id,&device,&peer,now()).map_err(|_|denied())?;
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    if !active_grant(&tx,&grant.grant_id,&device)? {return Err(denied());}
    if !device_permissions(&tx,&device)?.allows(&grant.permissions) {return Err(denied());}
    let consent:bool=tx.query_row("SELECT unattended FROM devices WHERE id=?1",[&device],|r|r.get(0))?;
    if grant.unattended && !(consent && profile.allow_unattended) {return Err(denied());}
    if tx.execute("UPDATE grants SET claimed=1,lease_until=?1 WHERE id=?2 AND claimed=0 AND expires_at>?3",params![now()+300,grant.grant_id,now()])?!=1 {return Err(denied());}
    tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,'session.claimed',?3)",params![now(),device,grant.grant_id])?;tx.commit()?;
    Ok(Json(json!({"lease_until":now()+300,"unattended":grant.unattended})))
}
async fn renew(State(s):State<Shared>,headers:HeaderMap,Path(id):Path<String>)->ApiResult {
    let device=s.device(&headers)?;let profile=s.profile()?;
    let db=s.db.lock().unwrap();
    if !active_grant(&db,&id,&device)? {return Err(denied());}
    let raw:String=db.query_row("SELECT body FROM grants WHERE id=?1",[&id],|r|r.get(0))?;
    let grant:SessionGrant=serde_json::from_str(&raw)?;
    if !device_permissions(&db,&device)?.allows(&grant.permissions) {return Err(denied());}
    let consent:bool=db.query_row("SELECT unattended FROM devices WHERE id=?1",[&device],|r|r.get(0))?;
    if grant.unattended && !(consent && profile.allow_unattended){return Err(denied());}
    if db.execute("UPDATE grants SET lease_until=?1 WHERE id=?2 AND claimed=1 AND lease_until>?3 AND closed=0",params![now()+300,id,now()])?!=1 {return Err(denied());}
    Ok(Json(json!({"lease_until":now()+300})))
}
async fn close_grant(State(s):State<Shared>,headers:HeaderMap,Path(id):Path<String>)->ApiResult {
    let device=s.device(&headers)?;
    s.db.lock().unwrap().execute("UPDATE grants SET closed=1 WHERE id=?1 AND device_id=?2",params![id,device])?;
    s.audit(&device,"session.closed",&id)?;Ok(Json(json!({"ok":true})))
}
async fn audit(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    s.user(&headers,true)?;let db=s.db.lock().unwrap();let mut statement=db.prepare("SELECT at,actor,event,target FROM audit ORDER BY id DESC LIMIT 500")?;
    let rows=statement.query_map([],|r|Ok(json!({"at":r.get::<_,u64>(0)?,"actor":r.get::<_,String>(1)?,"event":r.get::<_,String>(2)?,"target":r.get::<_,String>(3)?})))?.collect::<Result<Vec<_>,_>>()?;Ok(Json(json!(rows)))
}
async fn session_history(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    let user=s.user(&headers,false)?;let db=s.db.lock().unwrap();
    let administrator:bool=db.query_row("SELECT role='admin' FROM users WHERE id=?1",[&user],|r|r.get(0))?;
    let mut statement=db.prepare("SELECT g.id,g.body,g.claimed,g.closed,g.lease_until,d.name FROM grants g JOIN devices d ON d.id=g.device_id WHERE (?1=1 OR g.user_id=?2) ORDER BY g.rowid DESC LIMIT 200")?;
    let rows=statement.query_map(params![administrator,user],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,r.get::<_,bool>(2)?,r.get::<_,bool>(3)?,r.get::<_,u64>(4)?,r.get::<_,String>(5)?)))?.collect::<Result<Vec<_>,_>>()?;
    let mut result=Vec::new();
    for(id,body,claimed,closed,lease,name)in rows {let grant:SessionGrant=serde_json::from_str(&body)?;result.push(json!({"id":id,"technician_id":grant.technician_id,"device_id":grant.device_id,"device_name":name,"requested_at":grant.issued_at,"unattended":grant.unattended,"claimed":claimed,"closed":closed,"lease_until":lease}));}
    Ok(Json(json!(result)))
}
#[derive(Deserialize)]
struct ReleaseImport { envelope:SignedEnvelope, public_key:String }
async fn import_release(State(s):State<Shared>,headers:HeaderMap,Json(input):Json<ReleaseImport>)->ApiResult {
    let user=s.user(&headers,true)?;
    // Project trust is configured by the operator, never accepted from the request.
    let trusted=std::env::var("SWAN_RELEASE_PUBLIC_KEY").map_err(|_|bad("Project release trust key not configured"))?;
    if input.public_key!=trusted {return Err(denied());}
    let release:Release=input.envelope.verify(&public_key(&trusted)?).map_err(|_|denied())?;
    release.validate(&release.edition,0,now()).map_err(|_|bad("Invalid release"))?;
    let id=format!("{}-{:?}-{}",release.sequence,release.edition,release.channel);
    s.db.lock().unwrap().execute("INSERT INTO releases(id,body,envelope,approved) VALUES(?1,?2,?3,0)",params![id,serde_json::to_string(&release)?,serde_json::to_string(&input.envelope)?])?;
    s.audit(&user,"release.imported",&id)?;Ok(Json(json!({"id":id})))
}
async fn approve_release(State(s):State<Shared>,headers:HeaderMap,Path(id):Path<String>)->ApiResult {
    let user=s.user(&headers,true)?;let db=s.db.lock().unwrap();
    let raw:String=db.query_row("SELECT body FROM releases WHERE id=?1",[&id],|r|r.get(0))?;
    let release:Release=serde_json::from_str(&raw)?;
    release.validate(&release.edition,0,now()).map_err(|_|bad("Release expired"))?;
    db.execute("UPDATE releases SET approved=1 WHERE id=?1",[&id])?;drop(db);
    s.audit(&user,"release.approved",&id)?;Ok(Json(json!({"ok":true})))
}
async fn releases(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    s.user(&headers,true)?;list_json(&s,"SELECT id,body,approved FROM releases",true)
}
fn list_json(s:&Store,sql:&str,approved:bool)->ApiResult {
    let db=s.db.lock().unwrap();let mut stmt=db.prepare(sql)?;
    let rows=stmt.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?,if approved{r.get::<_,bool>(2)?}else{false})))?.collect::<Result<Vec<_>,_>>()?;
    let mut values=Vec::new();
    for (id,raw,approved) in rows {values.push(json!({"id":id,"data":serde_json::from_str::<Value>(&raw)?,"approved":approved}));}
    Ok(Json(json!(values)))
}
#[derive(Default,Deserialize)]
#[serde(deny_unknown_fields)]
struct UpdateQuery {format:Option<String>}
async fn device_update(State(s):State<Shared>,headers:HeaderMap,Query(query):Query<UpdateQuery>)->ApiResult {
    let device=s.device(&headers)?;select_update(&s,&device,Edition::Customer,query.format.as_deref())
}
async fn technician_update(State(s):State<Shared>,headers:HeaderMap,Query(query):Query<UpdateQuery>)->ApiResult {
    let user=s.user(&headers,false)?;select_update(&s,&user,Edition::Technician,query.format.as_deref())
}
fn select_update(s:&Store,device:&str,edition:Edition,format:Option<&str>)->ApiResult {
    if format.map(|format|!["exe","msi"].contains(&format)).unwrap_or(false){return Err(bad("Invalid installer format"));}
    let profile=s.profile()?;
    let cohort=u8::from_str_radix(&digest(&device)[..2],16).map_err(|_|bad("Invalid device"))? as u32 * 100 / 256;
    if profile.updates_paused || cohort>=profile.rollout_percent as u32 || !maintenance_open(profile.maintenance_start_utc,profile.maintenance_end_utc,now()) {return Ok(Json(Value::Null));}
    let db=s.db.lock().unwrap();let mut stmt=db.prepare("SELECT body,envelope FROM releases WHERE approved=1")?;
    let rows=stmt.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<Result<Vec<_>,_>>()?;
    let mut best:Option<(u64,Value)>=None;
    for (body,envelope) in rows {let release:Release=serde_json::from_str(&body)?;
        if release.edition==edition && format.map(|format|release.format==format).unwrap_or(true) && release.channel==profile.update_channel && release.expires_at>now() && best.as_ref().map(|b|b.0<release.sequence).unwrap_or(true) {best=Some((release.sequence,serde_json::from_str(&envelope)?));}}
    Ok(Json(best.map(|v|v.1).unwrap_or(Value::Null)))
}
async fn create_worker(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    let user=s.user(&headers,true)?;let credential=random_token();let id=Uuid::new_v4().to_string();
    s.db.lock().unwrap().execute("INSERT INTO workers(id,token_hash) VALUES(?1,?2)",params![id,digest(&credential)])?;
    s.audit(&user,"worker.created",&id)?;Ok(Json(json!({"worker_id":id,"worker_token":credential})))
}
async fn workers(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    s.user(&headers,true)?;let db=s.db.lock().unwrap();
    let mut statement=db.prepare("SELECT id,disabled FROM workers ORDER BY rowid DESC")?;
    let rows=statement.query_map([],|r|Ok(json!({"worker_id":r.get::<_,String>(0)?,"disabled":r.get::<_,bool>(1)?})))?.collect::<Result<Vec<_>,_>>()?;
    Ok(Json(json!(rows)))
}
async fn revoke_worker(State(s):State<Shared>,headers:HeaderMap,Path(id):Path<String>)->ApiResult {
    let actor=s.user(&headers,true)?;let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    let worker:Option<String>=tx.query_row("SELECT token_hash FROM workers WHERE id=?1",[&id],|r|r.get(0)).optional()?;
    let worker=worker.ok_or(ApiError(StatusCode::NOT_FOUND,"Unknown worker"))?;
    tx.execute("UPDATE workers SET disabled=1 WHERE id=?1",[&id])?;
    // A revoked claim must never become retryable through the expiry sweep.
    let cancelled=tx.execute("UPDATE builds SET state='failed',result=?1 WHERE worker=?2 AND state IN ('running','uploaded')",params![json!({"artifact_url":"","sha256":"","log":"Installer worker revoked"}).to_string(),worker])?;
    tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,'worker.revoked',?3)",params![now(),actor,id])?;
    tx.commit()?;Ok(Json(json!({"ok":true,"cancelled_builds":cancelled})))
}
#[derive(Deserialize)]
struct BuildRequest { release_id:String }
async fn create_build(State(s):State<Shared>,headers:HeaderMap,Json(input):Json<BuildRequest>)->ApiResult {
    let actor=s.user(&headers,true)?;let mut profile=s.profile()?;
    profile.issued_at=now();profile.expires_at=now()+7*86400;
    let db=s.db.lock().unwrap();
    let envelope:String=db.query_row("SELECT envelope FROM releases WHERE id=?1 AND approved=1",[&input.release_id],|r|r.get(0))?;
    let id=Uuid::new_v4().to_string();let body=json!({"release":serde_json::from_str::<Value>(&envelope)?,"profile":SignedEnvelope::sign(&profile,&s.key)?,"profile_public_key":STANDARD.encode(s.key.verifying_key().as_bytes()),"release_public_key":std::env::var("SWAN_RELEASE_PUBLIC_KEY").map_err(|_|bad("Release key not configured"))?});
    db.execute("INSERT INTO builds(id,body,state) VALUES(?1,?2,'queued')",params![id,body.to_string()])?;drop(db);s.audit(&actor,"build.queued",&id)?;Ok(Json(json!({"id":id})))
}
async fn builds(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    s.user(&headers,true)?;let db=s.db.lock().unwrap();let mut stmt=db.prepare("SELECT id,state,result FROM builds ORDER BY rowid DESC LIMIT 100")?;
    let rows=stmt.query_map([],|r|Ok(json!({"id":r.get::<_,String>(0)?,"state":r.get::<_,String>(1)?,"result":r.get::<_,String>(2)?})))?.collect::<Result<Vec<_>,_>>()?;Ok(Json(json!(rows)))
}
async fn claim_build(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    s.worker(&headers)?;let worker=digest(token(&headers)?);
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    // Recheck under the claim transaction: revocation may follow the initial
    // authentication check before this request acquires the database lock.
    let enabled:bool=tx.query_row("SELECT EXISTS(SELECT 1 FROM workers WHERE token_hash=?1 AND disabled=0)",[&worker],|r|r.get(0))?;
    if !enabled{return Err(unauthorized());}
    // Expired claims are retryable; worker output is named by immutable job ID.
    tx.execute("UPDATE builds SET state='queued',worker='' WHERE state IN ('running','uploaded') AND claimed_at<?1",[now().saturating_sub(3600)])?;
    let row:Option<(String,String)>=tx.query_row("SELECT id,body FROM builds WHERE state='queued' ORDER BY rowid LIMIT 1",[],|r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    if let Some((id,body))=row {tx.execute("UPDATE builds SET state='running',worker=?1,claimed_at=?2 WHERE id=?3",params![worker,now(),id])?;tx.commit()?;Ok(Json(json!({"id":id,"job":serde_json::from_str::<Value>(&body)?})))} else {Ok(Json(Value::Null))}
}
#[derive(Deserialize)]
struct BuildResult { success:bool, artifact_url:String,sha256:String,log:String }

#[cfg(test)]
mod worker_completion_tests {
    use super::*;
    #[tokio::test]
    async fn administrator_revocation_cancels_claims_and_hides_credentials() {
        let directory=std::env::temp_dir().join(format!("swan-worker-revocation-{}",random_token()));
        let store=Arc::new(Store::open(&directory).unwrap());
        let admin=random_token();let technician=random_token();let credential=random_token();let worker=digest(&credential);
        {
            let db=store.db.lock().unwrap();
            for (id,role,token) in [("admin","admin",&admin),("technician","technician",&technician)] {
                db.execute("INSERT INTO users(id,username,password_hash,totp_secret,role) VALUES(?1,?1,'unused','unused',?2)",params![id,role]).unwrap();
                db.execute("INSERT INTO sessions VALUES(?1,?2,?3)",params![digest(token),id,now()+3600]).unwrap();
            }
            db.execute("INSERT INTO workers(id,token_hash) VALUES('worker',?1)",[&worker]).unwrap();
            for state in ["running","uploaded","completed"] {
                db.execute("INSERT INTO builds(id,body,state,worker,claimed_at) VALUES(?1,'{}',?1,?2,0)",params![state,worker]).unwrap();
            }
            db.execute("INSERT INTO builds(id,body,state,worker) VALUES('other','{}','running','other-worker')",[]).unwrap();
        }
        let headers=|token:&str| {let mut h=HeaderMap::new();h.insert(header::AUTHORIZATION,format!("Bearer {token}").parse().unwrap());h};
        assert!(workers(State(store.clone()),HeaderMap::new()).await.is_err());
        assert!(revoke_worker(State(store.clone()),headers(&technician),Path("worker".into())).await.is_err());
        assert!(store.worker(&headers(&credential)).is_ok());
        let listing=workers(State(store.clone()),headers(&admin)).await.ok().unwrap().0;
        assert_eq!(listing,json!([{"worker_id":"worker","disabled":false}]));
        assert!(!listing.to_string().contains(&worker));assert!(!listing.to_string().contains(&credential));
        let result=revoke_worker(State(store.clone()),headers(&admin),Path("worker".into())).await.ok().unwrap().0;
        assert_eq!(result["cancelled_builds"],2);
        assert!(store.worker(&headers(&credential)).is_err());
        assert!(claim_build(State(store.clone()),headers(&credential)).await.is_err());
        assert!(persist_verified_artifact(&store,"uploaded",&worker,b"late bundle").is_err());
        let result=BuildResult{success:false,artifact_url:String::new(),sha256:String::new(),log:"Late completion".into()};
        assert!(finish_build(State(store.clone()),headers(&credential),Path("running".into()),Json(result)).await.is_err());
        assert_eq!(revoke_worker(State(store.clone()),headers(&admin),Path("worker".into())).await.ok().unwrap().0["cancelled_builds"],0);
        assert!(matches!(revoke_worker(State(store.clone()),headers(&admin),Path("missing".into())).await,Err(ApiError(StatusCode::NOT_FOUND,_))));
        {
            let db=store.db.lock().unwrap();
            for (id,expected) in [("running","failed"),("uploaded","failed"),("completed","completed"),("other","running")] {
                let state:String=db.query_row("SELECT state FROM builds WHERE id=?1",[id],|r|r.get(0)).unwrap();assert_eq!(state,expected);
            }
            let events:i64=db.query_row("SELECT COUNT(*) FROM audit WHERE event='worker.revoked' AND actor='admin' AND target='worker'",[],|r|r.get(0)).unwrap();assert_eq!(events,2);
        }
        use tower::ServiceExt;
        let app=router(store.clone());
        for (method,path,credential,expected) in [
            ("GET","/api/v1/workers",None,StatusCode::UNAUTHORIZED),
            ("GET","/api/v1/workers",Some(technician.as_str()),StatusCode::UNAUTHORIZED),
            ("GET","/api/v1/workers",Some(admin.as_str()),StatusCode::OK),
            ("DELETE","/api/v1/workers/worker",Some(technician.as_str()),StatusCode::UNAUTHORIZED),
            ("DELETE","/api/v1/workers/worker",Some(admin.as_str()),StatusCode::OK),
            ("DELETE","/api/v1/workers/missing",Some(admin.as_str()),StatusCode::NOT_FOUND),
            ("POST","/api/v1/worker/claim",Some(credential.as_str()),StatusCode::UNAUTHORIZED),
            ("PUT","/api/v1/worker/builds/uploaded/artifact",Some(credential.as_str()),StatusCode::UNAUTHORIZED),
        ] {
            let mut request=axum::http::Request::builder().method(method).uri(path);
            if let Some(credential)=credential {request=request.header(header::AUTHORIZATION,format!("Bearer {credential}"));}
            let response=app.clone().oneshot(request.body(axum::body::Body::empty()).unwrap()).await.unwrap();
            assert_eq!(response.status(),expected,"{method} {path}");
        }
        drop(app);
        drop(store);std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn compressed_artifact_hash_checks_expanded_limit_and_crc() {
        use std::io::Write;
        let payload=vec![42u8;256*1024];
        let mut writer=zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
        writer.start_file("artifact.exe",zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated)).unwrap();
        writer.write_all(&payload).unwrap();let bytes=writer.finish().unwrap().into_inner();
        assert!(bytes.len()<payload.len()/100);
        let mut archive=zip::ZipArchive::new(std::io::Cursor::new(&bytes)).unwrap();
        assert_eq!(bounded_artifact_digest(archive.by_name("artifact.exe").unwrap(),payload.len() as u64).ok().unwrap(),digest(&payload));
        assert!(bounded_artifact_digest(archive.by_name("artifact.exe").unwrap(),64*1024).is_err());
        let mut corrupt=bytes.clone();
        let central=corrupt.windows(4).position(|header|header==b"PK\x01\x02").unwrap();
        corrupt[14]^=1;corrupt[central+16]^=1;
        let mut archive=zip::ZipArchive::new(std::io::Cursor::new(corrupt)).unwrap();
        assert!(bounded_artifact_digest(archive.by_name("artifact.exe").unwrap(),payload.len() as u64).is_err());
    }
    #[test]
    fn worker_bundle_requires_complete_exact_recipe_and_company_trust() {
        // Inert bytes exercise the upload contract; no installer or signing
        // provider is invoked, and these fixtures are never published.
        fn archive(entries:&[(String,Vec<u8>)])->Vec<u8> {
            use std::io::Write;
            let mut writer=zip::ZipWriter::new(std::io::Cursor::new(Vec::new()));
            let options=zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
            for (name,bytes) in entries {writer.start_file(name,options).unwrap();writer.write_all(bytes).unwrap();}
            writer.finish().unwrap().into_inner()
        }
        let company=SigningKey::from_bytes(&[23;32]);let project=SigningKey::from_bytes(&[24;32]);
        let company_public=STANDARD.encode(company.verifying_key().as_bytes());
        let trust=STANDARD.encode(project.verifying_key().as_bytes());
        let profile=CompanyProfile{schema:SCHEMA,company_id:"bundle-test".into(),revision:1,issued_at:now()-1,expires_at:now()+3600,management_url:"https://support.example.com".into(),rendezvous:"support.example.com".into(),relay:"support.example.com".into(),transport_public_key:company_public.clone(),customer:Default::default(),technician:Default::default(),allow_unattended:false,updates_paused:true,rollout_percent:0,maintenance_start_utc:0,maintenance_end_utc:0,update_channel:"test".into(),next_profile_public_key:None};
        let signed_profile=SignedEnvelope::sign(&profile,&company).unwrap();
        for edition in [Edition::Customer,Edition::Technician] {for format in ["exe","msi"] {
            let installer=b"inert installer fixture";let agent=b"inert agent fixture";
            let executable=if edition==Edition::Customer {"Swan Remote Support.exe"}else{"SwanRemoteSupport-Technician.exe"};
            let mut installed_files=vec![InstalledFile{path:executable.into(),sha256:digest(installer)}];
            if edition==Edition::Customer {for name in ["librustdesk.dll","flutter_windows.dll"] {installed_files.push(InstalledFile{path:name.into(),sha256:digest(b"inert library fixture")});}}
            let release=Release{schema:SCHEMA,product:PRODUCT.into(),version:"2.0.0".into(),sequence:1,edition:edition.clone(),architecture:"x64".into(),channel:"test".into(),expires_at:now()+3600,artifact_url:format!("https://releases.example/install.{format}"),sha256:digest(installer),installed_sha256:digest(installer),installed_files,agent_url:"https://releases.example/agent.exe".into(),agent_sha256:digest(agent),publisher:"Inert fixture".into(),publisher_certificate_sha256:"d".repeat(64),windows_versions:vec!["windows_11".into()],source_url:"https://releases.example/source.tar.gz".into(),format:format.into(),rollback_protocol:1};
            release.validate(&edition,0,now()).unwrap();
            let signed_release=SignedEnvelope::sign(&release,&project).unwrap();
            let job=json!({"profile":signed_profile,"release":signed_release});
            let bootstrap=swan_agent::Bootstrap{schema:SCHEMA,edition:edition.clone(),company_id:profile.company_id.clone(),management_url:profile.management_url.clone(),profile_public_key:company_public.clone(),release_public_key:trust.clone()};
            let mut entries=vec![
                (format!("SwanRemoteSupport-install.{format}"),installer.to_vec()),("swan-agent.exe".into(),agent.to_vec()),
                ("bootstrap.json".into(),serde_json::to_vec_pretty(&bootstrap).unwrap()),
                ("company-profile.json".into(),serde_json::to_vec_pretty(&signed_profile).unwrap()),
                ("release.json".into(),serde_json::to_vec_pretty(&signed_release).unwrap()),
            ];
            for (name,bytes) in COMPANY_BUNDLE_TEXT_FILES {entries.push((name.into(),bundle_text(bytes).unwrap()));}
            let recipe_count=5+COMPANY_BUNDLE_TEXT_FILES.len();
            assert_eq!(entries.len(),recipe_count);
            let verify=|bytes:&[u8]|validate_worker_bundle(bytes,&job,&release,&company.verifying_key(),&trust).is_ok();
            let valid=archive(&entries);assert!(verify(&valid),"Complete {edition:?}/{format} bundle was rejected");
            let helper=entries.iter().position(|(name,_)|name=="Get-MsiInstallMode.ps1").unwrap();
            let mut altered=entries.clone();altered[helper].1=b"altered script".to_vec();assert!(!verify(&archive(&altered)));
            let uninstall=entries.iter().position(|(name,_)|name=="Uninstall-Technician.ps1").unwrap();
            let mut altered=entries.clone();altered[uninstall].1=b"altered uninstall script".to_vec();assert!(!verify(&archive(&altered)));
            let mut missing=entries.clone();missing.remove(helper);assert!(!verify(&archive(&missing)));
            let mut extra=entries.clone();extra.push(("extra.dll".into(),b"unexpected".to_vec()));assert!(!verify(&archive(&extra)));
            let mut renamed=entries.clone();renamed[helper].0="Get-MsiInstallM0de.ps1".into();assert!(!verify(&archive(&renamed)));
            let mut binary=entries.clone();binary[0].1=b"different installer".to_vec();assert!(!verify(&archive(&binary)));
            for index in [3,4] {
                let mut hidden=entries.clone();let mut metadata:Value=serde_json::from_slice(&hidden[index].1).unwrap();
                metadata["technician_token"]=json!("inert secret canary");hidden[index].1=serde_json::to_vec_pretty(&metadata).unwrap();
                assert!(!verify(&archive(&hidden)),"Unsigned fields cannot enter public bundles");
            }
            let mut duplicate_json=entries.clone();
            let mut metadata=b"{\"payload\":\"inert secret canary\",".to_vec();
            metadata.extend_from_slice(&duplicate_json[3].1[1..]);duplicate_json[3].1=metadata;
            assert!(!verify(&archive(&duplicate_json)),"Duplicate JSON keys cannot hide discarded metadata");
            for (field,value) in [("company_id",json!("wrong-company")),("schema",json!(2)),("management_url",json!("https://wrong.example.com")),("edition",json!(if edition==Edition::Customer {"technician"}else{"customer"}))] {
                let mut wrong=entries.clone();let mut metadata:Value=serde_json::from_slice(&wrong[2].1).unwrap();metadata[field]=value;
                // Serialize through Bootstrap so the fixture keeps the worker's field order.
                let bootstrap:swan_agent::Bootstrap=serde_json::from_value(metadata).unwrap();wrong[2].1=serde_json::to_vec_pretty(&bootstrap).unwrap();
                assert!(!verify(&archive(&wrong)),"Wrong bootstrap {field} accepted");
            }
            let mut local_name=valid.clone();local_name[30]=b'X';assert!(!verify(&local_name),"Local and central filenames must agree");
            let mut truncated=valid.clone();truncated.pop();assert!(!verify(&truncated));
            let mut prefixed=b"MZ".to_vec();prefixed.extend_from_slice(&valid);assert!(!verify(&prefixed));
            // zip 2.4.2 hides duplicate names behind its unique-name map. Build
            // a distinct alias, then change both header names to the real helper.
            let alias=b"Get-MsiInstallM0de.ps1";let canonical=b"Get-MsiInstallMode.ps1";
            let mut duplicate=entries.clone();duplicate.push((String::from_utf8(alias.to_vec()).unwrap(),entries[helper].1.clone()));
            let mut duplicate=archive(&duplicate);
            for offset in 0..=duplicate.len()-alias.len() {if &duplicate[offset..offset+alias.len()]==alias {duplicate[offset..offset+alias.len()].copy_from_slice(canonical);}}
            assert_eq!(zip::ZipArchive::new(std::io::Cursor::new(&duplicate)).unwrap().len(),recipe_count,"Regression must exercise hidden physical duplicates");
            assert!(!verify(&duplicate));
            let footer=duplicate.len()-22;let count=(recipe_count as u16).to_le_bytes();
            duplicate[footer+8..footer+10].copy_from_slice(&count);duplicate[footer+10..footer+12].copy_from_slice(&count);
            assert!(!verify(&duplicate),"Forged entry counts cannot hide duplicate records");
            let mut expired=profile.clone();expired.expires_at=now();let expired=SignedEnvelope::sign(&expired,&company).unwrap();
            let mut expired_entries=entries.clone();expired_entries[3].1=serde_json::to_vec_pretty(&expired).unwrap();
            let expired_job=json!({"profile":expired,"release":signed_release});
            assert!(validate_worker_bundle(&archive(&expired_entries),&expired_job,&release,&company.verifying_key(),&trust).is_err());
        }}
    }
    #[test]
    fn legacy_schema_migration_requires_encrypted_backup_and_keeps_original_key() {
        let directory=std::env::temp_dir().join(format!("swan-key-migration-{}",random_token()));
        let initial=Store::open(&directory).unwrap();let public=initial.key.verifying_key();
        initial.db.lock().unwrap().execute_batch("DROP TABLE profile_signing_keys;PRAGMA user_version=1;").unwrap();drop(initial);
        assert!(Store::open_with_backup_password(&directory,None).is_err());
        let db=Connection::open(directory.join("management.sqlite3")).unwrap();assert_eq!(db.pragma_query_value(None,"user_version",|r|r.get::<_,u32>(0)).unwrap(),1);drop(db);
        let upgraded=Store::open_with_backup_password(&directory,Some("schema migration backup password")).unwrap();assert_eq!(upgraded.key.verifying_key(),public);
        let archive=std::fs::read_dir(&directory).unwrap().map(|e|e.unwrap().path()).find(|path|path.extension().is_some_and(|e|e=="swan-backup")).unwrap();drop(upgraded);
        let restored_path=directory.with_extension("restored");crate::backup::restore(&restored_path,&archive,"schema migration backup password").unwrap();
        let db=Connection::open(restored_path.join("management.sqlite3")).unwrap();assert_eq!(db.pragma_query_value(None,"user_version",|r|r.get::<_,u32>(0)).unwrap(),1);drop(db);
        std::fs::remove_dir_all(directory).unwrap();std::fs::remove_dir_all(restored_path).unwrap();
    }
    #[test]
    fn profile_key_rotation_is_atomic_survives_backup_and_requires_published_transition() {
        let directory=std::env::temp_dir().join(format!("swan-server-key-rotation-{}",random_token()));
        let store=Store::open(&directory).unwrap();assert!(Store::open(&directory).is_err());
        let old=store.key.verifying_key();let old_public=STANDARD.encode(old.as_bytes());
        let profile=CompanyProfile{schema:1,company_id:"rotation-test".into(),revision:1,issued_at:now()-1,expires_at:now()+3600,management_url:"https://support.example.com".into(),rendezvous:"support.example.com".into(),relay:"support.example.com".into(),transport_public_key:old_public.clone(),customer:Default::default(),technician:Default::default(),allow_unattended:false,updates_paused:true,rollout_percent:0,maintenance_start_utc:0,maintenance_end_utc:0,update_channel:"test".into(),next_profile_public_key:None};
        store.db.lock().unwrap().execute("INSERT INTO profile VALUES(1,?1)",[serde_json::to_string(&profile).unwrap()]).unwrap();
        assert!(store.activate_profile_rotation().is_err());
        let next=store.prepare_profile_rotation().unwrap();assert_ne!(next,old_public);assert!(store.prepare_profile_rotation().is_err());
        let transition=store.profile().unwrap();assert_eq!(transition.revision,2);assert_eq!(transition.next_profile_public_key.as_ref(),Some(&next));
        let envelope=SignedEnvelope::sign(&transition,&store.key).unwrap();assert!(envelope.verify::<CompanyProfile>(&old).is_ok());
        let mut altered=transition.clone();altered.next_profile_public_key=Some(old_public);
        store.db.lock().unwrap().execute("UPDATE profile SET body=?1 WHERE id=1",[serde_json::to_string(&altered).unwrap()]).unwrap();
        assert!(store.activate_profile_rotation().is_err());
        store.db.lock().unwrap().execute("UPDATE profile SET body=?1 WHERE id=1",[serde_json::to_string(&transition).unwrap()]).unwrap();
        assert_eq!(store.activate_profile_rotation().unwrap(),next);drop(store);
        let restarted=Store::open(&directory).unwrap();assert_eq!(STANDARD.encode(restarted.key.verifying_key().as_bytes()),next);
        let activated=restarted.profile().unwrap();assert_eq!(activated.revision,3);assert!(activated.next_profile_public_key.is_none());
        let signed=SignedEnvelope::sign(&activated,&restarted.key).unwrap();assert!(signed.verify::<CompanyProfile>(&old).is_err());assert!(signed.verify::<CompanyProfile>(&public_key(&next).unwrap()).is_ok());
        let archive=directory.with_extension("backup");crate::backup::export(&directory,&archive,"rotation backup test passphrase").unwrap();drop(restarted);
        let restored_path=directory.with_extension("restored");crate::backup::restore(&restored_path,&archive,"rotation backup test passphrase").unwrap();
        let restored=Store::open(&restored_path).unwrap();assert_eq!(STANDARD.encode(restored.key.verifying_key().as_bytes()),next);assert_eq!(restored.profile().unwrap().revision,3);drop(restored);
        std::fs::remove_dir_all(directory).unwrap();std::fs::remove_dir_all(restored_path).unwrap();std::fs::remove_file(archive).unwrap();
    }
    #[tokio::test]
    async fn release_withdrawal_requires_admin_and_stops_update_selection() {
        let directory=std::env::temp_dir().join(format!("swan-release-withdrawal-{}",random_token()));
        let store=std::sync::Arc::new(Store::open(&directory).unwrap());
        let public=STANDARD.encode(store.key.verifying_key().as_bytes());
        let profile=CompanyProfile{schema:1,company_id:"release-test".into(),revision:1,issued_at:now()-1,expires_at:now()+3600,management_url:"https://support.example.com".into(),rendezvous:"support.example.com".into(),relay:"support.example.com".into(),transport_public_key:public,customer:Default::default(),technician:Default::default(),allow_unattended:false,updates_paused:false,rollout_percent:100,maintenance_start_utc:0,maintenance_end_utc:0,update_channel:"stable".into(),next_profile_public_key:None};
        let release=Release{schema:1,product:PRODUCT.into(),version:"2.0.0".into(),sequence:1,edition:Edition::Technician,architecture:"x64".into(),channel:"stable".into(),expires_at:now()+3600,artifact_url:"https://releases.example/install.msi".into(),sha256:"a".repeat(64),installed_sha256:"b".repeat(64),installed_files:vec![InstalledFile{path:"SwanRemoteSupport-Technician.exe".into(),sha256:"b".repeat(64)}],agent_url:"https://releases.example/agent.exe".into(),agent_sha256:"c".repeat(64),publisher:"Example".into(),publisher_certificate_sha256:"d".repeat(64),windows_versions:vec!["windows_11".into()],source_url:"https://releases.example/source.tar.gz".into(),format:"msi".into(),rollback_protocol:0};
        let envelope=SignedEnvelope::sign(&release,&store.key).unwrap();
        let admin_token=random_token();let technician_token=random_token();
        let completed=Uuid::new_v4().to_string();
        let in_flight=Uuid::new_v4().to_string();let worker_token=random_token();let worker=digest(&worker_token);
        {
            let db=store.db.lock().unwrap();
            db.execute("INSERT INTO profile VALUES(1,?1)",[serde_json::to_string(&profile).unwrap()]).unwrap();
            for (id,role,credential) in [("admin","admin",&admin_token),("technician","technician",&technician_token)] {
                db.execute("INSERT INTO users(id,username,password_hash,totp_secret,role) VALUES(?1,?1,'unused','unused',?2)",params![id,role]).unwrap();
                db.execute("INSERT INTO sessions VALUES(?1,?2,?3)",params![digest(credential),id,now()+3600]).unwrap();
            }
            db.execute("INSERT INTO releases VALUES('release',?1,?2,1)",params![serde_json::to_string(&release).unwrap(),serde_json::to_string(&envelope).unwrap()]).unwrap();
            let job=json!({"release":envelope}).to_string();
            for state in ["queued","running","uploaded"] {db.execute("INSERT INTO builds(id,body,state) VALUES(?1,?2,?1)",params![state,job]).unwrap();}
            db.execute("INSERT INTO builds(id,body,state) VALUES(?1,?2,'completed')",params![completed,job]).unwrap();
            db.execute("INSERT INTO workers(id,token_hash) VALUES('worker',?1)",[&worker]).unwrap();
            db.execute("INSERT INTO builds(id,body,state,worker,claimed_at) VALUES(?1,?2,'running',?3,?4)",params![in_flight,job,worker,now()]).unwrap();
        }
        persist_verified_artifact(&store,&in_flight,&worker,b"previously validated bundle").ok().unwrap();
        let headers=|credential:&str| {let mut h=HeaderMap::new();h.insert(header::AUTHORIZATION,format!("Bearer {credential}").parse().unwrap());h};
        assert!(!select_update(&store,"technician",Edition::Technician,None).ok().unwrap().0.is_null());
        assert!(select_update(&store,"device",Edition::Customer,None).ok().unwrap().0.is_null());
        let mut portable=release.clone();portable.sequence=2;portable.format="exe".into();portable.sha256=portable.installed_sha256.clone();
        release.validate(&Edition::Technician,0,now()).unwrap();portable.validate(&Edition::Technician,0,now()).unwrap();
        let portable_envelope=SignedEnvelope::sign(&portable,&store.key).unwrap();
        store.db.lock().unwrap().execute("INSERT INTO releases VALUES('portable-release',?1,?2,1)",params![serde_json::to_string(&portable).unwrap(),serde_json::to_string(&portable_envelope).unwrap()]).unwrap();
        assert_eq!(select_update(&store,"technician",Edition::Technician,Some("msi")).ok().unwrap().0,serde_json::to_value(&envelope).unwrap());
        assert_eq!(select_update(&store,"technician",Edition::Technician,Some("exe")).ok().unwrap().0,serde_json::to_value(&portable_envelope).unwrap());
        assert_eq!(select_update(&store,"technician",Edition::Technician,None).ok().unwrap().0,serde_json::to_value(&portable_envelope).unwrap());
        assert!(select_update(&store,"technician",Edition::Technician,Some("zip")).is_err());
        assert!(technician_update(State(store.clone()),HeaderMap::new(),Query(UpdateQuery{format:Some("exe".into())})).await.is_err());
        assert!(device_update(State(store.clone()),HeaderMap::new(),Query(UpdateQuery{format:Some("msi".into())})).await.is_err());
        assert_eq!(technician_update(State(store.clone()),headers(&technician_token),Query(UpdateQuery{format:Some("msi".into())})).await.ok().unwrap().0,serde_json::to_value(&envelope).unwrap());
        store.db.lock().unwrap().execute("UPDATE releases SET approved=0 WHERE id='portable-release'",[]).unwrap();
        assert!(select_update(&store,"technician",Edition::Technician,Some("exe")).ok().unwrap().0.is_null());
        let save_policy=|policy:&CompanyProfile| {store.db.lock().unwrap().execute("UPDATE profile SET body=?1 WHERE id=1",[serde_json::to_string(policy).unwrap()]).unwrap();};
        let mut paused=profile.clone();paused.updates_paused=true;save_policy(&paused);
        assert!(select_update(&store,"technician",Edition::Technician,None).ok().unwrap().0.is_null());
        let mut staged=profile.clone();staged.rollout_percent=0;save_policy(&staged);
        assert!(select_update(&store,"technician",Edition::Technician,None).ok().unwrap().0.is_null());
        let mut channel=profile.clone();channel.update_channel="test".into();save_policy(&channel);
        assert!(select_update(&store,"technician",Edition::Technician,None).ok().unwrap().0.is_null());
        save_policy(&profile);
        assert!(withdraw_release(State(store.clone()),headers(&technician_token),Path("release".into())).await.is_err());
        assert!(!select_update(&store,"technician",Edition::Technician,None).ok().unwrap().0.is_null());
        assert!(withdraw_release(State(store.clone()),headers(&admin_token),Path("release".into())).await.is_ok());
        assert!(select_update(&store,"technician",Edition::Technician,None).ok().unwrap().0.is_null());
        assert!(withdraw_release(State(store.clone()),headers(&admin_token),Path("missing".into())).await.is_err());
        let cancelled:i64=store.db.lock().unwrap().query_row("SELECT COUNT(*) FROM builds WHERE state='failed'",[],|r|r.get(0)).unwrap();assert_eq!(cancelled,4);
        assert!(persist_verified_artifact(&store,&in_flight,&worker,b"late replacement").is_err());
        let late_result=BuildResult{success:false,artifact_url:String::new(),sha256:String::new(),log:"Worker reports a different terminal outcome".into()};
        assert!(finish_build(State(store.clone()),headers(&worker_token),Path(in_flight.clone()),Json(late_result)).await.is_err());
        let late_success=BuildResult{success:true,artifact_url:format!("{}/api/v1/downloads/{in_flight}",profile.management_url),sha256:digest(b"previously validated bundle"),log:"Worker finished after withdrawal".into()};
        assert!(finish_build(State(store.clone()),headers(&worker_token),Path(in_flight.clone()),Json(late_success)).await.is_err());
        assert_eq!(std::fs::read(store.artifact_dir.join(format!("{in_flight}.zip"))).unwrap(),b"previously validated bundle");
        assert!(matches!(download_artifact(State(store.clone()),Path(completed),HeaderMap::new()).await,Err(ApiError(StatusCode::NOT_FOUND,_))));
        let audit:i64=store.db.lock().unwrap().query_row("SELECT COUNT(*) FROM audit WHERE event='release.withdrawn'",[],|r|r.get(0)).unwrap();assert_eq!(audit,1);
        // Restoring release approval cannot revive a cancelled worker claim.
        store.db.lock().unwrap().execute("UPDATE releases SET approved=1 WHERE id='release'",[]).unwrap();
        assert!(persist_verified_artifact(&store,&in_flight,&worker,b"late replacement").is_err());
        assert!(claim_build(State(store.clone()),headers(&worker_token)).await.ok().unwrap().0.is_null());
        drop(store);std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn uploaded_artifact_retry_preserves_bytes_and_owner() {
        let directory=std::env::temp_dir().join(format!("swan-upload-retry-{}",random_token()));
        let store=Store::open(&directory).unwrap();let id=Uuid::new_v4().to_string();
        store.db.lock().unwrap().execute("INSERT INTO builds(id,body,state,worker) VALUES(?1,'{}','running','owner')",[&id]).unwrap();
        assert!(persist_verified_artifact(&store,&id,"owner",b"validated bundle").is_ok());
        assert!(persist_verified_artifact(&store,&id,"owner",b"validated bundle").is_ok());
        assert!(persist_verified_artifact(&store,&id,"owner",b"different bundle").is_err());
        assert!(persist_verified_artifact(&store,&id,"other-worker",b"validated bundle").is_err());
        assert_eq!(std::fs::read(store.artifact_dir.join(format!("{id}.zip"))).unwrap(),b"validated bundle");
        store.db.lock().unwrap().execute("UPDATE builds SET state='completed' WHERE id=?1",[&id]).unwrap();
        assert!(persist_verified_artifact(&store,&id,"owner",b"validated bundle").is_err());
        drop(store);std::fs::remove_dir_all(directory).unwrap();
    }
    #[tokio::test]
    async fn lost_completion_response_retries_without_changing_terminal_result() {
        let directory=std::env::temp_dir().join(format!("swan-worker-retry-{}",random_token()));
        let store=Arc::new(Store::open(&directory).unwrap());
        let credential=random_token();let worker=digest(&credential);let id=Uuid::new_v4().to_string();
        {
            let db=store.db.lock().unwrap();
            db.execute("INSERT INTO workers(id,token_hash) VALUES('test',?1)",[&worker]).unwrap();
            db.execute("INSERT INTO builds(id,body,state,worker) VALUES(?1,'{}','running',?2)",params![id,worker]).unwrap();
        }
        let mut headers=HeaderMap::new();headers.insert(header::AUTHORIZATION,format!("Bearer {credential}").parse().unwrap());
        let result=||BuildResult{success:false,artifact_url:String::new(),sha256:String::new(),log:"Rejected invalid package".into()};
        assert!(finish_build(State(store.clone()),headers.clone(),Path(id.clone()),Json(result())).await.is_ok());
        assert!(finish_build(State(store.clone()),headers.clone(),Path(id.clone()),Json(result())).await.is_ok());
        let mut changed=result();changed.log="Different outcome".into();
        assert!(finish_build(State(store.clone()),headers.clone(),Path(id.clone()),Json(changed)).await.is_err());
        store.db.lock().unwrap().execute("UPDATE workers SET disabled=1",[]).unwrap();
        assert!(finish_build(State(store.clone()),headers,Path(id.clone()),Json(result())).await.is_err());
        let saved:String=store.db.lock().unwrap().query_row("SELECT result FROM builds WHERE id=?1",[id],|r|r.get(0)).unwrap();
        assert_eq!(serde_json::from_str::<Value>(&saved).unwrap()["log"],"Rejected invalid package");
        drop(store);std::fs::remove_dir_all(directory).unwrap();
    }
}
async fn withdraw_release(State(s):State<Shared>,headers:HeaderMap,Path(id):Path<String>)->ApiResult {
    let user=s.user(&headers,true)?;
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    if tx.execute("UPDATE releases SET approved=0 WHERE id=?1",[&id])?!=1{return Err(bad("Unknown release"));}
    let envelope:String=tx.query_row("SELECT envelope FROM releases WHERE id=?1",[&id],|r|r.get(0))?;
    let envelope:Value=serde_json::from_str(&envelope)?;
    let jobs={let mut statement=tx.prepare("SELECT id,body FROM builds WHERE state IN ('queued','running','uploaded')")?;
        let rows=statement.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<Result<Vec<_>,_>>()?;rows};
    for (job,body) in jobs {
        if serde_json::from_str::<Value>(&body)?["release"]==envelope {
            let result=json!({"artifact_url":"","sha256":"","log":"Release approval withdrawn"});
            tx.execute("UPDATE builds SET state='failed',result=?1 WHERE id=?2",params![result.to_string(),job])?;
        }
    }
    tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,'release.withdrawn',?3)",params![now(),user,id])?;
    tx.commit()?;Ok(Json(json!({"ok":true})))
}
async fn revoke_group_access(State(s):State<Shared>,headers:HeaderMap,Path((group,user)):Path<(String,String)>)->ApiResult {
    let actor=s.user(&headers,true)?;
    if group.len()>64 || group.is_empty(){return Err(bad("Invalid group"));}
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
    tx.execute("DELETE FROM group_access WHERE group_id=?1 AND user_id=?2",params![group,user])?;
    // Re-granting access must not revive authorization issued before revocation.
    tx.execute("UPDATE grants SET closed=1,lease_until=0 WHERE user_id=?1 AND device_id IN (SELECT id FROM devices WHERE group_id=?2)",params![user,group])?;
    tx.execute("INSERT INTO audit(at,actor,event,target) VALUES(?1,?2,'group.access_revoked',?3)",params![now(),actor,format!("{group}/{user}")])?;
    tx.commit()?;
    Ok(Json(json!({"ok":true})))
}
async fn finish_build(State(s):State<Shared>,headers:HeaderMap,Path(id):Path<String>,Json(input):Json<BuildResult>)->ApiResult {
    s.worker(&headers)?;
    if Uuid::parse_str(&id).is_err(){return Err(bad("Invalid build identity"));}
    let worker=digest(token(&headers)?);
    if input.log.len()>8192 || (input.success && (https_url(&input.artifact_url).is_err() || input.sha256.len()!=64 || !input.sha256.bytes().all(|b|b.is_ascii_hexdigit()))) {return Err(bad("Invalid build result"));}
    let result=json!({"artifact_url":input.artifact_url,"sha256":input.sha256,"log":input.log});
    let terminal=if input.success{"completed"}else{"failed"};
    let previous:Option<(String,String)>=s.db.lock().unwrap().query_row(
        "SELECT state,result FROM builds WHERE id=?1 AND worker=?2",params![id,worker],
        |r|Ok((r.get(0)?,r.get(1)?))).optional()?;
    match previous {
        Some((state,saved)) if state==terminal && serde_json::from_str::<Value>(&saved)?==result=>return Ok(Json(json!({"ok":true}))),
        Some((state,_)) if state=="running" || state=="uploaded"=>{},
        _=>return Err(denied()),
    }
    if input.success {
        let expected=format!("{}/api/v1/downloads/{id}",s.profile()?.management_url.trim_end_matches('/'));
        if input.artifact_url!=expected{return Err(bad("Artifact URL must use this company server"));}
        let bytes=tokio::fs::read(s.artifact_dir.join(format!("{id}.zip"))).await.map_err(|_|bad("Verified artifact upload required"))?;
        if !digest(bytes).eq_ignore_ascii_case(&input.sha256){return Err(bad("Uploaded artifact hash mismatch"));}
    }
    let db=s.db.lock().unwrap();
    let count=db.execute("UPDATE builds SET state=?1,result=?2 WHERE id=?3 AND worker=?4 AND (state='uploaded' OR (state='running' AND ?5=0))",params![terminal,result.to_string(),id,worker,input.success])?;
    if count!=1 {
        let saved:Option<String>=db.query_row("SELECT result FROM builds WHERE id=?1 AND worker=?2 AND state=?3",params![id,worker,terminal],|r|r.get(0)).optional()?;
        if saved.map(|value|serde_json::from_str::<Value>(&value)).transpose()?.as_ref()!=Some(&result){return Err(denied());}
    }
    Ok(Json(json!({"ok":true})))
}

async fn upload_artifact(State(s):State<Shared>,Path(id):Path<String>,request:axum::extract::Request)->ApiResult {
    s.worker(request.headers())?;
    if id.len()!=36 || !id.bytes().all(|c|c.is_ascii_hexdigit() || c==b'-'){return Err(bad("Invalid build identity"));}
    let worker=digest(token(request.headers())?);
    let job:String=s.db.lock().unwrap().query_row("SELECT body FROM builds WHERE id=?1 AND worker=?2 AND state IN ('running','uploaded')",params![id,worker],|r|r.get(0)).optional()?.ok_or_else(denied)?;
    let bytes=axum::body::to_bytes(request.into_body(),512*1024*1024).await.map_err(|_|bad("Artifact too large"))?;
    let job:Value=serde_json::from_str(&job)?;
    let release_envelope:SignedEnvelope=serde_json::from_value(job["release"].clone())?;
    let trust=std::env::var("SWAN_RELEASE_PUBLIC_KEY").map_err(|_|bad("Release key not configured"))?;
    let release:Release=release_envelope.verify(&public_key(&trust)?).map_err(|_|denied())?;
    release.validate(&release.edition,0,now()).map_err(|_|bad("Expired release"))?;
    let company_key=s.key.verifying_key();
    let hash=tokio::task::spawn_blocking(move || {
        validate_worker_bundle(&bytes,&job,&release,&company_key,&trust)?;
        let hash=digest(&bytes);
        persist_verified_artifact(&s,&id,&worker,&bytes)?;
        Ok::<_,ApiError>(hash)
    })
        .await.map_err(|_|bad("Artifact persistence failed"))??;
    Ok(Json(json!({"sha256":hash})))
}

fn validate_bundle_zip_layout(bytes:&[u8],count:usize)->Result<(),ApiError> {
    // Admit the worker's bounded, seek-written ZIP32 recipe only. zip 2.4.2
    // deduplicates central-directory names, so ZipArchive::len alone cannot
    // establish that an archive has exactly the expected physical entries.
    let range=|offset:usize,length:usize|->Result<&[u8],ApiError>{
        bytes.get(offset..offset.checked_add(length).ok_or_else(||bad("Invalid ZIP bounds"))?).ok_or_else(||bad("Invalid ZIP bounds"))
    };
    let word=|offset:usize|->Result<usize,ApiError>{let data=range(offset,2)?;Ok(u16::from_le_bytes([data[0],data[1]]) as usize)};
    let dword=|offset:usize|->Result<usize,ApiError>{let data=range(offset,4)?;Ok(u32::from_le_bytes([data[0],data[1],data[2],data[3]]) as usize)};
    let end=bytes.len().checked_sub(22).ok_or_else(||bad("Incomplete ZIP footer"))?;
    if range(end,4)?!=b"PK\x05\x06" || word(end+4)?!=0 || word(end+6)?!=0 || word(end+8)?!=count || word(end+10)?!=count || word(end+20)?!=0 {return Err(bad("Unexpected ZIP footer"));}
    let start=dword(end+16)?;
    if start.checked_add(dword(end+12)?)!=Some(end) {return Err(bad("Invalid ZIP directory bounds"));}
    let mut central=start;let mut local=0;let mut names=std::collections::HashSet::new();
    for _ in 0..count {
        let header=range(central,46)?;
        if &header[..4]!=b"PK\x01\x02" || word(central+30)?!=0 || word(central+32)?!=0 || word(central+34)?!=0 || dword(central+42)?!=local {return Err(bad("Unsupported ZIP entry layout"));}
        let name_length=word(central+28)?;
        let name=range(central+46,name_length)?;
        if !names.insert(name) {return Err(bad("Duplicate ZIP entry"));}
        let local_header=range(local,30)?;
        let flags=word(central+8)?;
        if &local_header[..4]!=b"PK\x03\x04" || flags & !0x0800!=0 || word(local+6)?!=flags || word(central+10)?!=8 || word(local+8)?!=8 || word(local+26)?!=name_length || word(local+28)?!=0 || range(local+30,name_length)?!=name || range(local+14,12)?!=range(central+16,12)? {return Err(bad("Ambiguous ZIP entry headers"));}
        let compressed=dword(central+20)?;
        local=local.checked_add(30+name_length).and_then(|offset|offset.checked_add(compressed)).ok_or_else(||bad("Invalid ZIP data bounds"))?;
        central=central.checked_add(46+name_length).ok_or_else(||bad("Invalid ZIP directory bounds"))?;
        if local>start || central>end {return Err(bad("ZIP entry exceeds its region"));}
    }
    if local!=start || central!=end {return Err(bad("Unaccounted ZIP contents"));}
    Ok(())
}

fn validate_worker_bundle(bytes:&[u8],job:&Value,release:&Release,company_key:&ed25519_dalek::VerifyingKey,trust:&str)->Result<(),ApiError> {
    // Verify content without extracting any paths. Download bundles cannot inject executables or setup scripts.
    validate_bundle_zip_layout(bytes,5+COMPANY_BUNDLE_TEXT_FILES.len())?;
    let mut zip=zip::ZipArchive::new(std::io::Cursor::new(bytes)).map_err(|_|bad("Invalid ZIP artifact"))?;
    if zip.len()!=5+COMPANY_BUNDLE_TEXT_FILES.len() {return Err(bad("Unexpected bundle contents"));}
    use std::io::Read;
    for (name,expected) in [(format!("SwanRemoteSupport-install.{}",release.format),&release.sha256),("swan-agent.exe".into(),&release.agent_sha256)] {
        let file=zip.by_name(&name).map_err(|_|bad("Missing bundle file"))?;
        if file.is_dir() || file.is_symlink() || file.size()>512*1024*1024{return Err(bad("Invalid bundle entry type or size"));}
        if bounded_artifact_digest(file,512*1024*1024)?!=expected.to_lowercase(){return Err(bad("Bundle binary hash mismatch"));}
    }
    let mut read=|name:&str,limit:u64|->Result<Vec<u8>,ApiError>{
        let file=zip.by_name(name).map_err(|_|bad("Missing bundle file"))?;
        if file.is_dir() || file.is_symlink() || file.size()>limit{return Err(bad("Invalid bundle entry type or size"));}
        let mut output=Vec::new();file.take(limit+1).read_to_end(&mut output).map_err(|_|bad("Unreadable bundle"))?;
        if output.len() as u64>limit{return Err(bad("Bundle entry too large"));}Ok(output)
    };
    for (name,expected) in COMPANY_BUNDLE_TEXT_FILES {let expected=bundle_text(expected)?;if read(name,expected.len() as u64)?!=expected{return Err(bad("Bundle script or license mismatch"));}}
    let profile_bytes=read("company-profile.json",2*1024*1024)?;
    let profile:SignedEnvelope=serde_json::from_slice(&profile_bytes)?;
    if profile_bytes!=serde_json::to_vec_pretty(&profile)? || serde_json::to_value(&profile)?!=job["profile"] {return Err(bad("Wrong company profile"));}
    let release_bytes=read("release.json",2*1024*1024)?;
    let copied_release:SignedEnvelope=serde_json::from_slice(&release_bytes)?;
    if release_bytes!=serde_json::to_vec_pretty(&copied_release)? || serde_json::to_value(&copied_release)?!=job["release"] {return Err(bad("Wrong release metadata"));}
    let bootstrap_bytes=read("bootstrap.json",16*1024)?;
    let bootstrap:swan_agent::Bootstrap=serde_json::from_slice(&bootstrap_bytes)?;
    if bootstrap_bytes!=serde_json::to_vec_pretty(&bootstrap)? {return Err(bad("Noncanonical bootstrap configuration"));}
    bootstrap.validate().map_err(|_|bad("Invalid bootstrap configuration"))?;
    let p:CompanyProfile=profile.verify(company_key).map_err(|_|denied())?;
    p.validate(&p.company_id,0,now()).map_err(|_|bad("Expired or invalid bundle profile"))?;
    if bootstrap.company_id!=p.company_id || bootstrap.edition!=release.edition || bootstrap.management_url!=p.management_url || bootstrap.profile_public_key!=STANDARD.encode(company_key.as_bytes()) || bootstrap.release_public_key!=trust {return Err(bad("Bootstrap trust mismatch"));}
    Ok(())
}

fn bounded_artifact_digest(mut reader:impl std::io::Read,limit:u64)->Result<String,ApiError> {
    use sha2::{Digest,Sha256};
    let mut hash=Sha256::new();let mut total=0u64;let mut chunk=[0u8;64*1024];
    loop {
        let count=reader.read(&mut chunk).map_err(|_|bad("Unreadable bundle"))?;
        if count==0 {break;}
        total=total.checked_add(count as u64).filter(|size|*size<=limit).ok_or_else(||bad("Bundle entry too large"))?;
        hash.update(&chunk[..count]);
    }
    Ok(hex::encode(hash.finalize()))
}

// Called only after bundle validation, under a blocking task.
fn persist_verified_artifact(s:&Store,id:&str,worker:&str,bytes:&[u8])->Result<(),ApiError> {
        let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
        let state:Option<String>=tx.query_row("SELECT state FROM builds WHERE id=?1 AND worker=?2",params![id,worker],|r|r.get(0)).optional()?;
        let target=s.artifact_dir.join(format!("{id}.zip"));
        match state.as_deref() {
            Some("uploaded")=>{
                let saved=std::fs::read(&target).map_err(|_|bad("Uploaded artifact missing"))?;
                if digest(saved)!=digest(bytes){return Err(bad("Conflicting artifact retry"));}
                return Ok(());
            },
            Some("running")=>{},
            _=>return Err(denied()),
        }
        let temporary=s.artifact_dir.join(format!("{id}.partial"));
        use std::io::Write;
        let mut artifact=std::fs::File::create(&temporary).map_err(|_|bad("Cannot save artifact"))?;
        artifact.write_all(&bytes).map_err(|_|bad("Cannot write artifact"))?;
        artifact.sync_all().map_err(|_|bad("Cannot flush artifact"))?;drop(artifact);
        if target.exists(){std::fs::remove_file(&target).map_err(|_|bad("Cannot replace retry artifact"))?;}
        std::fs::rename(&temporary,target).map_err(|_|bad("Cannot publish artifact"))?;
        tx.execute("UPDATE builds SET state='uploaded' WHERE id=?1",[&id])?;tx.commit()?;Ok(())
}
async fn download_artifact(State(s):State<Shared>,Path(id):Path<String>,headers:HeaderMap)->Result<Response,ApiError> {
    if id.len()!=36 || !id.bytes().all(|c|c.is_ascii_hexdigit() || c==b'-'){return Err(bad("Invalid build identity"));}
    let job:String=s.db.lock().unwrap().query_row("SELECT body FROM builds WHERE id=?1 AND state='completed'",[&id],|r|r.get(0)).optional()?.ok_or(ApiError(StatusCode::NOT_FOUND,"Artifact unavailable"))?;
    let job:Value=serde_json::from_str(&job)?;
    let envelope:SignedEnvelope=serde_json::from_value(job["release"].clone())?;
    let approved:bool=s.db.lock().unwrap().query_row("SELECT EXISTS(SELECT 1 FROM releases WHERE approved=1 AND envelope=?1)",[serde_json::to_string(&envelope)?],|r|r.get(0))?;
    if !approved{return Err(ApiError(StatusCode::NOT_FOUND,"Release approval withdrawn"));}
    let trust=std::env::var("SWAN_RELEASE_PUBLIC_KEY").map_err(|_|bad("Release trust key unavailable"))?;
    let release:Release=envelope.verify(&public_key(&trust)?).map_err(|_|denied())?;
    if release.edition==Edition::Technician{s.user(&headers,false)?;}
    let bytes=tokio::fs::read(s.artifact_dir.join(format!("{id}.zip"))).await.map_err(|_|ApiError(StatusCode::NOT_FOUND,"Artifact unavailable"))?;
    Ok(([(header::CONTENT_TYPE,"application/zip"),(header::CONTENT_DISPOSITION,"attachment; filename=\"SwanRemoteSupport-company.zip\"")],bytes).into_response())
}
