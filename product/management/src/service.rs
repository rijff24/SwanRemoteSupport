use anyhow::ensure;
use argon2::{password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString}, Argon2};
use axum::{extract::{DefaultBodyLimit, Path, State}, http::{header, HeaderMap, StatusCode}, response::{Html, IntoResponse, Response}, routing::{get, post, put}, Json, Router};
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
        std::fs::create_dir_all(directory)?;
        #[cfg(unix)] { use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(directory,std::fs::Permissions::from_mode(0o700))?; }
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
        ensure!(version<=1,"Database was created by a newer server; restore the pre-upgrade backup rather than downgrading it");
        db.execute_batch(include_str!("schema.sql"))?;
        let artifact_dir=directory.join("artifacts");std::fs::create_dir_all(&artifact_dir)?;
        Ok(Self {db:Mutex::new(db),key:SigningKey::from_bytes(&key_bytes),setup_hash,limits:Mutex::new(HashMap::new()),artifact_dir})
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
        .route("/api/v1/releases/{id}/approve",post(approve_release))
        .route("/api/v1/device/update",get(device_update))
        .route("/api/v1/user/update",get(technician_update))
        .route("/api/v1/workers",post(create_worker))
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
async fn device_update(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    let device=s.device(&headers)?;select_update(&s,&device,Edition::Customer)
}
async fn technician_update(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    let user=s.user(&headers,false)?;select_update(&s,&user,Edition::Technician)
}
fn select_update(s:&Store,device:&str,edition:Edition)->ApiResult {
    let profile=s.profile()?;
    let cohort=u8::from_str_radix(&digest(&device)[..2],16).map_err(|_|bad("Invalid device"))? as u32 * 100 / 256;
    if profile.updates_paused || cohort>=profile.rollout_percent as u32 || !maintenance_open(profile.maintenance_start_utc,profile.maintenance_end_utc,now()) {return Ok(Json(Value::Null));}
    let db=s.db.lock().unwrap();let mut stmt=db.prepare("SELECT body,envelope FROM releases WHERE approved=1")?;
    let rows=stmt.query_map([],|r|Ok((r.get::<_,String>(0)?,r.get::<_,String>(1)?)))?.collect::<Result<Vec<_>,_>>()?;
    let mut best:Option<(u64,Value)>=None;
    for (body,envelope) in rows {let release:Release=serde_json::from_str(&body)?;
        if release.edition==edition && release.channel==profile.update_channel && release.expires_at>now() && best.as_ref().map(|b|b.0<release.sequence).unwrap_or(true) {best=Some((release.sequence,serde_json::from_str(&envelope)?));}}
    Ok(Json(best.map(|v|v.1).unwrap_or(Value::Null)))
}
async fn create_worker(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    let user=s.user(&headers,true)?;let credential=random_token();let id=Uuid::new_v4().to_string();
    s.db.lock().unwrap().execute("INSERT INTO workers(id,token_hash) VALUES(?1,?2)",params![id,digest(&credential)])?;
    s.audit(&user,"worker.created",&id)?;Ok(Json(json!({"worker_id":id,"worker_token":credential})))
}
#[derive(Deserialize)]
struct BuildRequest { release_id:String }
async fn create_build(State(s):State<Shared>,headers:HeaderMap,Json(input):Json<BuildRequest>)->ApiResult {
    let actor=s.user(&headers,true)?;let mut profile=s.profile()?;
    profile.issued_at=now();profile.expires_at=now()+7*86400;
    let envelope:String=s.db.lock().unwrap().query_row("SELECT envelope FROM releases WHERE id=?1 AND approved=1",[&input.release_id],|r|r.get(0))?;
    let id=Uuid::new_v4().to_string();let body=json!({"release":serde_json::from_str::<Value>(&envelope)?,"profile":SignedEnvelope::sign(&profile,&s.key)?,"profile_public_key":STANDARD.encode(s.key.verifying_key().as_bytes()),"release_public_key":std::env::var("SWAN_RELEASE_PUBLIC_KEY").map_err(|_|bad("Release key not configured"))?});
    s.db.lock().unwrap().execute("INSERT INTO builds(id,body,state) VALUES(?1,?2,'queued')",params![id,body.to_string()])?;s.audit(&actor,"build.queued",&id)?;Ok(Json(json!({"id":id})))
}
async fn builds(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    s.user(&headers,true)?;let db=s.db.lock().unwrap();let mut stmt=db.prepare("SELECT id,state,result FROM builds ORDER BY rowid DESC LIMIT 100")?;
    let rows=stmt.query_map([],|r|Ok(json!({"id":r.get::<_,String>(0)?,"state":r.get::<_,String>(1)?,"result":r.get::<_,String>(2)?})))?.collect::<Result<Vec<_>,_>>()?;Ok(Json(json!(rows)))
}
async fn claim_build(State(s):State<Shared>,headers:HeaderMap)->ApiResult {
    s.worker(&headers)?;let worker=digest(token(&headers)?);
    let mut db=s.db.lock().unwrap();let tx=db.transaction()?;
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
    // Verify content without extracting any paths. Download bundles cannot inject executables or setup scripts.
    let mut zip=zip::ZipArchive::new(std::io::Cursor::new(&bytes)).map_err(|_|bad("Invalid ZIP artifact"))?;
    if zip.len()!=9 {return Err(bad("Unexpected bundle contents"));}
    use std::io::Read;
    let mut read=|name:&str|->Result<Vec<u8>,ApiError>{
        let file=zip.by_name(name).map_err(|_|bad("Missing bundle file"))?;
        if file.size()>512*1024*1024{return Err(bad("Bundle entry too large"));}
        let mut output=Vec::new();file.take(512*1024*1024+1).read_to_end(&mut output).map_err(|_|bad("Unreadable bundle"))?;
        if output.len()>512*1024*1024{return Err(bad("Bundle entry too large"));}Ok(output)
    };
    if digest(read(&format!("SwanRemoteSupport-install.{}",release.format))?)!=release.sha256.to_lowercase() || digest(read("swan-agent.exe")?)!=release.agent_sha256.to_lowercase(){return Err(bad("Bundle binary hash mismatch"));}
    for (name,expected) in [
        ("Install-Company.ps1",include_bytes!("../../../deployment/windows/Install-Company.ps1").as_slice()),
        ("Open-Technician.ps1",include_bytes!("../../../deployment/windows/Open-Technician.ps1").as_slice()),
        ("Verify-Package.ps1",include_bytes!("../../../deployment/windows/Verify-Package.ps1").as_slice()),
        ("LICENSE.txt",include_bytes!("../../../LICENCE").as_slice())
    ] {if read(name)?!=bundle_text(expected)?{return Err(bad("Bundle script or license mismatch"));}}
    let profile:SignedEnvelope=serde_json::from_slice(&read("company-profile.json")?)?;
    if serde_json::to_value(&profile)?!=job["profile"] {return Err(bad("Wrong company profile"));}
    let copied_release:SignedEnvelope=serde_json::from_slice(&read("release.json")?)?;
    if serde_json::to_value(&copied_release)?!=job["release"] {return Err(bad("Wrong release metadata"));}
    let bootstrap:swan_agent::Bootstrap=serde_json::from_slice(&read("bootstrap.json")?)?;
    let p:CompanyProfile=profile.verify(&s.key.verifying_key()).map_err(|_|denied())?;
    if bootstrap.company_id!=p.company_id || bootstrap.edition!=release.edition || bootstrap.management_url!=p.management_url || bootstrap.profile_public_key!=STANDARD.encode(s.key.verifying_key().as_bytes()) || bootstrap.release_public_key!=trust {return Err(bad("Bootstrap trust mismatch"));}
    drop(read);drop(zip);
    let hash=digest(&bytes);
    tokio::task::spawn_blocking(move || persist_verified_artifact(&s,&id,&worker,&bytes))
        .await.map_err(|_|bad("Artifact persistence failed"))??;
    Ok(Json(json!({"sha256":hash})))
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
    let trust=std::env::var("SWAN_RELEASE_PUBLIC_KEY").map_err(|_|bad("Release trust key unavailable"))?;
    let release:Release=envelope.verify(&public_key(&trust)?).map_err(|_|denied())?;
    if release.edition==Edition::Technician{s.user(&headers,false)?;}
    let bytes=tokio::fs::read(s.artifact_dir.join(format!("{id}.zip"))).await.map_err(|_|ApiError(StatusCode::NOT_FOUND,"Artifact unavailable"))?;
    Ok(([(header::CONTENT_TYPE,"application/zip"),(header::CONTENT_DISPOSITION,"attachment; filename=\"SwanRemoteSupport-company.zip\"")],bytes).into_response())
}
