use anyhow::{bail, ensure, Context, Result};
use ed25519_dalek::SigningKey;
use rand::rngs::OsRng;
use serde_json::json;
use std::path::PathBuf;
use swan_agent::*;
use swan_protocol::*;

#[tokio::main]
async fn main()->Result<()> {
    let args:Vec<String>=std::env::args().collect();
    let directory=state_directory();
    let command=args.get(1).map(String::as_str).unwrap_or("help");
    match command {
        "setup"=>{
            let file=args.get(2).context("Usage: swan-agent setup bootstrap.json --accept-company")?;
            ensure!(args.iter().any(|a|a=="--accept-company"),"Review the company HTTPS domain and trust key; pass --accept-company only after confirming them");
            ensure!(!directory.join("managed-state.json").exists(),"Already configured; refuse to silently change company ownership");
            let input:Bootstrap=serde_json::from_slice(&std::fs::read(file)?)?;
            println!("Connecting to company domain {}",input.management_url);
            bootstrap(input).await?.save(&directory)?;
            println!("Company verified and profile pinned.");
        }
        "enroll"=>{
            let name=args.get(2).context("Usage: swan-agent enroll COMPUTER_NAME RUSTDESK_ID [--unattended-consent]")?;
            let peer=args.get(3).context("Missing RustDesk ID")?;
            let mut state=AgentState::load(&directory)?;
            state.enroll(name,peer,args.iter().any(|a|a=="--unattended-consent")).await?;state.save(&directory)?;
            println!("Device enrolled. Company approval is required before remote access.");
        }
        "verify-bootstrap"=>{
            let file=args.get(2).context("Usage: swan-agent verify-bootstrap bootstrap.json")?;
            let input:Bootstrap=serde_json::from_slice(&std::fs::read(file)?)?;
            AgentState::load(&directory)?.validate_installer_bootstrap(&input)?;
            println!("Installer company and trust match existing configuration.");
        }
        "sync"=>{let mut state=AgentState::load_for_refresh(&directory)?;state.sync().await?;state.save(&directory)?;println!("Profile synchronized.");}
        "watch"=>{loop {match AgentState::load_for_refresh(&directory){Ok(mut state)=>{
            match state.sync().await{Ok(())=>state.save(&directory)?,Err(error)=>eprintln!("Profile refresh failed: {error}")};
            if state.bootstrap.edition==Edition::Customer {if let Err(error)=state.update(&directory,None).await {eprintln!("Update deferred: {error}");}}
        },Err(error)=>eprintln!("Configuration unavailable: {error}")};tokio::time::sleep(std::time::Duration::from_secs(300)).await;}}
        "update"=>{
            let mut state=AgentState::load_for_refresh(&directory)?;state.sync().await?;state.save(&directory)?;
            let token=std::env::var("SWAN_TECHNICIAN_TOKEN").ok();
            println!("Update installed: {}",state.update(&directory,token.as_deref()).await?);
        }
        #[cfg(windows)]
        "recover-update"=>{let mut state=AgentState::load_for_refresh(&directory)?;state.recover_update(&directory)?;println!("Installed release verified and update state recovered.");}
        #[cfg(windows)]
        "verify-installed"=>{
            let state=AgentState::load_for_refresh(&directory)?;
            swan_agent::update::verify_installed_metadata(&state,&directory)?;
            println!("Installed executable and pinned release publisher verified.");
        }
        #[cfg(windows)]
        "verify-package"=>{
            let state=AgentState::load(&directory)?;
            let metadata=args.get(2).context("Usage: swan-agent verify-package release.json INSTALLER")?;
            let package=args.get(3).context("Missing installer path")?;
            let envelope:SignedEnvelope=serde_json::from_slice(&std::fs::read(metadata)?)?;
            swan_agent::update::verify_package(&state,&envelope,std::path::Path::new(package))?;
            println!("Installer metadata, bytes and publisher verified.");
        }
        "revoke-unattended"=>{
            let mut state=AgentState::load_for_refresh(&directory)?;state.set_local_consent(&directory,false)?;
            match state.consent(false).await {Ok(())=>println!("Unattended access revoked locally and on the server."),Err(_)=>println!("Unattended access revoked locally. Server synchronization will retry when available.")}
        }
        "allow-unattended"=>{
            ensure!(args.iter().any(|value|value=="--confirm-unattended"),"Explicit --confirm-unattended consent is required");
            let mut state=AgentState::load(&directory)?;
            state.consent(true).await?;state.set_local_consent(&directory,true)?;
            println!("Unattended support enabled by explicit customer consent.");
        }
        "login"=>{
            let state=AgentState::load(&directory)?;
            ensure!(state.bootstrap.edition==Edition::Technician,"Technician edition required");
            let username=args.get(2).context("Usage: swan-agent login USERNAME (password and code from SWAN_LOGIN_PASSWORD and SWAN_LOGIN_TOTP)")?;
            let password=std::env::var("SWAN_LOGIN_PASSWORD").context("Set SWAN_LOGIN_PASSWORD for this process only")?;
            let code=std::env::var("SWAN_LOGIN_TOTP").context("Set SWAN_LOGIN_TOTP for this process only")?;
            let token=state.technician_login(username,&password,&code).await?;
            // Explicit CLI exchange for automation. Never write tokens to logs or persistent profile files.
            println!("{}",json!({"token":token}));
        }
        "devices"=>{
            let state=AgentState::load(&directory)?;
            let token=std::env::var("SWAN_TECHNICIAN_TOKEN").context("Set authenticated technician token")?;
            let devices=state.technician_inventory(&token).await?;
            println!("{}",serde_json::to_string_pretty(&devices)?);
        }
        "history"=>{
            let state=AgentState::load(&directory)?;
            let token=std::env::var("SWAN_TECHNICIAN_TOKEN").context("Set authenticated technician token")?;
            println!("{}",serde_json::to_string_pretty(&state.technician_history(&token).await?)?);
        }
        "logout"=>{
            let state=AgentState::load_for_refresh(&directory)?;
            let token=std::env::var("SWAN_TECHNICIAN_TOKEN").context("Set authenticated technician token")?;
            state.technician_logout(&token).await?;println!("Technician session signed out.");
        }
        "connect"=>{
            let state=AgentState::load(&directory)?;let device=args.get(2).context("Usage: swan-agent connect DEVICE_ID APP_EXE [--unattended]")?;
            let executable=PathBuf::from(args.get(3).context("Missing app executable")?);
            ensure!(executable.is_file(),"App executable not found");
            let token=std::env::var("SWAN_TECHNICIAN_TOKEN").context("Authenticated technician token required")?;
            let proof=SigningKey::generate(&mut OsRng);
            let envelope=state.request_grant(&token,device,args.iter().any(|a|a=="--unattended"),&proof).await?;
            let grant:SessionGrant=envelope.verify(&public_key(&state.bootstrap.profile_public_key)?)?;
            let mut child=std::process::Command::new(executable).arg("--connect").arg(&grant.rustdesk_id)
                .env("SWAN_STATE_DIR",&directory).env("SWAN_SESSION_GRANT",serde_json::to_string(&envelope)?)
                .env("SWAN_SESSION_PROOF_KEY",hex::encode(proof.to_bytes())).spawn()?;
            ensure!(child.wait()?.success(),"Technician application exited unsuccessfully");
        }
        _=>{println!("Swan Remote Support configuration agent\nCommands: setup, verify-bootstrap, enroll, sync, watch, update, revoke-unattended, allow-unattended --confirm-unattended, login, devices, history, logout, connect\nState directory: {}",directory.display());if command!="help"{bail!("Unknown command");}}
    }
    Ok(())
}
