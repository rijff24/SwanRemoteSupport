use anyhow::{ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use swan_protocol::*;
pub use swan_protocol as protocol;
pub mod update;
pub mod technician;

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Bootstrap {
    pub schema:u32,
    pub edition:Edition,
    pub company_id:String,
    pub management_url:String,
    pub profile_public_key:String,
    pub release_public_key:String,
}
impl Bootstrap {
    pub fn validate(&self)->Result<()> {
        ensure!(self.schema==SCHEMA && !self.company_id.is_empty(),"Invalid bootstrap");
        https_url(&self.management_url)?;public_key(&self.profile_public_key)?;
        if !self.release_public_key.is_empty(){public_key(&self.release_public_key)?;}
        Ok(())
    }
}

#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentState {
    pub bootstrap:Bootstrap,
    pub profile:SignedEnvelope,
    pub accepted_revision:u64,
    pub device_id:Option<String>,
    pub device_token:Option<String>,
    pub unattended_consent:bool,
    #[serde(default)]
    pub consent_revision:u64,
    pub last_release_sequence:u64,
}
impl AgentState {
    pub fn load(directory:&Path)->Result<Self> {
        let state=Self::load_for_refresh(directory)?;
        state.company_profile()?;
        Ok(state)
    }
    // Expired cached policy may fetch a replacement, but cannot authorize sessions.
    pub fn load_for_refresh(directory:&Path)->Result<Self> {
        let state:Self=serde_json::from_slice(&std::fs::read(directory.join("managed-state.json"))?)?;
        state.bootstrap.validate()?;
        state.cached_profile()?;
        Ok(state)
    }
    pub fn save(&self,directory:&Path)->Result<()> {
        self.save_internal(directory,None)
    }
    pub fn set_local_consent(&mut self,directory:&Path,enabled:bool)->Result<()> {
        ensure!(self.bootstrap.edition==Edition::Customer,"Customer consent is required");
        ensure!(self.device_id.is_some(),"Device enrollment required");
        ensure!(!enabled || self.company_profile()?.allow_unattended,"Company disallows unattended support");
        self.save_internal(directory,Some(enabled))?;
        *self=Self::load_for_refresh(directory)?;Ok(())
    }
    fn save_internal(&self,directory:&Path,consent_change:Option<bool>)->Result<()> {
        self.bootstrap.validate()?;self.cached_profile()?;
        std::fs::create_dir_all(directory)?;
        #[cfg(unix)] {use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(directory,std::fs::Permissions::from_mode(0o700))?;}
        let path=directory.join("managed-state.json");
        use fs2::FileExt;
        let lock=std::fs::OpenOptions::new().create(true).read(true).write(true).open(directory.join("state.lock"))?;
        lock.lock_exclusive()?;
        let mut saved=self.clone();
        if path.exists(){
            let existing:AgentState=serde_json::from_slice(&std::fs::read(&path)?)?;
            existing.bootstrap.validate()?;let existing_profile=existing.cached_profile()?;
            ensure!(existing.bootstrap.company_id==self.bootstrap.company_id,"Cannot change company ownership");
            ensure!(existing.bootstrap.edition==self.bootstrap.edition,"Cannot change installed edition");
            ensure!(existing.bootstrap.release_public_key==self.bootstrap.release_public_key,"Cannot silently replace the release trust key");
            ensure!(existing.bootstrap.profile_public_key==self.bootstrap.profile_public_key || existing_profile.next_profile_public_key.as_ref()==Some(&self.bootstrap.profile_public_key),"Profile key rotation requires the previously trusted signed profile");
            ensure!(existing.accepted_revision<=self.accepted_revision,"A newer profile has already been saved");
            saved.last_release_sequence=saved.last_release_sequence.max(existing.last_release_sequence);
            if let Some(id)=existing.device_id.as_ref(){
                ensure!(consent_change.is_some() || self.consent_revision<=existing.consent_revision,"Consent revision changes require explicit customer consent");
                ensure!(self.device_id.as_ref().map(|value|value==id).unwrap_or(true),"Cannot replace enrolled device identity");
                ensure!(self.device_token.as_ref().map(|value|Some(value)==existing.device_token.as_ref()).unwrap_or(true),"Cannot replace enrolled device credential");
                if self.device_id.is_none() || self.consent_revision<existing.consent_revision {saved.unattended_consent=existing.unattended_consent;}
                if self.consent_revision==existing.consent_revision && !existing.unattended_consent {saved.unattended_consent=false;}
                saved.consent_revision=existing.consent_revision;
                if let Some(enabled)=consent_change {
                    saved.consent_revision=existing.consent_revision.checked_add(1).context("Consent revision exhausted")?;
                    saved.unattended_consent=enabled;
                }
                saved.device_id=existing.device_id;saved.device_token=existing.device_token;
            }
        }
        let temporary=directory.join(format!("state-{}.tmp",random_token()));
        use std::io::Write;
        let mut options=std::fs::OpenOptions::new();options.write(true).create_new(true);
        #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
        let mut file=options.open(&temporary)?;file.write_all(&serde_json::to_vec_pretty(&saved)?)?;file.sync_all()?;
        drop(file);
        replace_state(&temporary,&path)?;Ok(())
    }
    pub fn company_profile(&self)->Result<CompanyProfile> {
        let p=self.cached_profile()?;
        p.validate(&self.bootstrap.company_id,self.accepted_revision,now())?;
        Ok(p)
    }
    /// Signed cached branding may still be displayed offline. This is never an
    /// authorization policy; company_profile() remains mandatory for access.
    pub fn display_profile(&self)->Result<CompanyProfile> {self.cached_profile()}
    fn cached_profile(&self)->Result<CompanyProfile> {
        let p:CompanyProfile=self.profile.verify(&public_key(&self.bootstrap.profile_public_key)?)?;
        // Expiration blocks use, but a previously signed rotation key remains a
        // trust anchor for obtaining fresh policy after a long offline period.
        p.validate(&self.bootstrap.company_id,self.accepted_revision,now().min(p.expires_at.saturating_sub(1)))?;
        Ok(p)
    }
    pub fn client(&self)->Result<reqwest::Client> {
        http_client(15,false)
    }
    pub fn endpoint(&self,path:&str)->Result<String> {
        self.bootstrap.validate()?;
        Ok(format!("{}/api/v1/{path}",self.bootstrap.management_url.trim_end_matches('/')))
    }
    pub async fn sync(&mut self)->Result<()> {
        let envelope:SignedEnvelope=self.client()?.get(self.endpoint("profile")?).send().await?.error_for_status()?.json().await?;
        let key=public_key(&self.bootstrap.profile_public_key)?;
        let profile:CompanyProfile=match envelope.verify(&key){
            Ok(p)=>p,
            Err(error)=>{
                let old=self.cached_profile()?;
                let next=old.next_profile_public_key.as_ref().context("Profile signature invalid and no trusted key rotation")?;
                let p=envelope.verify(&public_key(next)?)?;
                // Commit the new pin only after every validation succeeds below.
                let _:anyhow::Error=error;
                p
            }
        };
        profile.validate(&self.bootstrap.company_id,self.accepted_revision,now())?;
        if envelope.verify::<CompanyProfile>(&key).is_err(){
            self.bootstrap.profile_public_key=self.cached_profile()?.next_profile_public_key.context("Missing rotation key")?;
        }
        self.bootstrap.management_url=profile.management_url.clone();
        self.accepted_revision=profile.revision;self.profile=envelope;
        Ok(())
    }
    pub async fn enroll(&mut self,name:&str,rustdesk_id:&str,consent:bool)->Result<()> {
        ensure!(self.bootstrap.edition==Edition::Customer,"Only customer edition enrolls devices");
        let profile=self.company_profile()?;
        if let Some(device)=self.device_id.as_ref() {
            // Repair validates existing ownership instead of issuing new identity
            // or changing consent merely because setup was run again.
            let status:Value=self.client()?.get(self.endpoint("device/status")?).bearer_auth(self.device_token.as_ref().context("Missing enrolled credential")?).send().await?.error_for_status()?.json().await?;
            ensure!(status["company_id"].as_str()==Some(self.bootstrap.company_id.as_str()) && status["device_id"].as_str()==Some(device.as_str()) && status["rustdesk_id"].as_str()==Some(rustdesk_id),"Repair cannot replace company enrollment or transport identity");
            ensure!(matches!(status["state"].as_str(),Some("pending"|"approved")),"Device requires company administrator intervention");
            return Ok(());
        }
        ensure!(!consent || profile.allow_unattended,"Company has disabled unattended access");
        let result:Value=self.client()?.post(self.endpoint("enroll")?).json(&json!({"name":name,"rustdesk_id":rustdesk_id,"unattended_consent":consent})).send().await?.error_for_status()?.json().await?;
        ensure!(result["company_id"].as_str()==Some(self.bootstrap.company_id.as_str()),"Wrong company enrollment response");
        self.device_id=Some(result["device_id"].as_str().context("Missing device identity")?.into());
        self.device_token=Some(result["device_token"].as_str().context("Missing device credential")?.into());
        self.unattended_consent=consent;Ok(())
    }
    pub async fn consent(&mut self,enabled:bool)->Result<()> {
        // Local revoke takes effect even if the server is unavailable.
        if !enabled {self.unattended_consent=false;}
        ensure!(!enabled || self.company_profile()?.allow_unattended,"Company disallows unattended support");
        self.client()?.put(self.endpoint("device/consent")?).bearer_auth(self.device_token.as_ref().context("Not enrolled")?).json(&json!({"unattended":enabled})).send().await?.error_for_status()?;
        self.unattended_consent=enabled;Ok(())
    }
    pub async fn request_grant(&self,token:&str,device:&str,unattended:bool,key:&SigningKey)->Result<SignedEnvelope> {
        ensure!(self.bootstrap.edition==Edition::Technician,"Technician edition required");
        self.company_profile()?;
        let grant:SignedEnvelope=self.client()?.post(self.endpoint("grants")?).bearer_auth(token).json(&json!({"device_id":device,"proof_public_key":STANDARD.encode(key.verifying_key().as_bytes()),"unattended":unattended})).send().await?.error_for_status()?.json().await?;
        let parsed:SessionGrant=grant.verify(&public_key(&self.bootstrap.profile_public_key)?)?;
        parsed.validate(&self.bootstrap.company_id,device,&parsed.rustdesk_id,now())?;
        ensure!(parsed.proof_public_key==STANDARD.encode(key.verifying_key().as_bytes()) && parsed.unattended==unattended,"Grant differs from requested proof or consent mode");
        Ok(grant)
    }
    pub async fn claim(&self,login:&ManagedLogin,challenge:&str,peer:&str)->Result<Lease> {
        let activity=lock_session(&state_directory())?;
        ensure!(self.bootstrap.edition==Edition::Customer,"Customer edition required");
        let profile=self.company_profile()?;
        let grant:SessionGrant=login.grant.verify(&public_key(&self.bootstrap.profile_public_key)?)?;
        grant.validate(&self.bootstrap.company_id,self.device_id.as_ref().context("Not enrolled")?,peer,now())?;
        login.verify_proof(&grant,challenge)?;
        ensure!(!grant.unattended || (self.unattended_consent && profile.allow_unattended),"Unattended consent required");
        let result:Value=self.client()?.post(self.endpoint("grants/claim")?).bearer_auth(self.device_token.as_ref().context("Not enrolled")?).json(&json!({"grant":login.grant})).send().await?.error_for_status()?.json().await?;
        let expires=result["lease_until"].as_u64().context("Missing lease expiry")?;
        ensure!(expires>now() && expires<=now()+310,"Invalid session lease");
        Ok(Lease { grant_id:grant.grant_id,unattended:grant.unattended,expires_at:expires,last_renewed:now(),_activity:activity })
    }
    pub async fn renew(&self,lease:&mut Lease)->Result<()> {
        let profile=self.company_profile()?;
        ensure!(!lease.unattended || (self.unattended_consent && profile.allow_unattended),"Unattended consent or company permission revoked");
        let result:Value=self.client()?.post(self.endpoint(&format!("grants/{}/renew",lease.grant_id))?).bearer_auth(self.device_token.as_ref().context("Not enrolled")?).send().await?.error_for_status()?.json().await?;
        let expires=result["lease_until"].as_u64().context("Missing lease expiry")?;
        ensure!(expires>now() && expires<=now()+310,"Invalid session lease");
        lease.expires_at=expires;lease.last_renewed=now();Ok(())
    }
    pub async fn close(&self,grant_id:&str)->Result<()> {
        self.client()?.post(self.endpoint(&format!("grants/{grant_id}/close"))?).bearer_auth(self.device_token.as_ref().context("Not enrolled")?).send().await?.error_for_status()?;Ok(())
    }
}

fn replace_state(temporary:&Path,path:&Path)->Result<()> {
    #[cfg(windows)] {
        use std::os::windows::ffi::OsStrExt;
        use windows_sys::Win32::Storage::FileSystem::{MoveFileExW,MOVEFILE_REPLACE_EXISTING,MOVEFILE_WRITE_THROUGH};
        let from:Vec<u16>=temporary.as_os_str().encode_wide().chain(Some(0)).collect();
        let to:Vec<u16>=path.as_os_str().encode_wide().chain(Some(0)).collect();
        // Replace without deleting the valid state first; a crash retains old or new.
        if unsafe{MoveFileExW(from.as_ptr(),to.as_ptr(),MOVEFILE_REPLACE_EXISTING|MOVEFILE_WRITE_THROUGH)}==0 {return Err(std::io::Error::last_os_error().into());}
        Ok(())
    }
    #[cfg(not(windows))] {std::fs::rename(temporary,path)?;Ok(())}
}

pub struct Lease { pub grant_id:String,pub unattended:bool,pub expires_at:u64,pub last_renewed:u64,_activity:std::fs::File }
pub fn activity_file(directory:&Path)->Result<std::fs::File> {
    std::fs::create_dir_all(directory)?;
    Ok(std::fs::OpenOptions::new().create(true).read(true).write(true).open(directory.join("activity.lock"))?)
}
pub fn lock_session(directory:&Path)->Result<std::fs::File> {
    let file=activity_file(directory)?;fs2::FileExt::try_lock_shared(&file).context("An update is in progress")?;Ok(file)
}
pub fn state_directory()->PathBuf {
    if let Some(directory)=std::env::var_os("SWAN_STATE_DIR"){return directory.into();}
    #[cfg(windows)] {return PathBuf::from(std::env::var_os("PROGRAMDATA").unwrap_or_else(||"C:\\ProgramData".into())).join("SwanRemoteSupport");}
    #[cfg(not(windows))] {PathBuf::from("/var/lib/swan-remote-support")}
}
pub async fn bootstrap(input:Bootstrap)->Result<AgentState> {
    input.validate()?;
    let client=http_client(15,false)?;
    let profile:SignedEnvelope=client.get(format!("{}/api/v1/profile",input.management_url.trim_end_matches('/'))).send().await?.error_for_status()?.json().await?;
    let parsed:CompanyProfile=profile.verify(&public_key(&input.profile_public_key)?)?;
    parsed.validate(&input.company_id,0,now())?;
    Ok(AgentState {bootstrap:input,profile,accepted_revision:parsed.revision,device_id:None,device_token:None,unattended_consent:false,consent_revision:0,last_release_sequence:0})
}

pub fn http_client(timeout_seconds:u64,artifact_redirects:bool)->Result<reqwest::Client> {
    let builder=reqwest::Client::builder().timeout(std::time::Duration::from_secs(timeout_seconds)).https_only(true)
        .redirect(if artifact_redirects{reqwest::redirect::Policy::limited(5)}else{reqwest::redirect::Policy::none()});
    #[cfg(debug_assertions)]
    let builder=if let Some(path)=std::env::var_os("SWAN_TEST_CA_FILE") {
        builder.add_root_certificate(reqwest::Certificate::from_der(&std::fs::read(path)?)?)
    }else{builder};
    Ok(builder.build()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->AgentState {
        let key=SigningKey::from_bytes(&[7;32]);let public=STANDARD.encode(key.verifying_key().as_bytes());
        let profile=CompanyProfile{schema:1,company_id:"test-company".into(),revision:1,issued_at:now()-1000,expires_at:now()+3600,management_url:"https://support.example.com".into(),rendezvous:"support.example.com".into(),relay:"support.example.com".into(),transport_public_key:public.clone(),customer:Branding::default(),technician:Branding::default(),allow_unattended:true,updates_paused:true,rollout_percent:100,maintenance_start_utc:0,maintenance_end_utc:0,update_channel:"test".into(),next_profile_public_key:None};
        AgentState{bootstrap:Bootstrap{schema:1,edition:Edition::Customer,company_id:profile.company_id.clone(),management_url:profile.management_url.clone(),profile_public_key:public.clone(),release_public_key:public},profile:SignedEnvelope::sign(&profile,&key).unwrap(),accepted_revision:1,device_id:Some("device".into()),device_token:Some(random_token()),unattended_consent:true,consent_revision:0,last_release_sequence:0}
    }
    #[test]
    fn explicit_consent_changes_survive_older_refreshes() {
        let directory=std::env::temp_dir().join(format!("swan-consent-{}",random_token()));
        let mut state=fixture();state.save(&directory).unwrap();let old_allowed=state.clone();
        state.set_local_consent(&directory,false).unwrap();let old_revoked=state.clone();
        old_allowed.save(&directory).unwrap();assert!(!AgentState::load(&directory).unwrap().unattended_consent);
        state.set_local_consent(&directory,true).unwrap();old_revoked.save(&directory).unwrap();
        let current=AgentState::load(&directory).unwrap();assert!(current.unattended_consent);assert_eq!(current.consent_revision,2);
        state.set_local_consent(&directory,false).unwrap();current.save(&directory).unwrap();
        assert!(!AgentState::load(&directory).unwrap().unattended_consent);
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn stale_refresh_cannot_restore_consent_or_replay_update_state() {
        let directory=std::env::temp_dir().join(format!("swan-state-{}",random_token()));
        let mut state=fixture();state.save(&directory).unwrap();let stale=state.clone();
        state.unattended_consent=false;state.last_release_sequence=10;state.save(&directory).unwrap();
        stale.save(&directory).unwrap();let actual=AgentState::load(&directory).unwrap();
        assert!(!actual.unattended_consent);assert_eq!(actual.last_release_sequence,10);
        assert_eq!(actual.device_id,state.device_id);assert_eq!(actual.device_token,state.device_token);
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn stale_pre_enrollment_refresh_preserves_identity_and_trust() {
        let directory=std::env::temp_dir().join(format!("swan-enrollment-{}",random_token()));
        let enrolled=fixture();let mut before=enrolled.clone();before.device_id=None;before.device_token=None;before.unattended_consent=false;
        before.save(&directory).unwrap();enrolled.save(&directory).unwrap();before.save(&directory).unwrap();
        let current=AgentState::load(&directory).unwrap();assert_eq!(current.device_id,enrolled.device_id);assert_eq!(current.device_token,enrolled.device_token);assert!(current.unattended_consent);
        let mut replacement=current.clone();replacement.device_id=Some("different-device".into());assert!(replacement.save(&directory).is_err());
        let mut replacement=current.clone();replacement.device_token=Some(random_token());assert!(replacement.save(&directory).is_err());
        let mut replacement=current.clone();replacement.bootstrap.edition=Edition::Technician;assert!(replacement.save(&directory).is_err());
        let mut replacement=current.clone();replacement.bootstrap.release_public_key=STANDARD.encode(SigningKey::from_bytes(&[8;32]).verifying_key().as_bytes());assert!(replacement.save(&directory).is_err());
        let mut replacement=current.clone();let key=SigningKey::from_bytes(&[9;32]);replacement.bootstrap.profile_public_key=STANDARD.encode(key.verifying_key().as_bytes());replacement.profile=SignedEnvelope::sign(&current.company_profile().unwrap(),&key).unwrap();assert!(replacement.save(&directory).is_err());
        assert_eq!(AgentState::load(&directory).unwrap().device_token,enrolled.device_token);
        std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn trusted_profile_rotation_preserves_enrollment_and_rejects_rollback() {
        let directory=std::env::temp_dir().join(format!("swan-rotation-{}",random_token()));
        let mut state=fixture();let next=SigningKey::from_bytes(&[10;32]);let next_public=STANDARD.encode(next.verifying_key().as_bytes());
        let mut profile=state.company_profile().unwrap();profile.next_profile_public_key=Some(next_public.clone());
        state.profile=SignedEnvelope::sign(&profile,&SigningKey::from_bytes(&[7;32])).unwrap();state.save(&directory).unwrap();let old=state.clone();
        profile.revision+=1;profile.next_profile_public_key=None;profile.management_url="https://new-support.example.com".into();
        state.bootstrap.profile_public_key=next_public.clone();state.bootstrap.management_url=profile.management_url.clone();state.accepted_revision=profile.revision;state.profile=SignedEnvelope::sign(&profile,&next).unwrap();
        state.save(&directory).unwrap();let loaded=AgentState::load(&directory).unwrap();
        assert_eq!(loaded.bootstrap.profile_public_key,next_public);assert_eq!(loaded.bootstrap.management_url,profile.management_url);assert_eq!(loaded.device_id,old.device_id);assert_eq!(loaded.device_token,old.device_token);assert_eq!(loaded.unattended_consent,old.unattended_consent);
        assert!(old.save(&directory).is_err());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn expired_cache_only_allows_refresh_and_never_unsigned_policy() {
        let directory=std::env::temp_dir().join(format!("swan-expired-{}",random_token()));let mut state=fixture();
        let mut profile=state.company_profile().unwrap();profile.expires_at=now()-1;
        state.profile=SignedEnvelope::sign(&profile,&SigningKey::from_bytes(&[7;32])).unwrap();state.save(&directory).unwrap();
        assert!(AgentState::load(&directory).is_err());assert!(AgentState::load_for_refresh(&directory).is_ok());
        assert_eq!(AgentState::load_for_refresh(&directory).unwrap().display_profile().unwrap().customer.display_name,PRODUCT);
        state.profile.payload=STANDARD.encode(b"{}");std::fs::write(directory.join("managed-state.json"),serde_json::to_vec(&state).unwrap()).unwrap();
        assert!(AgentState::load_for_refresh(&directory).is_err());std::fs::remove_dir_all(directory).unwrap();
    }
    #[test]
    fn bootstrap_never_accepts_http_or_embedded_credentials() {
        let key=STANDARD.encode(SigningKey::from_bytes(&[1;32]).verifying_key().as_bytes());
        let mut b=Bootstrap{schema:1,edition:Edition::Customer,company_id:"a".into(),management_url:"https://support.example.com".into(),profile_public_key:key,release_public_key:String::new()};
        assert!(b.validate().is_ok());b.management_url="http://support.example.com".into();assert!(b.validate().is_err());
        b.management_url="https://secret@example.com".into();assert!(b.validate().is_err());
    }
}
