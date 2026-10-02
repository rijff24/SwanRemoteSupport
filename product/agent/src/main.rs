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
        #[cfg(windows)]
        "remove-configuration-task"|"restore-configuration-task"=>{
            let executable=std::env::current_exe()?;
            let installed_directory=executable.parent().context("Missing installed agent directory")?;
            ensure!(executable.file_name().is_some_and(|name|name.to_string_lossy().eq_ignore_ascii_case("swan-agent.exe")),"Uninstall cleanup requires the installed configuration agent");
            // MSI invokes the installed binary under SYSTEM before removing it.
            // The script checks the exact task executable, arguments and owner.
            let (name,contents)=if command=="remove-configuration-task" {
                ("Remove-Configuration.ps1",include_str!("../../../deployment/windows/Remove-Configuration.ps1"))
            }else{("Restore-Configuration.ps1",include_str!("../../../deployment/windows/Restore-Configuration.ps1"))};
            let script=installed_directory.join(name);
            std::fs::write(&script,contents)?;
            let powershell=PathBuf::from(std::env::var_os("SystemRoot").context("Missing Windows directory")?).join("System32/WindowsPowerShell/v1.0/powershell.exe");
            let status=std::process::Command::new(powershell).args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(&script).arg("-Directory").arg(installed_directory).status()?;
            std::fs::remove_file(&script)?;
            ensure!(status.success(),"Company configuration task cleanup failed");
        }
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
            // Reload after refresh: another process may have changed consent.
            // Server revisions reject delayed requests from an older choice.
            let mut current=AgentState::load_for_refresh(&directory)?;
            if current.bootstrap.edition==Edition::Customer && current.device_id.is_some() && !current.unattended_consent {
                if let Err(error)=current.consent(false).await {eprintln!("Consent revocation synchronization deferred: {error}");}
            }
            if state.bootstrap.edition==Edition::Customer {match state.update(&directory,None).await {Ok(true)=>return Ok(()),Ok(false)=>{},Err(error)=>eprintln!("Update deferred: {error}")}}
        },Err(error)=>eprintln!("Configuration unavailable: {error}")};tokio::time::sleep(std::time::Duration::from_secs(300)).await;}}
        "update"=>{
            let mut state=AgentState::load_for_refresh(&directory)?;state.sync().await?;state.save(&directory)?;
            let token=std::env::var("SWAN_TECHNICIAN_TOKEN").ok();
            println!("Update handed off: {}",state.update(&directory,token.as_deref()).await?);
        }
        #[cfg(windows)]
        "resume-update"=>{
            let state=AgentState::load_for_refresh(&directory)?;
            println!("Update recovery handed off: {}",state.resume_pending_update_online(&directory,true).await?);
        }
        #[cfg(windows)]
        "apply-update"=>{
            let update_directory=directory.clone();
            let customer=AgentState::load_for_refresh(&directory)?.bootstrap.edition==Edition::Customer;
            let outcome=tokio::task::spawn_blocking(move ||->anyhow::Result<bool>{
                let mut state=AgentState::load_for_refresh(&update_directory)?;
                state.apply_pending_update(&update_directory)
            }).await.context("Update helper task failed").and_then(|result|result);
            if customer && (matches!(&outcome,Ok(true)) || outcome.is_err()) {
                let script=directory.join("Restart-Configuration.ps1");
                std::fs::write(&script,include_str!("../../../deployment/windows/Restart-Configuration.ps1"))?;
                let status=std::process::Command::new("powershell.exe").args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(script).arg("-Directory").arg(&directory).status()?;
                if !status.success(){
                    if let Err(error)=&outcome {eprintln!("Update remains pending: {error:#}");}
                    bail!("Configuration task restart failed; inspect the protected update log and resume recovery");
                }
            }
            let installed=outcome?;
            if !customer && installed {
                use std::os::windows::process::CommandExt;
                // Installation verified the pinned executable and publisher.
                // Restart with company state only; technicians sign in again.
                std::process::Command::new(directory.join("SwanRemoteSupport-Technician.exe"))
                    .env("SWAN_STATE_DIR",&directory).env_remove("SWAN_TECHNICIAN_TOKEN")
                    .env_remove("SWAN_SESSION_GRANT").env_remove("SWAN_SESSION_PROOF_KEY")
                    .creation_flags(0x08000000).stdin(std::process::Stdio::null())
                    .stdout(std::process::Stdio::null()).stderr(std::process::Stdio::null()).spawn()
                    .context("Updated technician application could not restart")?;
            }
            println!("Installed release verified: {}",installed);
        }
        #[cfg(windows)]
        "record-installation"=>{
            let file=args.get(2).context("Usage: swan-agent record-installation release.json")?;
            let envelope:SignedEnvelope=serde_json::from_slice(&std::fs::read(file)?)?;
            let mut state=AgentState::load(&directory)?;state.record_installation(&directory,&envelope,args.iter().any(|value|value=="--repair"))?;
            println!("Installed application and agent identities verified; release sequence recorded.");
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
            if args.iter().any(|value|value=="--repair") {swan_agent::update::verify_repair_package(&state,&directory,&envelope,std::path::Path::new(package))?;}
            else{swan_agent::update::verify_package(&state,&directory,&envelope,std::path::Path::new(package))?;}
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
