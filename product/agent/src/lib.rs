use anyhow::{ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::SigningKey;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use swan_protocol::*;
pub use swan_protocol as protocol;
pub mod update;

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
        let profile:CompanyProfile=state.profile.verify(&public_key(&state.bootstrap.profile_public_key)?)?;
        ensure!(profile.company_id==state.bootstrap.company_id && profile.revision>=state.accepted_revision,"Invalid cached company profile");
        Ok(state)
    }
    pub fn save(&self,directory:&Path)->Result<()> {
        std::fs::create_dir_all(directory)?;
        #[cfg(unix)] {use std::os::unix::fs::PermissionsExt; std::fs::set_permissions(directory,std::fs::Permissions::from_mode(0o700))?;}
        let path=directory.join("managed-state.json");
        use fs2::FileExt;
        let lock=std::fs::OpenOptions::new().create(true).read(true).write(true).open(directory.join("state.lock"))?;
        lock.lock_exclusive()?;
        let mut saved=self.clone();
        if path.exists(){
            let existing:AgentState=serde_json::from_slice(&std::fs::read(&path)?)?;
            ensure!(existing.bootstrap.company_id==self.bootstrap.company_id,"Cannot change company ownership");
            ensure!(existing.accepted_revision<=self.accepted_revision,"A newer profile has already been saved");
            saved.last_release_sequence=saved.last_release_sequence.max(existing.last_release_sequence);
            if existing.device_id.is_some() && !existing.unattended_consent {saved.unattended_consent=false;}
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
        let p:CompanyProfile=self.profile.verify(&public_key(&self.bootstrap.profile_public_key)?)?;
        p.validate(&self.bootstrap.company_id,self.accepted_revision,now())?;
        Ok(p)
    }
    pub fn client(&self)->Result<reqwest::Client> {
        Ok(reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).redirect(reqwest::redirect::Policy::none()).https_only(true).build()?)
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
                let old=self.company_profile()?;
                let next=old.next_profile_public_key.as_ref().context("Profile signature invalid and no trusted key rotation")?;
                let p=envelope.verify(&public_key(next)?)?;
                // Commit the new pin only after every validation succeeds below.
                let _:anyhow::Error=error;
                p
            }
        };
        profile.validate(&self.bootstrap.company_id,self.accepted_revision,now())?;
        if envelope.verify::<CompanyProfile>(&key).is_err(){
            self.bootstrap.profile_public_key=self.company_profile()?.next_profile_public_key.context("Missing rotation key")?;
        }
        self.bootstrap.management_url=profile.management_url.clone();
        self.accepted_revision=profile.revision;self.profile=envelope;
        Ok(())
    }
    pub async fn enroll(&mut self,name:&str,rustdesk_id:&str,consent:bool)->Result<()> {
        ensure!(self.bootstrap.edition==Edition::Customer,"Only customer edition enrolls devices");
        ensure!(self.device_id.is_none(),"Already enrolled");
        let profile=self.company_profile()?;
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
        let grant:SignedEnvelope=self.client()?.post(self.endpoint("grants")?).bearer_auth(token).json(&json!({"device_id":device,"proof_public_key":STANDARD.encode(key.verifying_key().as_bytes()),"unattended":unattended})).send().await?.error_for_status()?.json().await?;
        let parsed:SessionGrant=grant.verify(&public_key(&self.bootstrap.profile_public_key)?)?;
        parsed.validate(&self.bootstrap.company_id,device,&parsed.rustdesk_id,now())?;
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
        ensure!(!lease.unattended || self.unattended_consent,"Unattended consent revoked");
        let result:Value=self.client()?.post(self.endpoint(&format!("grants/{}/renew",lease.grant_id))?).bearer_auth(self.device_token.as_ref().context("Not enrolled")?).send().await?.error_for_status()?.json().await?;
        let expires=result["lease_until"].as_u64().context("Missing lease expiry")?;
        ensure!(expires>now() && expires<=now()+310,"Invalid session lease");
        lease.expires_at=expires;lease.last_renewed=now();Ok(())
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
    let client=reqwest::Client::builder().timeout(std::time::Duration::from_secs(15)).redirect(reqwest::redirect::Policy::none()).https_only(true).build()?;
    let profile:SignedEnvelope=client.get(format!("{}/api/v1/profile",input.management_url.trim_end_matches('/'))).send().await?.error_for_status()?.json().await?;
    let parsed:CompanyProfile=profile.verify(&public_key(&input.profile_public_key)?)?;
    parsed.validate(&input.company_id,0,now())?;
    Ok(AgentState {bootstrap:input,profile,accepted_revision:parsed.revision,device_id:None,device_token:None,unattended_consent:false,last_release_sequence:0})
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn bootstrap_never_accepts_http_or_embedded_credentials() {
        let key=STANDARD.encode(SigningKey::from_bytes(&[1;32]).verifying_key().as_bytes());
        let mut b=Bootstrap{schema:1,edition:Edition::Customer,company_id:"a".into(),management_url:"https://support.example.com".into(),profile_public_key:key,release_public_key:String::new()};
        assert!(b.validate().is_ok());b.management_url="http://support.example.com".into();assert!(b.validate().is_err());
        b.management_url="https://secret@example.com".into();assert!(b.validate().is_err());
    }
}
