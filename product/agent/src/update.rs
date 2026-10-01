//! Company-approved endpoint updates. No package code executes before metadata,
//! bytes, publisher and session exclusion have been checked.
use crate::*;
use anyhow::{ensure,Context,Result};
use serde::{Deserialize,Serialize};
use std::path::Path;

#[derive(Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt { release:SignedEnvelope, previous_sequence:u64, phase:String }

pub fn validate_release(state:&AgentState,envelope:&SignedEnvelope)->Result<Release> {
    let profile=state.company_profile()?;
    let release:Release=envelope.verify(&public_key(&state.bootstrap.release_public_key)?)?;
    release.validate(&state.bootstrap.edition,state.last_release_sequence,now())?;
    ensure!(release.channel==profile.update_channel,"Wrong update channel");
    Ok(release)
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
pub fn verify_package(state:&AgentState,envelope:&SignedEnvelope,path:&Path)->Result<()> {
    let release=validate_release(state,envelope)?;
    verify_compatibility(path.parent().context("Missing package directory")?,&release)?;
    ensure!(digest(std::fs::read(path)?).eq_ignore_ascii_case(&release.sha256),"Installer hash differs from signed metadata");
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

impl AgentState {
    #[cfg(windows)]
    pub fn recover_update(&mut self,directory:&Path)->Result<()> {
        let activity=activity_file(directory)?;fs2::FileExt::try_lock_exclusive(&activity).context("Recovery requires all sessions to close")?;
        let receipt_path=directory.join("pending-update.json");
        let receipt:Receipt=serde_json::from_slice(&std::fs::read(&receipt_path)?)?;
        let release:Release=receipt.release.verify(&public_key(&self.bootstrap.release_public_key)?)?;
        ensure!(release.edition==self.bootstrap.edition && release.product==PRODUCT && release.schema==SCHEMA,"Wrong recovery release");
        ensure!(self.last_release_sequence==receipt.previous_sequence || self.last_release_sequence==release.sequence,"Recovery sequence conflict");
        verify_installed(directory,&release)?;
        save_installed_metadata(directory,&receipt.release)?;
        // A crashed updater may have installed successfully or saved state before
        // removing the receipt. Exact signed executable identity proves either case.
        self.last_release_sequence=release.sequence;self.save(directory)?;
        std::fs::remove_file(receipt_path)?;Ok(())
    }
    pub async fn update(&mut self,directory:&Path,technician_token:Option<&str>)->Result<bool> {
        #[cfg(not(windows))] {let _=(directory,technician_token);anyhow::bail!("Endpoint updates require Windows");}
        #[cfg(windows)] {
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
            ensure!(release.edition!=Edition::Technician || (release.format=="exe" && release.installed_sha256.eq_ignore_ascii_case(&release.sha256)),"Technician updates require a portable EXE with matching installed identity");
            let folder=directory.join("updates").join(release.sequence.to_string());std::fs::create_dir_all(&folder)?;
            verify_compatibility(&folder,&release)?;
            let package=folder.join(format!("SwanRemoteSupport-install.{}",release.format));
            download(&release.artifact_url,&release.sha256,&package).await?;
            verify_publisher(&package,&release)?;
            let activity=activity_file(directory)?;
            fs2::FileExt::try_lock_exclusive(&activity).context("Update deferred while a session or technician app is active")?;
            // Re-read consent and enrollment immediately before installation.
            let mut latest=AgentState::load(directory)?;
            validate_release(&latest,&envelope)?;
            let receipt=Receipt{release:envelope,previous_sequence:latest.last_release_sequence,phase:"installing".into()};
            let mut options=std::fs::OpenOptions::new();options.create_new(true).write(true);
            use std::io::Write;
            let mut file=options.open(&receipt_path)?;file.write_all(&serde_json::to_vec(&receipt)?)?;file.sync_all()?;drop(file);
            if latest.bootstrap.edition==Edition::Technician {
                ensure!(release.format=="exe","Technician portable updates require EXE artifacts");
                let target=directory.join("SwanRemoteSupport-Technician.exe");
                if target.exists(){std::fs::copy(&target,directory.join("SwanRemoteSupport-Technician.previous.exe"))?;}
                std::fs::copy(&package,&target)?;
            }else{
                let package_for_install=package.clone();let format=release.format.clone();
                let status=tokio::task::spawn_blocking(move || ->Result<std::process::ExitStatus>{
                    let mut command=if format=="msi" {let mut c=std::process::Command::new("msiexec.exe");c.arg("/i").arg(&package_for_install).args(["/qn","/norestart"]);c}else{let mut c=std::process::Command::new(package_for_install);c.args(["--silent-install","printer=0"]);c};
                    Ok(command.status()?)
                }).await??;
                ensure!(matches!(status.code(),Some(0|3010)),"Installation failed; retain signed recovery receipt");
            }
            verify_installed(directory,&release)?;
            save_installed_metadata(directory,&receipt.release)?;
            latest.last_release_sequence=release.sequence;latest.save(directory)?;
            std::fs::remove_file(receipt_path)?;*self=latest;Ok(true)
        }
    }
}

#[cfg(windows)]
fn verify_installed(directory:&Path,release:&Release)->Result<()> {
    let target=if release.edition==Edition::Technician {directory.join("SwanRemoteSupport-Technician.exe")}else{
        std::path::PathBuf::from(std::env::var_os("ProgramFiles").context("ProgramFiles unavailable")?).join("Swan Remote Support/Swan Remote Support.exe")
    };
    ensure!(digest(std::fs::read(&target)?).eq_ignore_ascii_case(&release.installed_sha256),"Installed executable differs from signed release; recovery remains pending");
    let mut installed=release.clone();installed.sha256=release.installed_sha256.clone();verify_publisher(&target,&installed)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn update_lock_excludes_sessions_and_releases_on_close() {
        let folder=std::env::temp_dir().join(format!("swan-lock-{}",random_token()));
        let session=lock_session(&folder).unwrap();let update=activity_file(&folder).unwrap();
        assert!(fs2::FileExt::try_lock_exclusive(&update).is_err());drop(session);
        fs2::FileExt::try_lock_exclusive(&update).unwrap();assert!(lock_session(&folder).is_err());drop(update);
        assert!(lock_session(&folder).is_ok());std::fs::remove_dir_all(folder).unwrap();
    }
}
