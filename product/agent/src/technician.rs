//! HTTPS technician operations. Credentials and session tokens are never saved.
use crate::{protocol::*, AgentState};
use anyhow::{ensure, Context, Result};
use ed25519_dalek::SigningKey;
use serde_json::{json, Value};

impl AgentState {
    pub async fn technician_login(&self, username:&str, password:&str, code:&str)->Result<String> {
        ensure!(self.bootstrap.edition==Edition::Technician,"Technician edition required");
        self.company_profile()?;
        ensure!(!username.is_empty() && username.len()<=100 && password.len()<=256 && code.len()==6 && code.bytes().all(|b|b.is_ascii_digit()),"Invalid login input");
        let response:Value=self.client()?.post(self.endpoint("login")?).json(&json!({"username":username,"password":password,"totp_code":code})).send().await?.error_for_status()?.json().await?;
        Ok(response["token"].as_str().context("Missing authenticated token")?.to_owned())
    }
    pub async fn technician_inventory(&self, token:&str)->Result<Value> {
        ensure!(self.bootstrap.edition==Edition::Technician,"Technician edition required");
        self.company_profile()?;
        Ok(self.client()?.get(self.endpoint("devices")?).bearer_auth(token).send().await?.error_for_status()?.json().await?)
    }
    pub async fn technician_history(&self, token:&str)->Result<Value> {
        ensure!(self.bootstrap.edition==Edition::Technician,"Technician edition required");
        self.company_profile()?;
        Ok(self.client()?.get(self.endpoint("sessions")?).bearer_auth(token).send().await?.error_for_status()?.json().await?)
    }
    pub async fn technician_logout(&self, token:&str)->Result<()> {
        ensure!(self.bootstrap.edition==Edition::Technician,"Technician edition required");
        self.client()?.post(self.endpoint("logout")?).bearer_auth(token).send().await?.error_for_status()?;
        Ok(())
    }
    pub async fn technician_ticket(&self, token:&str, device:&str, unattended:bool, key:&SigningKey)->Result<(SignedEnvelope,SessionGrant)> {
        let envelope=self.request_grant(token,device,unattended,key).await?;
        let grant:SessionGrant=envelope.verify(&public_key(&self.bootstrap.profile_public_key)?)?;
        Ok((envelope,grant))
    }
}
