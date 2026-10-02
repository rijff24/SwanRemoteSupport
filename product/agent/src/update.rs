//! Company-approved endpoint updates. No package code executes before metadata,
//! bytes, publisher and session exclusion have been checked.
use crate::*;
use anyhow::{ensure,Context,Result};
use serde::{Deserialize,Serialize};
use std::path::Path;

#[derive(Clone,Serialize,Deserialize)]
#[serde(deny_unknown_fields)]
struct Receipt {
    release:SignedEnvelope,
    previous_sequence:u64,
    phase:String,
    #[serde(default)]
    previous_release:Option<SignedEnvelope>,
    #[serde(default)]
    rollback_protocol:u32,
}

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
    ensure!(matches!(phase,"installing"|"rolling_back"),"Unknown update recovery phase");
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
    }else{
        let release=validate_release(state,envelope)?;
        ensure!(release.sequence>failed_release_sequence(state,directory)?,"Failed release is quarantined; approve a newer release");
        Ok(release)
    }
}

#[cfg(any(windows,test))]
fn validate_pending_receipt(state:&AgentState,receipt:&Receipt)->Result<Release> {
    ensure!(receipt.rollback_protocol<=1,"Unsupported pending rollback protocol");
    let release=validate_recovery_release(state,&receipt.release,receipt.previous_sequence,&receipt.phase)?;
    if let Some(previous)=receipt.previous_release.as_ref(){
        let old=validate_previous_release(state,previous,receipt.previous_sequence)?;
        ensure!(old.format==release.format,"Installer format migration requires explicit setup");
    }
    if receipt.phase=="rolling_back" {
        ensure!(state.last_release_sequence==receipt.previous_sequence,"Cannot roll back a committed newer installation");
        let previous=receipt.previous_release.as_ref().context("Rollback requires previous release metadata")?;
        ensure!(receipt.rollback_protocol==1 && validate_previous_release(state,previous,receipt.previous_sequence)?.rollback_protocol==1,"Previous release cannot enforce rollback quarantine");
    }
    Ok(release)
}

#[cfg(any(windows,test))]
fn validate_completion_receipt(state:&AgentState,receipt:&Receipt)->Result<Release> {
    ensure!(receipt.phase=="installing","Use resume-update to finish pending rollback");
    validate_pending_receipt(state,receipt)
}

fn failed_release_sequence(state:&AgentState,directory:&Path)->Result<u64> {
    let path=directory.join("failed-update.json");
    let bytes=match std::fs::read(path) {
        Ok(bytes)=>bytes,
        Err(error) if error.kind()==std::io::ErrorKind::NotFound=>return Ok(0),
        Err(error)=>return Err(error).context("Cannot read failed update quarantine"),
    };
    let receipt:Receipt=serde_json::from_slice(&bytes)?;
    ensure!(receipt.phase=="rolled_back" && receipt.rollback_protocol==1,"Invalid failed update quarantine");
    let release:Release=receipt.release.verify(&public_key(&state.bootstrap.release_public_key)?)?;
    release.validate(&state.bootstrap.edition,receipt.previous_sequence,now().min(release.expires_at.saturating_sub(1)))?;
    let previous=receipt.previous_release.context("Failed update lacks previous release")?;
    ensure!(validate_previous_release(state,&previous,receipt.previous_sequence)?.rollback_protocol==1,"Previous release does not support quarantine");
    Ok(release.sequence)
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
    let path=path.to_owned();
    tokio::task::spawn_blocking(move ||publish_download(&path,&bytes)).await.context("Download publication task failed")?
}

fn publish_download(path:&Path,bytes:&[u8])->Result<()> {
    use std::io::Write;
    // A crash or a concurrent reader must see the previous complete download,
    // never a truncated installer or helper executable.
    let temporary=path.parent().context("Missing download directory")?.join(format!("swan-download-{}.tmp",random_token()));
    let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(&temporary)?;
    let result=(||->Result<()> {
        file.write_all(bytes)?;file.sync_all()?;drop(file);
        crate::replace_state(&temporary,path).context("Cannot publish complete download")
    })();
    if result.is_err(){std::fs::remove_file(&temporary).context("Cannot remove failed download staging file")?;}
    result
}

#[cfg(any(windows,test))]
fn publish_initial_receipt(path:&Path,receipt:&Receipt)->Result<()> {
    use std::io::Write;
    let temporary=path.parent().context("Missing receipt directory")?.join(format!("swan-receipt-{}.tmp",random_token()));
    let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(&temporary)?;
    let result=(||->Result<()> {
        file.write_all(&serde_json::to_vec(receipt)?)?;file.sync_all()?;drop(file);
        #[cfg(windows)] {
            use std::os::windows::ffi::OsStrExt;
            use windows_sys::Win32::Storage::FileSystem::{MoveFileExW,MOVEFILE_WRITE_THROUGH};
            let from:Vec<u16>=temporary.as_os_str().encode_wide().chain(Some(0)).collect();
            let to:Vec<u16>=path.as_os_str().encode_wide().chain(Some(0)).collect();
            // Do not replace a receipt another updater already published.
            if unsafe{MoveFileExW(from.as_ptr(),to.as_ptr(),MOVEFILE_WRITE_THROUGH)}==0 {return Err(std::io::Error::last_os_error().into());}
        }
        #[cfg(not(windows))] {std::fs::hard_link(&temporary,path)?;std::fs::remove_file(&temporary)?;}
        Ok(())
    })();
    if result.is_err(){std::fs::remove_file(&temporary).context("Cannot clean failed receipt staging")?;}
    result
}

fn staged_hash_matches(path:&Path,hash:&str)->Result<bool> {
    match std::fs::read(path) {
        Ok(bytes)=>Ok(digest(bytes).eq_ignore_ascii_case(hash)),
        Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(false),
        Err(error)=>Err(error).context("Cannot inspect staged download"),
    }
}

#[cfg(windows)]
async fn restore_staged_download(path:&Path,url:&str,hash:&str,release:&Release)->Result<()> {
    let check_path=path.to_owned();let check_hash=hash.to_owned();
    let valid=tokio::task::spawn_blocking(move ||staged_hash_matches(&check_path,&check_hash)).await.context("Staging inspection task failed")??;
    if !valid {download(url,hash,path).await?;}
    let mut artifact_release=release.clone();artifact_release.sha256=hash.to_owned();
    verify_downloaded_publisher(path,&artifact_release).await
}

#[cfg(windows)]
fn system_directory()->Result<std::path::PathBuf> {
    let mut directory=[0u16;32768];
    let length=unsafe{windows_sys::Win32::System::SystemInformation::GetSystemDirectoryW(directory.as_mut_ptr(),directory.len() as u32)} as usize;
    ensure!(length>0 && length<directory.len(),"Cannot resolve trusted Windows system directory");
    Ok(std::path::PathBuf::from(String::from_utf16(&directory[..length])?))
}

#[cfg(windows)]
pub fn powershell_command()->Result<std::process::Command> {
    let root=system_directory()?.join("WindowsPowerShell/v1.0");
    let mut command=std::process::Command::new(root.join("powershell.exe"));
    // PowerShell 7 can pass its module paths through a Rust child to Windows
    // PowerShell. Use only Windows' built-ins for trust and compatibility checks.
    command.env("PSModulePath",root.join("Modules"));
    Ok(command)
}

#[cfg(windows)]
fn verify_publisher(path:&Path,release:&Release)->Result<()> {
    let script=path.parent().context("Missing update directory")?.join("Verify-Package.ps1");
    std::fs::write(&script,include_str!("../../../deployment/windows/Verify-Package.ps1"))?;
    let status=powershell_command()?.args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"])
        .arg(script).arg("-Path").arg(path).arg("-Publisher").arg(&release.publisher).arg("-CertificateSha256").arg(&release.publisher_certificate_sha256).arg("-Sha256").arg(&release.sha256).status()?;
    ensure!(status.success(),"Package publisher verification failed");Ok(())
}

#[cfg(windows)]
async fn verify_downloaded_publisher(path:&Path,release:&Release)->Result<()> {
    let path=path.to_owned();let release=release.clone();
    tokio::task::spawn_blocking(move ||verify_publisher(&path,&release)).await.context("Publisher verification task failed")?
}

pub(crate) fn update_policy_open(state:&AgentState)->Result<bool> {
    let profile=state.company_profile()?;
    Ok(!profile.updates_paused && maintenance_open(profile.maintenance_start_utc,profile.maintenance_end_utc,now()))
}

#[cfg(any(windows,test))]
fn replace_verified_file(source:&Path,target:&Path,expected_hash:&str)->Result<()> {
    use std::io::Write;
    // Stage on the destination volume so replacement never streams bytes into
    // the currently installed executable. Verify the exact bytes being staged.
    let bytes=std::fs::read(source)?;
    ensure!(digest(&bytes).eq_ignore_ascii_case(expected_hash),"Replacement executable hash mismatch");
    let temporary=target.parent().context("Missing replacement directory")?.join(format!("swan-replacement-{}.tmp",random_token()));
    let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(&temporary)?;
    let result=(||->Result<()> {
        file.write_all(&bytes)?;file.sync_all()?;drop(file);
        let deadline=std::time::Instant::now()+std::time::Duration::from_secs(30);
        loop {
            match crate::replace_state(&temporary,target) {
                Ok(())=>return Ok(()),
                Err(error)=>{
                    let sharing_conflict=error.downcast_ref::<std::io::Error>().map(|e|matches!(e.raw_os_error(),Some(5|32|33))).unwrap_or(false);
                    if cfg!(windows) && sharing_conflict && std::time::Instant::now()<deadline {
                        std::thread::sleep(std::time::Duration::from_millis(100));
                    }else{return Err(error).context("Executable replacement failed; previous file retained");}
                }
            }
        }
    })();
    if result.is_err(){let _=std::fs::remove_file(&temporary);}
    result
}

#[cfg(windows)]
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct MsiIdentity {product_code:String,upgrade_code:String,product_version:String,template:String}

#[cfg(windows)]
fn validate_msi_identity(identity:&MsiIdentity,edition:&Edition,version:&str)->Result<()> {
    let guid=|value:&str|value.len()==38 && value.starts_with('{') && value.ends_with('}') && value.bytes().enumerate().all(|(index,byte)|match index {0|37=>true,9|14|19|24=>byte==b'-',_=>byte.is_ascii_hexdigit()});
    ensure!(guid(&identity.product_code) && guid(&identity.upgrade_code),"Invalid MSI product or upgrade identity");
    let upgrade=if *edition==Edition::Customer {"{32A585D7-9A78-4AD2-AF72-D0266EFC709D}"}else{"{A4374699-436F-4917-9A4E-223F62A9E634}"};
    ensure!(identity.upgrade_code.eq_ignore_ascii_case(upgrade),"MSI belongs to another product or edition");
    ensure!(identity.product_version==version,"MSI version differs from signed release");
    ensure!(identity.template.split(';').next()==Some("x64"),"MSI must target Windows x64");
    Ok(())
}

#[cfg(windows)]
fn read_msi_identity(package:&Path)->Result<MsiIdentity> {
    let script=package.parent().context("Missing MSI directory")?.join("Get-MsiIdentity.ps1");
    std::fs::write(&script,include_str!("../../../deployment/windows/Get-MsiIdentity.ps1"))?;
    let output=powershell_command()?.args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(script).arg("-Package").arg(package).output()?;
    ensure!(output.status.success(),"Cannot inspect MSI identity without installation");
    let identity:MsiIdentity=serde_json::from_slice(&output.stdout)?;
    Ok(identity)
}

#[cfg(windows)]
fn verify_msi_identity(package:&Path,release:&Release)->Result<MsiIdentity> {
    ensure!(release.format=="msi","Expected a signed MSI release");
    let identity=read_msi_identity(package)?;
    validate_msi_identity(&identity,&release.edition,&release.version)?;
    Ok(identity)
}

#[cfg(windows)]
pub fn verify_msi_release_identity(package:&Path,release:&Release)->Result<()> {
    verify_msi_identity(package,release)?;Ok(())
}

#[cfg(windows)]
fn msi_install_command(package:&Path,release:&Release)->Result<std::process::Command> {
    verify_msi_identity(package,release)?;
    let script=package.parent().context("Missing update directory")?.join("Get-MsiInstallMode.ps1");
    std::fs::write(&script,include_str!("../../../deployment/windows/Get-MsiInstallMode.ps1"))?;
    let output=powershell_command()?
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
    verify_publisher(path,&release)?;
    if release.format=="msi" {verify_msi_identity(path,&release)?;}
    Ok(())
}

#[cfg(windows)]
pub fn verify_repair_package(state:&AgentState,directory:&Path,envelope:&SignedEnvelope,path:&Path)->Result<()> {
    let release=installation_release(state,directory,envelope,true)?;
    verify_compatibility(path.parent().context("Missing package directory")?,&release)?;
    ensure!(digest(std::fs::read(path)?).eq_ignore_ascii_case(&release.sha256),"Repair installer hash differs from signed metadata");
    verify_publisher(path,&release)?;
    if release.format=="msi" {verify_msi_identity(path,&release)?;}
    Ok(())
}

#[cfg(windows)]
fn verify_compatibility(directory:&Path,release:&Release)->Result<()> {
    let script=directory.join("Get-WindowsCompatibility.ps1");
    std::fs::write(&script,include_str!("../../../deployment/windows/Get-WindowsCompatibility.ps1"))?;
    let output=powershell_command()?.args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(script).output()?;
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

pub(crate) fn validate_previous_release(state:&AgentState,envelope:&SignedEnvelope,previous_sequence:u64)->Result<Release> {
    ensure!(previous_sequence>0,"Rollback requires a recorded signed installation");
    let release:Release=envelope.verify(&public_key(&state.bootstrap.release_public_key)?)?;
    release.validate(&state.bootstrap.edition,previous_sequence-1,now().min(release.expires_at.saturating_sub(1)))?;
    ensure!(release.sequence==previous_sequence,"Rollback release differs from previous installation");
    Ok(release)
}

#[cfg(windows)]
fn save_receipt(path:&Path,receipt:&Receipt)->Result<()> {
    publish_download(path,&serde_json::to_vec(receipt)?)
}

#[cfg(windows)]
fn installer_cache(directory:&Path,release:&Release)->std::path::PathBuf {
    directory.join("releases").join(release.sequence.to_string()).join(format!("installer.{}",release.format))
}

#[cfg(windows)]
fn retain_installer(source:&Path,target:&Path,release:&Release)->Result<()> {
    ensure!(std::fs::metadata(source)?.len()<=512*1024*1024,"Installer exceeds cache size limit");
    ensure!(staged_hash_matches(source,&release.sha256)?,"Installer cache source differs from signed release");
    verify_publisher(source,release)?;
    std::fs::create_dir_all(target.parent().context("Missing installer cache directory")?)?;
    replace_verified_file(source,target,&release.sha256)?;
    verify_publisher(target,release)
}

#[cfg(windows)]
async fn ensure_previous_installer(state:&AgentState,directory:&Path)->Result<()> {
    let state=state.clone();let installed_directory=directory.to_owned();
    let release=tokio::task::spawn_blocking(move ||->Result<Release> {
        let envelope:SignedEnvelope=serde_json::from_slice(&std::fs::read(installed_directory.join("installed-release.json"))?)?;
        validate_previous_release(&state,&envelope,state.last_release_sequence)
    }).await.context("Previous installer discovery failed")??;
    let cache=installer_cache(directory,&release);
    tokio::fs::create_dir_all(cache.parent().context("Missing installer cache directory")?).await?;
    // Previously recorded metadata may be expired. It authorizes retaining only
    // the exact old package bytes, never selecting another software release.
    restore_staged_download(&cache,&release.artifact_url,&release.sha256,&release).await
}

#[cfg(windows)]
fn agent_rollback_protocol(agent:&Path,directory:&Path)->Result<u32> {
    let output=std::process::Command::new(agent).arg("rollback-protocol")
        .env("SWAN_STATE_DIR",directory).env_remove("SWAN_TECHNICIAN_TOKEN")
        .env_remove("SWAN_SESSION_GRANT").env_remove("SWAN_SESSION_PROOF_KEY")
        .output().context("Cannot inspect signed previous agent rollback support")?;
    // Older signed agents may not implement the command. They can still upgrade,
    // but cannot be automatically restored with an unsupported quarantine format.
    Ok(if output.status.success() && std::str::from_utf8(&output.stdout)?.trim()=="1" {1}else{0})
}

#[cfg(windows)]
fn supports_rollback(previous:&Release,failed:&Release)->bool {
    previous.rollback_protocol==1 && previous.edition==failed.edition && previous.format==failed.format &&
        (previous.format=="msi" || (previous.edition==Edition::Technician && previous.format=="exe" && previous.sha256.eq_ignore_ascii_case(&previous.installed_sha256)))
}

#[cfg(windows)]
fn rollback_release(state:&AgentState,directory:&Path,folder:&Path,receipt:&Receipt)->Result<()> {
    ensure!(!directory.join("pending-uninstall").exists(),"Explicit uninstall cancels automatic rollback");
    ensure!(receipt.rollback_protocol==1 && receipt.phase=="rolling_back","Rollback was not durably prepared");
    ensure!(state.last_release_sequence==receipt.previous_sequence,"Cannot roll back a committed newer installation");
    let previous=receipt.previous_release.as_ref().context("Rollback requires a signed previous release")?;
    let release=validate_previous_release(state,previous,receipt.previous_sequence)?;
    let failed_release=receipt.release.verify::<Release>(&public_key(&state.bootstrap.release_public_key)?)?;
    ensure!(release.edition==state.bootstrap.edition && supports_rollback(&release,&failed_release),"Rollback requires compatible releases");
    let snapshot=folder.join("rollback");
    verify_compatibility(directory,&release)?;
    verify_rollback_snapshot(state,&snapshot,previous,receipt.previous_sequence)?;
    ensure!(agent_rollback_protocol(&snapshot.join("agent/swan-agent.exe"),directory)?==1,"Previous agent cannot enforce failed-release quarantine");
    let target=installed_target(directory,&state.bootstrap.edition)?;
    if release.format=="msi" {
        let package=snapshot.join("package/installer.msi");
        let previous_identity=verify_msi_identity(&package,&release)?;
        let next_package=folder.join("SwanRemoteSupport-install.msi");
        ensure!(staged_hash_matches(&next_package,&failed_release.sha256)?,"Failed installer identity is unavailable for narrow MSI recovery");
        verify_publisher(&next_package,&failed_release)?;
        let next_identity=verify_msi_identity(&next_package,&failed_release)?;
        let script=folder.join("Restore-ReleaseMsi.ps1");
        std::fs::write(&script,include_str!("../../../deployment/windows/Restore-ReleaseMsi.ps1"))?;
        let status=powershell_command()?.args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(script)
            .arg("-PreviousPackage").arg(&package).arg("-NextProductCode").arg(next_identity.product_code).arg("-NextVersion").arg(&failed_release.version)
            .arg("-PreviousProductCode").arg(previous_identity.product_code).arg("-PreviousVersion").arg(&release.version).arg("-Directory").arg(target.parent().context("Missing installation directory")?).arg("-Edition").arg(if release.edition==Edition::Customer {"customer"}else{"technician"}).status()?;
        ensure!(status.success(),"MSI rollback remains pending; explicit recovery or restart may be required");
    }else{
        require_existing_application(&target)?;
        replace_verified_file(&snapshot.join("endpoint/SwanRemoteSupport-Technician.exe"),&target,&release.installed_sha256)?;
    }
    replace_verified_file(&snapshot.join("agent/swan-agent.exe"),&directory.join("swan-agent.exe"),&release.agent_sha256)?;
    verify_installed(directory,&release)?;
    if release.edition==Edition::Customer {
        let script=folder.join("Restore-Configuration.ps1");
        std::fs::write(&script,include_str!("../../../deployment/windows/Restore-Configuration.ps1"))?;
        let status=powershell_command()?.args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(script).arg("-Directory").arg(directory).args(["-DeferStart","-RequireTask"]).status()?;
        ensure!(status.success(),"Customer configuration task restoration remains pending");
    }
    save_installed_metadata(directory,previous)?;
    let mut failed=receipt.clone();failed.phase="rolled_back".into();
    ensure!(receipt.release.verify::<Release>(&public_key(&state.bootstrap.release_public_key)?)?.sequence>=failed_release_sequence(state,directory)?,"Cannot lower failed-release quarantine");
    // Persist quarantine before unblocking sessions. No enrollment or consent
    // state is restored, and the installed sequence never advances on failure.
    save_receipt(&directory.join("failed-update.json"),&failed)?;
    std::fs::remove_file(directory.join("pending-update.json"))?;
    Ok(())
}

#[cfg(windows)]
fn verify_rollback_snapshot(state:&AgentState,directory:&Path,envelope:&SignedEnvelope,previous_sequence:u64)->Result<()> {
    let release=validate_previous_release(state,envelope,previous_sequence)?;
    let stored:SignedEnvelope=serde_json::from_slice(&std::fs::read(directory.join("installed-release.json"))?)?;
    ensure!(stored.payload==envelope.payload && stored.signature==envelope.signature,"Rollback metadata differs from pending receipt");
    crate::payload::verify(&directory.join("endpoint"),&release.installed_files)?;
    crate::payload::verify(&directory.join("agent"),&[InstalledFile{path:"swan-agent.exe".into(),sha256:release.agent_sha256.clone()}])?;
    let mut executable_release=release.clone();executable_release.sha256=release.installed_sha256.clone();
    let executable=if release.edition==Edition::Technician {"SwanRemoteSupport-Technician.exe"}else{"Swan Remote Support.exe"};
    verify_publisher(&directory.join("endpoint").join(executable),&executable_release)?;
    let mut agent_release=release.clone();agent_release.sha256=release.agent_sha256.clone();
    verify_publisher(&directory.join("agent/swan-agent.exe"),&agent_release)?;
    let package=directory.join("package").join(format!("installer.{}",release.format));
    // Older snapshots did not preserve installers. Forward recovery remains
    // available; future MSI rollback must separately require this package.
    if package.exists(){
        ensure!(staged_hash_matches(&package,&release.sha256)?,"Rollback installer differs from signed release");
        verify_publisher(&package,&release)?;
    }
    Ok(())
}

#[cfg(windows)]
fn prepare_rollback_snapshot(state:&AgentState,directory:&Path,update:&Release)->Result<SignedEnvelope> {
    let envelope:SignedEnvelope=serde_json::from_slice(&std::fs::read(directory.join("installed-release.json"))?)?;
    let previous=validate_previous_release(state,&envelope,state.last_release_sequence)?;
    ensure!(previous.format==update.format,"Installer format migration requires explicit setup");
    verify_compatibility(directory,&previous)?;
    verify_installed(directory,&previous)?;
    let folder=directory.join("updates").join(update.sequence.to_string());
    let destination=folder.join("rollback");
    if destination.exists(){
        ensure!(destination.join("package").join(format!("installer.{}",previous.format)).is_file(),"Existing rollback snapshot lacks its original installer; explicit recovery is required");
        verify_rollback_snapshot(state,&destination,&envelope,state.last_release_sequence)?;return Ok(envelope);
    }
    let temporary=folder.join(format!("swan-rollback-{}",random_token()));std::fs::create_dir(&temporary)?;
    let result=(||->Result<()> {
        let target=installed_target(directory,&state.bootstrap.edition)?;
        crate::payload::snapshot(target.parent().context("Missing installed directory")?,&temporary.join("endpoint"),&previous.installed_files)?;
        crate::payload::snapshot(directory,&temporary.join("agent"),&[InstalledFile{path:"swan-agent.exe".into(),sha256:previous.agent_sha256.clone()}])?;
        retain_installer(&installer_cache(directory,&previous),&temporary.join("package").join(format!("installer.{}",previous.format)),&previous)?;
        save_installed_metadata(&temporary,&envelope)?;
        verify_rollback_snapshot(state,&temporary,&envelope,state.last_release_sequence)?;
        std::fs::rename(&temporary,&destination).context("Cannot publish complete release rollback snapshot")
    })();
    if result.is_err(){std::fs::remove_dir_all(&temporary).context("Cannot remove incomplete release rollback snapshot")?;}
    result?;Ok(envelope)
}

#[cfg(windows)]
fn recovery_task(directory:&Path,release:&Release,operation:&str)->Result<()> {
    ensure!(matches!(operation,"register"|"remove"),"Unknown recovery task operation");
    let folder=directory.join("updates").join(release.sequence.to_string());
    let script=folder.join("Update-RecoveryTask.ps1");
    std::fs::write(&script,include_str!("../../../deployment/windows/Update-RecoveryTask.ps1"))?;
    let status=powershell_command()?.args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(script)
        .arg("-Operation").arg(operation).arg("-Directory").arg(directory).arg("-Edition").arg(if release.edition==Edition::Customer {"customer"}else{"technician"})
        .arg("-Sequence").arg(release.sequence.to_string()).status()?;
    ensure!(status.success(),"Durable update recovery task operation failed");Ok(())
}

#[cfg(windows)]
fn launch_update_helper(directory:&Path,release:&Release)->Result<()> {
    ensure!(!directory.join("pending-uninstall").exists(),"Explicit uninstall cancels automatic update handoff");
    use std::os::windows::process::CommandExt;
    let folder=directory.join("updates").join(release.sequence.to_string());
    let helper=folder.join("swan-agent.exe");
    ensure!(digest(std::fs::read(&helper)?).eq_ignore_ascii_case(&release.agent_sha256),"Staged update helper hash mismatch");
    let mut agent_release=release.clone();agent_release.sha256=release.agent_sha256.clone();verify_publisher(&helper,&agent_release)?;
    recovery_task(directory,release,"register")?;
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
    pub async fn recover_from_staged_task(directory:&Path)->Result<bool> {
        let check_directory=directory.to_owned();
        let pending=tokio::task::spawn_blocking(move ||->Result<Option<AgentState>> {
            let state=AgentState::load_for_refresh(&check_directory)?;
            let helper=std::env::current_exe()?;
            let folder=helper.parent().context("Missing staged recovery folder")?;
            let sequence=folder.file_name().context("Missing recovery sequence")?.to_string_lossy().parse::<u64>()?;
            ensure!(sequence>0 && helper==check_directory.join("updates").join(sequence.to_string()).join("swan-agent.exe"),"Recovery task must run from its exact staged helper");
            let receipt_path=check_directory.join("pending-update.json");
            if !receipt_path.exists() || check_directory.join("pending-uninstall").exists(){
                let installed:SignedEnvelope=serde_json::from_slice(&std::fs::read(check_directory.join("installed-release.json"))?)?;
                let mut release:Release=installed.verify(&public_key(&state.bootstrap.release_public_key)?)?;
                ensure!(release.edition==state.bootstrap.edition,"Installed release belongs to another edition");
                release.sequence=sequence;recovery_task(&check_directory,&release,"remove")?;return Ok(None);
            }
            let receipt:Receipt=serde_json::from_slice(&std::fs::read(receipt_path)?)?;
            let release=validate_pending_receipt(&state,&receipt)?;
            ensure!(release.sequence==sequence,"Another update owns the pending receipt");
            ensure!(digest(std::fs::read(&helper)?).eq_ignore_ascii_case(&release.agent_sha256),"Recovery task helper differs from signed receipt");
            let mut agent_release=release;agent_release.sha256=agent_release.agent_sha256.clone();verify_publisher(&helper,&agent_release)?;
            Ok(Some(state))
        }).await.context("Staged recovery task verification failed")??;
        let Some(state)=pending else{return Ok(false);};
        state.resume_pending_update_online(directory,false).await
    }
    pub async fn approved_update(&self,directory:&Path,technician_token:Option<&str>)->Result<Option<SignedEnvelope>> {
        let profile=self.company_profile()?;
        if !update_policy_open(self)?{return Ok(None);}
        let (path,token)=if self.bootstrap.edition==Edition::Customer {
            ("device/update",self.device_token.as_deref().context("Not enrolled")?)
        }else{("user/update",technician_token.context("Technician authentication required")?)};
        let state=self.clone();let installed_directory=directory.to_owned();
        let format=tokio::task::spawn_blocking(move ||->Result<String> {
            let installed:SignedEnvelope=serde_json::from_slice(&std::fs::read(installed_directory.join("installed-release.json"))?)?;
            Ok(validate_previous_release(&state,&installed,state.last_release_sequence)?.format)
        }).await.context("Installed release discovery failed")??;
        let value:Value=self.client()?.get(self.endpoint(path)?).query(&[("format",&format)]).bearer_auth(token).send().await?.error_for_status()?.json().await?;
        if value.is_null(){return Ok(None);}
        let envelope:SignedEnvelope=serde_json::from_value(value)?;
        let release:Release=envelope.verify(&public_key(&self.bootstrap.release_public_key)?)?;
        release.validate(&self.bootstrap.edition,0,now())?;
        ensure!(release.channel==profile.update_channel,"Wrong update channel");
        ensure!(release.format==format,"Server offered an installer format migration; explicit setup is required");
        // An already installed release is not an installation request.
        let state=self.clone();let directory=directory.to_owned();
        let failed=tokio::task::spawn_blocking(move ||failed_release_sequence(&state,&directory)).await.context("Failed-release quarantine check failed")??;
        let minimum=self.last_release_sequence.max(failed);
        if release.sequence<=minimum{return Ok(None);}
        validate_release(self,&envelope)?;
        Ok(Some(envelope))
    }
    #[cfg(windows)]
    pub fn record_installation(&mut self,directory:&Path,envelope:&SignedEnvelope,package:&Path,repair:bool)->Result<()> {
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
        retain_installer(package,&installer_cache(directory,&release),&release)?;
        save_installed_metadata(directory,envelope)?;
        latest.last_release_sequence=release.sequence;latest.save(directory)?;
        if setup_marker.exists(){std::fs::remove_file(setup_marker)?;}
        let uninstall_marker=directory.join("pending-uninstall");
        if uninstall_marker.exists(){std::fs::remove_file(uninstall_marker)?;}
        let cancellation_rollback=directory.join("uninstall-cancellation-rollback.json");
        if cancellation_rollback.exists(){std::fs::remove_file(cancellation_rollback)?;}
        *self=latest;Ok(())
    }
    #[cfg(windows)]
    pub fn recover_update(&mut self,directory:&Path)->Result<()> {
        ensure!(!directory.join("pending-uninstall").exists(),"Explicit uninstall cancels automatic recovery");
        let activity=activity_file(directory)?;fs2::FileExt::try_lock_exclusive(&activity).context("Recovery requires all sessions to close")?;
        let receipt_path=directory.join("pending-update.json");
        let receipt:Receipt=serde_json::from_slice(&std::fs::read(&receipt_path)?)?;
        let release=validate_completion_receipt(self,&receipt)?;
        verify_compatibility(directory,&release)?;
        verify_installed(directory,&release)?;
        let cache=installer_cache(directory,&release);
        let package=if cache.is_file(){cache.clone()}else{directory.join("updates").join(release.sequence.to_string()).join(format!("SwanRemoteSupport-install.{}",release.format))};
        retain_installer(&package,&cache,&release)?;
        save_installed_metadata(directory,&receipt.release)?;
        // A crashed updater may have installed successfully or saved state before
        // removing the receipt. Exact signed executable identity proves either case.
        self.last_release_sequence=release.sequence;self.save(directory)?;
        std::fs::remove_file(receipt_path)?;Ok(())
    }
    #[cfg(windows)]
    pub fn resume_pending_update(&self,directory:&Path,retry_now:bool)->Result<bool> {
        self.resume_prepared_update(directory,retry_now,None)
    }
    #[cfg(windows)]
    fn resume_prepared_update(&self,directory:&Path,retry_now:bool,expected:Option<&Receipt>)->Result<bool> {
        let activity=activity_file(directory)?;fs2::FileExt::try_lock_exclusive(&activity).context("Update recovery already running or sessions remain active")?;
        let receipt_path=directory.join("pending-update.json");
        if !receipt_path.exists(){return Ok(false);}
        ensure!(!directory.join("pending-uninstall").exists(),"Explicit uninstall cancels automatic update recovery");
        ensure!(!directory.join("pending-install.json").exists(),"Company setup recovery must finish first");
        let latest=AgentState::load_for_refresh(directory)?;
        let receipt:Receipt=serde_json::from_slice(&std::fs::read(&receipt_path)?)?;
        if let Some(expected)=expected {
            ensure!(serde_json::to_vec(&receipt)?==serde_json::to_vec(expected)?,"Pending update changed during staging recovery");
        }
        let release=validate_pending_receipt(&latest,&receipt)?;
        verify_compatibility(directory,&release)?;
        let attempt=directory.join("updates").join(release.sequence.to_string()).join("last-attempt.txt");
        if !retry_now && attempt.exists(){
            let previous=std::fs::read_to_string(attempt)?.parse::<u64>().context("Invalid update retry timestamp")?;
            if now().saturating_sub(previous)<300{return Ok(false);}
        }
        launch_update_helper(directory,&release)?;Ok(true)
    }
    #[cfg(windows)]
    pub async fn resume_pending_update_online(&self,directory:&Path,retry_now:bool)->Result<bool> {
        let check_directory=directory.to_owned();
        let plan=tokio::task::spawn_blocking(move ||->Result<Option<(Receipt,Release)>> {
            let activity=activity_file(&check_directory)?;
            fs2::FileExt::try_lock_exclusive(&activity).context("Update recovery already running or sessions remain active")?;
            let receipt_path=check_directory.join("pending-update.json");
            if !receipt_path.exists(){return Ok(None);}
            ensure!(!check_directory.join("pending-uninstall").exists(),"Explicit uninstall cancels automatic update recovery");
            ensure!(!check_directory.join("pending-install.json").exists(),"Company setup recovery must finish first");
            let latest=AgentState::load_for_refresh(&check_directory)?;
            let receipt:Receipt=serde_json::from_slice(&std::fs::read(receipt_path)?)?;
            let release=validate_pending_receipt(&latest,&receipt)?;
            verify_compatibility(&check_directory,&release)?;
            let attempt=check_directory.join("updates").join(release.sequence.to_string()).join("last-attempt.txt");
            if !retry_now && attempt.exists(){
                let previous=std::fs::read_to_string(attempt)?.parse::<u64>().context("Invalid update retry timestamp")?;
                if now().saturating_sub(previous)<300{return Ok(None);}
            }
            Ok(Some((receipt,release)))
        }).await.context("Recovery planning task failed")??;
        let Some((receipt,release))=plan else{return Ok(false);};
        // The durable signed receipt authorizes only these exact artifact bytes.
        // No credentials accompany release downloads, including redirects.
        let folder=directory.join("updates").join(release.sequence.to_string());
        tokio::fs::create_dir_all(&folder).await?;
        if receipt.phase=="installing" || (receipt.phase=="rolling_back" && release.format=="msi") {
            restore_staged_download(&folder.join(format!("SwanRemoteSupport-install.{}",release.format)),&release.artifact_url,&release.sha256,&release).await?;
        }
        restore_staged_download(&folder.join("swan-agent.exe"),&release.agent_url,&release.agent_sha256,&release).await?;
        let state=self.clone();let directory=directory.to_owned();
        tokio::task::spawn_blocking(move ||state.resume_prepared_update(&directory,retry_now,Some(&receipt))).await.context("Recovery handoff task failed")?
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
        ensure!(!directory.join("pending-uninstall").exists(),"Explicit uninstall cancels automatic update recovery");
        ensure!(!directory.join("pending-install.json").exists(),"Company setup recovery must finish first");
        let mut receipt:Receipt=serde_json::from_slice(&std::fs::read(&receipt_path)?)?;
        let mut latest=AgentState::load_for_refresh(directory)?;
        let release=validate_pending_receipt(&latest,&receipt)?;
        verify_compatibility(directory,&release)?;
        let helper=std::env::current_exe()?;
        let mut agent_release=release.clone();agent_release.sha256=release.agent_sha256.clone();
        ensure!(digest(std::fs::read(&helper)?).eq_ignore_ascii_case(&release.agent_sha256),"Update helper differs from signed release");
        verify_publisher(&helper,&agent_release)?;
        let folder=directory.join("updates").join(release.sequence.to_string());
        if receipt.phase=="rolling_back" {
            rollback_release(&latest,directory,&folder,&receipt)?;
            *self=latest;return Ok(true);
        }
        let package=folder.join(format!("SwanRemoteSupport-install.{}",release.format));
        ensure!(digest(std::fs::read(&package)?).eq_ignore_ascii_case(&release.sha256),"Staged installer hash mismatch");
        verify_publisher(&package,&release)?;
        let installation=(||->Result<()> {
        if let Err(error)=verify_installed(directory,&release) {
            eprintln!("Installed payload requires recovery: {error:#}");
            if let Some(previous)=receipt.previous_release.as_ref(){
                verify_rollback_snapshot(&latest,&folder.join("rollback"),previous,receipt.previous_sequence)?;
            }
            if latest.bootstrap.edition==Edition::Technician {
                ensure!(release.format=="msi" || (release.format=="exe" && release.sha256.eq_ignore_ascii_case(&release.installed_sha256)),"Technician updates require an MSI or a portable EXE");
                let target=installed_target(directory,&latest.bootstrap.edition)?;
                require_existing_application(&target)?;
                let previous=folder.join("previous-technician.exe");
                if !previous.exists(){std::fs::copy(&target,&previous)?;}
                if release.format=="msi" {
                    let status=msi_install_command(&package,&release)?.arg(format!("INSTALLFOLDER={}",directory.display())).status()?;
                    ensure!(matches!(status.code(),Some(0|3010)),"Technician MSI installation failed; retain signed recovery receipt");
                }else{replace_verified_file(&package,&target,&release.installed_sha256)?;}
            }else{
                require_existing_application(&installed_target(directory,&latest.bootstrap.edition)?)?;
                let mut command=if release.format=="msi" {msi_install_command(&package,&release)?}else{let mut c=std::process::Command::new(&package);c.args(["--silent-install","printer=0"]);c};
                let status=command.status()?;
                ensure!(matches!(status.code(),Some(0|3010)),"Installation failed; retain signed recovery receipt");
            }
            let installed_agent=directory.join("swan-agent.exe");
            let agent_already_current=installed_agent.is_file() && digest(std::fs::read(&installed_agent)?).eq_ignore_ascii_case(&release.agent_sha256);
            if !agent_already_current {
                let previous=folder.join("previous-agent.exe");
                if installed_agent.exists() && !previous.exists(){std::fs::copy(&installed_agent,&previous)?;}
                replace_verified_file(&helper,&installed_agent,&release.agent_sha256).context("Agent replacement failed; retain update receipt")?;
            }
        }
        verify_installed(directory,&release)?;
        Ok(())
        })();
        if let Err(error)=installation {
            let previous=receipt.previous_release.as_ref().map(|envelope|validate_previous_release(&latest,envelope,receipt.previous_sequence)).transpose()?;
            let can_rollback=receipt.rollback_protocol==1 && latest.last_release_sequence==receipt.previous_sequence && previous.as_ref().map(|old|supports_rollback(old,&release)).unwrap_or(false);
            if !can_rollback{return Err(error).context("Update failed; signed recovery remains pending");}
            eprintln!("Update failed; restoring the verified previous release: {error:#}");
            receipt.phase="rolling_back".into();save_receipt(&receipt_path,&receipt)?;
            rollback_release(&latest,directory,&folder,&receipt)?;
            *self=latest;return Ok(true);
        }
        save_installed_metadata(directory,&receipt.release)?;
        retain_installer(&package,&installer_cache(directory,&release),&release)?;
        latest.last_release_sequence=release.sequence;latest.save(directory)?;
        std::fs::remove_file(receipt_path)?;*self=latest;Ok(true)
    }
    pub async fn update(&mut self,directory:&Path,technician_token:Option<&str>)->Result<bool> {
        #[cfg(not(windows))] {let _=(directory,technician_token);anyhow::bail!("Endpoint updates require Windows");}
        #[cfg(windows)] {
            ensure!(!directory.join("pending-uninstall").exists(),"Explicit uninstall cancels automatic updates");
            ensure!(!directory.join("pending-install.json").exists(),"Company setup recovery is required before automatic updates");
            if directory.join("pending-update.json").exists(){
                return self.resume_pending_update_online(directory,false).await;
            }
            require_existing_application(&installed_target(directory,&self.bootstrap.edition)?)?;
            if !update_policy_open(self)?{return Ok(false);}
            let receipt_path=directory.join("pending-update.json");
            // An interrupted installation is deliberately not guessed successful.
            // Keep the signed receipt so recovery can verify the installed build.
            ensure!(!receipt_path.exists(),"Interrupted update requires recovery before another installation");
            let Some(envelope)=self.approved_update(directory,technician_token).await? else{return Ok(false);};
            let release=validate_release(self,&envelope)?;
            ensure!(release.edition!=Edition::Technician || (release.format=="msi" || (release.format=="exe" && release.installed_sha256.eq_ignore_ascii_case(&release.sha256))),"Technician updates require an MSI or a portable EXE with matching installed identity");
            let folder=directory.join("updates").join(release.sequence.to_string());tokio::fs::create_dir_all(&folder).await?;
            let check_folder=folder.clone();let check_release=release.clone();
            tokio::task::spawn_blocking(move ||verify_compatibility(&check_folder,&check_release)).await.context("Compatibility verification task failed")??;
            let package=folder.join(format!("SwanRemoteSupport-install.{}",release.format));
            download(&release.artifact_url,&release.sha256,&package).await?;
            verify_downloaded_publisher(&package,&release).await?;
            let replacement_agent=folder.join("swan-agent.exe");
            download(&release.agent_url,&release.agent_sha256,&replacement_agent).await?;
            let mut agent_release=release.clone();agent_release.sha256=release.agent_sha256.clone();
            verify_downloaded_publisher(&replacement_agent,&agent_release).await?;
            ensure_previous_installer(self,directory).await?;
            // Downloads may take minutes. Require a fresh signed policy and
            // current server approval before creating an installation receipt.
            self.sync().await?;self.save(directory)?;
            let Some(current)=self.approved_update(directory,technician_token).await? else{return Ok(false);};
            if current.payload!=envelope.payload || current.signature!=envelope.signature{return Ok(false);}
            let directory=directory.to_owned();
            let prepared=tokio::task::spawn_blocking(move ||->Result<Option<AgentState>> {
            let activity=activity_file(&directory)?;
            fs2::FileExt::try_lock_exclusive(&activity).context("Update deferred while a session or connection attempt is active")?;
            ensure!(!directory.join("pending-install.json").exists(),"Company setup started while the update was downloading");
            ensure!(!directory.join("pending-uninstall").exists(),"Explicit uninstall started while the update was downloading");
            ensure!(!receipt_path.exists(),"Another update already requires recovery");
            // Re-read consent and enrollment immediately before installation.
            let latest=AgentState::load(&directory)?;
            if !update_policy_open(&latest)?{return Ok(None);}
            // Removal during download must not be turned into a fresh install.
            require_existing_application(&installed_target(&directory,&latest.bootstrap.edition)?)?;
            validate_release(&latest,&envelope)?;
            ensure!(release.sequence>failed_release_sequence(&latest,&directory)?,"Failed release is quarantined");
            let previous_release=prepare_rollback_snapshot(&latest,&directory,&release)?;
            let previous=validate_previous_release(&latest,&previous_release,latest.last_release_sequence)?;
            let rollback_protocol=if previous.rollback_protocol==1 {agent_rollback_protocol(&directory.join("updates").join(release.sequence.to_string()).join("rollback/agent/swan-agent.exe"),&directory)?}else{0};
            let receipt=Receipt{release:envelope,previous_sequence:latest.last_release_sequence,phase:"installing".into(),previous_release:Some(previous_release),rollback_protocol};
            publish_initial_receipt(&receipt_path,&receipt)?;
            // The signed replacement runs from staging. This caller must exit so
            // Windows releases the old executable before it is overwritten.
            launch_update_helper(&directory,&release)?;
            Ok(Some(latest))
            }).await.context("Update handoff task failed")??;
            if let Some(latest)=prepared {*self=latest;Ok(true)}else{Ok(false)}
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
        files.push(InstalledFile{path:"RuntimeBroker_rustdesk.exe".into(),sha256:digest(std::fs::read(system_directory()?.join("RuntimeBroker.exe"))?)});
        crate::payload::verify(target.parent().context("Missing installation directory")?,&files)?;
    }
    ensure!(digest(std::fs::read(&target)?).eq_ignore_ascii_case(&release.installed_sha256),"Installed executable differs from signed release; recovery remains pending");
    let mut installed=release.clone();installed.sha256=release.installed_sha256.clone();verify_publisher(&target,&installed)
}

#[cfg(test)]
pub(crate) fn test_failed_release_quarantine(state:&AgentState,release:&Release,key:&SigningKey) {
    let folder=std::env::temp_dir().join(format!("swan-quarantine-test-{}",random_token()));std::fs::create_dir(&folder).unwrap();
    assert_eq!(failed_release_sequence(state,&folder).unwrap(),0);
    let mut previous=release.clone();previous.sequence=1;previous.expires_at=now()-1;previous.rollback_protocol=1;
    #[cfg(windows)] {
        let mut old=previous.clone();old.format="msi".into();old.edition=Edition::Customer;
        let mut failed=old.clone();assert!(supports_rollback(&old,&failed));
        failed.edition=Edition::Technician;assert!(!supports_rollback(&old,&failed));
        old.edition=Edition::Technician;assert!(supports_rollback(&old,&failed));
        old.rollback_protocol=0;assert!(!supports_rollback(&old,&failed));old.rollback_protocol=1;
        old.format="exe".into();assert!(!supports_rollback(&old,&failed));
        failed.format="exe".into();old.installed_sha256=old.sha256.clone();assert!(supports_rollback(&old,&failed));
        old.edition=Edition::Customer;failed.edition=Edition::Customer;assert!(!supports_rollback(&old,&failed));
        let mut identity=MsiIdentity{product_code:"{00112233-4455-6677-8899-AABBCCDDEEFF}".into(),upgrade_code:"{32A585D7-9A78-4AD2-AF72-D0266EFC709D}".into(),product_version:release.version.clone(),template:"x64;1033".into()};
        validate_msi_identity(&identity,&Edition::Customer,&release.version).unwrap();
        assert!(validate_msi_identity(&identity,&Edition::Technician,&release.version).is_err());
        assert!(validate_msi_identity(&identity,&Edition::Customer,"99.0.0").is_err());
        identity.template="Intel;1033".into();assert!(validate_msi_identity(&identity,&Edition::Customer,&release.version).is_err());
        identity.template="x64;1033".into();identity.product_code="invalid product".into();assert!(validate_msi_identity(&identity,&Edition::Customer,&release.version).is_err());
        let builtins=powershell_command().unwrap().args(["-NoLogo","-NoProfile","-NonInteractive","-Command","$ErrorActionPreference='Stop'; Get-Command Get-FileHash,Get-AuthenticodeSignature | ForEach-Object Name"]).output().unwrap();
        assert!(builtins.status.success());
        let commands=String::from_utf8(builtins.stdout).unwrap();
        assert!(commands.contains("Get-FileHash") && commands.contains("Get-AuthenticodeSignature"));
        let source=folder.join("unsigned-installer.exe");let target=folder.join("cache/installer.exe");
        std::fs::write(&source,b"unsigned cache fixture").unwrap();
        let mut installer=release.clone();installer.sha256=digest(b"unsigned cache fixture");
        let mut tampered=installer.clone();tampered.sha256=digest(b"different installer");
        assert!(retain_installer(&source,&target,&tampered).is_err());
        assert!(!target.exists());
        assert!(retain_installer(&source,&target,&installer).is_err(),"Correct hashes cannot bypass publisher verification");
        assert!(!target.exists());
        // Exercise real Authenticode trust without signing a test program or
        // executing an installer: copy a Windows-signed OS binary into our fixture.
        std::fs::copy(system_directory().unwrap().join("cmd.exe"),&source).unwrap();
        let identity=powershell_command().unwrap().env("SWAN_TEST_TRUST_FIXTURE",&source).args(["-NoLogo","-NoProfile","-NonInteractive","-Command","$ErrorActionPreference='Stop'; $s=Get-AuthenticodeSignature -LiteralPath $env:SWAN_TEST_TRUST_FIXTURE; if($s.Status -ne 'Valid'){throw 'Windows fixture signature unavailable'}; $h=[Security.Cryptography.SHA256]::Create(); try { @{publisher=$s.SignerCertificate.GetNameInfo([Security.Cryptography.X509Certificates.X509NameType]::SimpleName,$false);certificate_hash=[BitConverter]::ToString($h.ComputeHash($s.SignerCertificate.RawData)).Replace('-','')} | ConvertTo-Json -Compress } finally {$h.Dispose()}"]).output().unwrap();
        assert!(identity.status.success());
        let identity:Value=serde_json::from_slice(&identity.stdout).unwrap();
        installer.publisher=identity["publisher"].as_str().unwrap().into();
        installer.publisher_certificate_sha256=identity["certificate_hash"].as_str().unwrap().into();
        installer.sha256=digest(std::fs::read(&source).unwrap());
        retain_installer(&source,&target,&installer).unwrap();
        assert!(staged_hash_matches(&target,&installer.sha256).unwrap());
        let mut wrong_publisher=installer.clone();wrong_publisher.publisher="Unapproved publisher".into();
        assert!(retain_installer(&source,&target,&wrong_publisher).is_err());
        assert!(staged_hash_matches(&target,&installer.sha256).unwrap(),"Publisher rejection retains the previous complete cache");
        let mut wrong_certificate=installer.clone();wrong_certificate.publisher_certificate_sha256="0".repeat(64);
        assert!(retain_installer(&source,&target,&wrong_certificate).is_err());
        assert!(staged_hash_matches(&target,&installer.sha256).unwrap(),"Certificate rejection retains the previous complete cache");
    }
    let mut failed=release.clone();failed.expires_at=now()-1;
    let receipt=Receipt{release:SignedEnvelope::sign(&failed,key).unwrap(),previous_sequence:1,phase:"rolled_back".into(),previous_release:Some(SignedEnvelope::sign(&previous,key).unwrap()),rollback_protocol:1};
    let path=folder.join("failed-update.json");
    let pending_path=folder.join("pending-update.json");
    publish_initial_receipt(&pending_path,&receipt).unwrap();
    let saved=std::fs::read(&pending_path).unwrap();
    let mut competing=receipt.clone();competing.phase="installing".into();
    assert!(publish_initial_receipt(&pending_path,&competing).is_err());
    assert_eq!(std::fs::read(&pending_path).unwrap(),saved,"Never overwrite another durable recovery receipt");
    assert!(!std::fs::read_dir(&folder).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with("swan-receipt-")));
    let write=|receipt:&Receipt|std::fs::write(&path,serde_json::to_vec(receipt).unwrap()).unwrap();
    write(&receipt);assert_eq!(failed_release_sequence(state,&folder).unwrap(),2);
    assert!(release.validate(&state.bootstrap.edition,failed_release_sequence(state,&folder).unwrap(),now()).is_err(),"The failed sequence cannot be selected again");
    let mut later=release.clone();later.sequence=3;
    assert!(later.validate(&state.bootstrap.edition,failed_release_sequence(state,&folder).unwrap(),now()).is_ok());
    let mut pending=receipt.clone();pending.phase="rolling_back".into();
    assert!(validate_pending_receipt(state,&pending).is_ok());
    assert!(validate_completion_receipt(state,&pending).is_err(),"Completion-only recovery must not override a pending rollback");
    let mut committed=state.clone();committed.last_release_sequence=2;
    assert!(validate_pending_receipt(&committed,&pending).is_err());
    let mut different_format=previous.clone();different_format.format="msi".into();
    pending.previous_release=Some(SignedEnvelope::sign(&different_format,key).unwrap());
    assert!(validate_pending_receipt(state,&pending).is_err(),"Automatic rollback must not ignore installer registrations");
    let mut legacy=previous.clone();legacy.rollback_protocol=0;
    pending.previous_release=Some(SignedEnvelope::sign(&legacy,key).unwrap());
    assert!(validate_pending_receipt(state,&pending).is_err(),"Never restore an endpoint lacking quarantine support");
    pending.previous_release=None;assert!(validate_pending_receipt(state,&pending).is_err());
    pending.phase="installing".into();pending.rollback_protocol=0;
    assert!(validate_pending_receipt(state,&pending).is_ok(),"Legacy receipts can still finish forward recovery");
    assert!(validate_completion_receipt(state,&pending).is_ok());
    pending.rollback_protocol=2;assert!(validate_pending_receipt(state,&pending).is_err());
    let mut newer=state.clone();newer.last_release_sequence=3;
    assert_eq!(failed_release_sequence(&newer,&folder).unwrap(),2,"Installing a later release retains failed-release history");
    let mut foreign=state.clone();foreign.bootstrap.edition=Edition::Technician;
    assert!(failed_release_sequence(&foreign,&folder).is_err());
    foreign.bootstrap.edition=state.bootstrap.edition.clone();foreign.bootstrap.release_public_key=STANDARD.encode(SigningKey::from_bytes(&[9;32]).verifying_key().as_bytes());
    assert!(failed_release_sequence(&foreign,&folder).is_err());
    let mut bad=receipt.clone();bad.release.signature=STANDARD.encode([0u8;64]);write(&bad);
    assert!(failed_release_sequence(state,&folder).is_err());
    let mut bad=receipt.clone();bad.previous_sequence=2;write(&bad);
    assert!(failed_release_sequence(state,&folder).is_err());
    let mut bad=receipt.clone();bad.rollback_protocol=0;write(&bad);
    assert!(failed_release_sequence(state,&folder).is_err());
    let mut bad=receipt.clone();bad.phase="rolling_back".into();write(&bad);
    assert!(failed_release_sequence(state,&folder).is_err());
    std::fs::write(&path,b"interrupted metadata").unwrap();assert!(failed_release_sequence(state,&folder).is_err());
    std::fs::remove_file(&path).unwrap();std::fs::create_dir(&path).unwrap();
    assert!(failed_release_sequence(state,&folder).is_err(),"Quarantine I/O failures must not clear suppression");
    std::fs::remove_dir_all(folder).unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;
    #[cfg(windows)]
    #[test]
    #[ignore="Requires an explicitly selected isolated MSI artifact; reads its database without installation"]
    fn real_msi_identity_matches_release_policy_without_installation(){
        let package=std::path::PathBuf::from(std::env::var_os("SWAN_TEST_MSI_PACKAGE").expect("Set private fixture MSI path"));
        let edition=match std::env::var("SWAN_TEST_MSI_EDITION").unwrap().as_str(){"customer"=>Edition::Customer,"technician"=>Edition::Technician,_=>panic!("Explicit fixture edition required")};
        let version=std::env::var("SWAN_TEST_MSI_VERSION").expect("Explicit fixture version required");
        let hash=digest(std::fs::read(&package).unwrap());
        let identity=read_msi_identity(&package).unwrap();
        validate_msi_identity(&identity,&edition,&version).unwrap();
        let wrong=if edition==Edition::Customer {Edition::Technician}else{Edition::Customer};
        assert!(validate_msi_identity(&identity,&wrong,&version).is_err());
        assert!(validate_msi_identity(&identity,&edition,"99.0.0").is_err());
        {
            let script=package.parent().unwrap().join("Restore-ReleaseMsi-read-only-test.ps1");
            std::fs::write(&script,include_str!("../../../deployment/windows/Restore-ReleaseMsi.ps1")).unwrap();
            let inspect=|previous_version:&str|powershell_command().unwrap().args(["-NoLogo","-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(&script)
                .arg("-PreviousPackage").arg(&package).arg("-NextProductCode").arg(&identity.product_code).arg("-NextVersion").arg(&version)
                .arg("-PreviousProductCode").arg(&identity.product_code).arg("-PreviousVersion").arg(previous_version).arg("-Directory").arg(package.parent().unwrap()).arg("-Edition").arg(if edition==Edition::Customer {"customer"}else{"technician"}).arg("-InspectOnly").output().unwrap();
            let result=inspect(&version);assert!(result.status.success(),"{}",String::from_utf8_lossy(&result.stderr));
            let plan:serde_json::Value=serde_json::from_slice(&result.stdout).unwrap();
            assert_eq!(plan["installer_executed"],false);assert_eq!(plan["would_remove_next"],false);
            assert!(!inspect("99.0.0").status.success(),"Read-only recovery planning must reject incorrect previous identity");
        }
        assert_eq!(hash,digest(std::fs::read(package).unwrap()),"Read-only MSI inspection must not change its bytes");
    }
    #[tokio::test]
    #[ignore="Requires the isolated loopback HTTPS download-recovery harness"]
    async fn https_download_recovery_preserves_staging_on_tamper_and_interruption(){
        let base=std::env::var("SWAN_TEST_DOWNLOAD_BASE").expect("Run deployment/windows/test-update-download.js");
        assert!(base.starts_with("https://localhost:"));
        let folder=std::env::temp_dir().join(format!("swan-https-download-test-{}",random_token()));
        std::fs::create_dir(&folder).unwrap();
        let artifact=folder.join("installer.exe");
        let payload=b"Swan HTTPS recovery download fixture\n";let hash=digest(payload);
        std::fs::write(&artifact,b"previous complete staging").unwrap();
        download(&format!("{base}/artifact"),&hash,&artifact).await.unwrap();
        assert_eq!(std::fs::read(&artifact).unwrap(),payload);
        assert!(download(&format!("{base}/tampered"),&hash,&artifact).await.is_err());
        assert_eq!(std::fs::read(&artifact).unwrap(),payload);
        assert!(download(&format!("{base}/partial"),&hash,&artifact).await.is_err());
        assert_eq!(std::fs::read(&artifact).unwrap(),payload);
        assert!(!std::fs::read_dir(&folder).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with("swan-download-")));
        std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn recovery_staging_requires_exact_hash_and_distinguishes_missing_from_io_failure(){
        let folder=std::env::temp_dir().join(format!("swan-staging-test-{}",random_token()));
        std::fs::create_dir(&folder).unwrap();
        let artifact=folder.join("helper.exe");let hash=digest(b"authorized helper");
        assert!(!staged_hash_matches(&artifact,&hash).unwrap());
        std::fs::write(&artifact,b"truncated helper").unwrap();
        assert!(!staged_hash_matches(&artifact,&hash).unwrap());
        publish_download(&artifact,b"authorized helper").unwrap();
        assert!(staged_hash_matches(&artifact,&hash.to_uppercase()).unwrap());
        assert!(staged_hash_matches(&folder,&hash).is_err());
        std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn download_publication_retains_complete_files_and_cleans_failed_staging(){
        let folder=std::env::temp_dir().join(format!("swan-download-test-{}",random_token()));
        std::fs::create_dir(&folder).unwrap();
        let target=folder.join("installer.exe");
        std::fs::write(&target,b"previous complete package").unwrap();
        publish_download(&target,b"new complete package").unwrap();
        assert_eq!(std::fs::read(&target).unwrap(),b"new complete package");
        let conflict=folder.join("directory.exe");std::fs::create_dir(&conflict).unwrap();
        assert!(publish_download(&conflict,b"replacement").is_err());
        assert!(conflict.is_dir());
        assert_eq!(std::fs::read(&target).unwrap(),b"new complete package");
        assert!(!std::fs::read_dir(&folder).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with("swan-download-")));
        std::fs::remove_dir_all(folder).unwrap();
    }
    #[test]
    fn executable_replacement_verifies_staged_bytes_and_preserves_previous_on_failure(){
        let folder=std::env::temp_dir().join(format!("swan-replacement-test-{}",random_token()));
        std::fs::create_dir(&folder).unwrap();
        let source=folder.join("source.exe");let target=folder.join("installed.exe");
        std::fs::write(&source,b"complete replacement").unwrap();std::fs::write(&target,b"previous executable").unwrap();
        assert!(replace_verified_file(&source,&target,&digest(b"wrong content")).is_err());
        assert_eq!(std::fs::read(&target).unwrap(),b"previous executable");
        replace_verified_file(&source,&target,&digest(b"complete replacement")).unwrap();
        assert_eq!(std::fs::read(&target).unwrap(),b"complete replacement");
        assert_eq!(std::fs::read(&source).unwrap(),b"complete replacement");
        let directory_target=folder.join("directory.exe");std::fs::create_dir(&directory_target).unwrap();
        assert!(replace_verified_file(&source,&directory_target,&digest(b"complete replacement")).is_err());
        assert!(directory_target.is_dir());
        assert!(!std::fs::read_dir(&folder).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with("swan-replacement-")));
        std::fs::remove_dir_all(folder).unwrap();
    }
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
        std::fs::write(folder.join("pending-uninstall"),b"explicit cancellation").unwrap();
        assert!(lock_session(&folder).is_err());
        std::fs::remove_file(folder.join("pending-uninstall")).unwrap();
        assert!(lock_session(&folder).is_ok());
        std::fs::remove_file(&target).unwrap();
        assert!(require_existing_application(&target).is_err());
        std::fs::remove_file(folder.join("activity.lock")).unwrap();
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
    #[test]
    fn closing_one_connection_does_not_release_another_connections_update_lock() {
        let folder=std::env::temp_dir().join(format!("swan-multiple-session-lock-{}",random_token()));
        let first=lock_session(&folder).unwrap();
        let second=lock_session(&folder).unwrap();
        let update=activity_file(&folder).unwrap();
        assert!(fs2::FileExt::try_lock_exclusive(&update).is_err());
        drop(first);
        assert!(fs2::FileExt::try_lock_exclusive(&update).is_err());
        drop(second);
        fs2::FileExt::try_lock_exclusive(&update).unwrap();
        assert!(lock_session(&folder).is_err());
        drop(update);
        assert!(lock_session(&folder).is_ok());
        std::fs::remove_dir_all(folder).unwrap();
    }
}
