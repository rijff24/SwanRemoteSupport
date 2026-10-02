use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::{io::Write,path::{Path,PathBuf}};
use swan_agent::Bootstrap;
use swan_protocol::*;

#[tokio::main]
async fn main()->Result<()> {
    ensure!(cfg!(windows),"Installer worker requires Windows for Authenticode verification");
    let server=std::env::var("SWAN_MANAGEMENT_URL").context("Set SWAN_MANAGEMENT_URL")?;https_url(&server)?;
    let token=std::env::var("SWAN_WORKER_TOKEN").context("Set SWAN_WORKER_TOKEN")?;
    let release_key=std::env::var("SWAN_RELEASE_PUBLIC_KEY").context("Pin SWAN_RELEASE_PUBLIC_KEY on the worker")?;public_key(&release_key)?;
    let company_key=std::env::var("SWAN_PROFILE_PUBLIC_KEY").context("Pin SWAN_PROFILE_PUBLIC_KEY on the worker")?;public_key(&company_key)?;
    let output=PathBuf::from(std::env::var("SWAN_ARTIFACT_DIR").context("Set worker artifact output directory")?);
    tokio::fs::create_dir_all(&output).await?;
    let client=swan_agent::http_client(600,false)?;
    loop {
        let response=client.post(format!("{}/api/v1/worker/claim",server.trim_end_matches('/'))).bearer_auth(&token).send().await;
        match response {
            Ok(response) if response.status().is_success()=>{
                if let Err(error)=process_claim(response,&client,&server,&token,&output,&release_key,&company_key).await {
                    eprintln!("Worker job processing failed: {error:#}. Continuing polling; the server retains the job for recovery.");
                }
            }
            Ok(response)=>eprintln!("Worker polling failed: {}",response.status()),
            Err(_)=>eprintln!("Worker management endpoint unavailable; retrying"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }
}

async fn process_claim(response:reqwest::Response,client:&reqwest::Client,server:&str,token:&str,output:&Path,release_key:&str,company_key:&str)->Result<()> {
                let value:Value=response.json().await?;
                if !value.is_null(){
                    let id=value["id"].as_str().context("Missing build identity")?;
                    // Server-generated UUIDs become filenames, never executable commands.
                    ensure!(id.len()==36 && id.bytes().all(|c|c.is_ascii_hexdigit() || c==b'-'),"Invalid job identity");
                    let outcome=build(&client,&value["job"],&output,id,&release_key,&company_key).await;
                    let result=match outcome {
                        Ok((name,hash))=>{
                            let bytes=tokio::fs::read(output.join(name)).await?;
                            send_idempotent(client.put(format!("{}/api/v1/worker/builds/{id}/artifact",server.trim_end_matches('/'))).bearer_auth(&token).header("Content-Type","application/zip").body(bytes)).await?;
                            json!({"success":true,"artifact_url":format!("{}/api/v1/downloads/{id}",server.trim_end_matches('/')),"sha256":hash,"log":"Verified signed inputs and uploaded company bundle."})
                        },
                        Err(error)=>{eprintln!("Build {id} failed: {error}");json!({"success":false,"artifact_url":"","sha256":"","log":"Build failed; see the protected worker log. No package was published."})}
                    };
                    report_completion(client,server,token,id,&result).await?;
                }
    Ok(())
}

async fn report_completion(client:&reqwest::Client,server:&str,token:&str,id:&str,result:&Value)->Result<()> {
    send_idempotent(client.put(format!("{}/api/v1/worker/builds/{id}",server.trim_end_matches('/'))).bearer_auth(token).json(result)).await
}

async fn send_idempotent(request:reqwest::RequestBuilder)->Result<()> {
    for attempt in 0..3 {
        let response=request.try_clone().context("Worker retry requires a buffered request")?.send().await;
        match response {
            Ok(response) if response.status().is_success()=>return Ok(()),
            Ok(response) if !response.status().is_server_error() && response.status()!=reqwest::StatusCode::TOO_MANY_REQUESTS=>{
                response.error_for_status()?;
                anyhow::bail!("Unexpected completion response");
            },
            _=>{},
        }
        if attempt<2 {tokio::time::sleep(std::time::Duration::from_secs(5*(attempt+1))).await;}
    }
    anyhow::bail!("Worker upload or report unavailable after bounded retries; server retains job recovery")
}

async fn build(_client:&reqwest::Client,job:&Value,output:&Path,id:&str,release_key:&str,company_key:&str)->Result<(String,String)> {
    ensure!(job["release_public_key"].as_str()==Some(release_key) && job["profile_public_key"].as_str()==Some(company_key),"Worker trust pins do not match the company job");
    let release_envelope:SignedEnvelope=serde_json::from_value(job["release"].clone())?;
    let release:Release=release_envelope.verify(&public_key(release_key)?)?;
    release.validate(&release.edition,0,now())?;
    let profile_envelope:SignedEnvelope=serde_json::from_value(job["profile"].clone())?;
    let profile:CompanyProfile=profile_envelope.verify(&public_key(company_key)?)?;
    profile.validate(&profile.company_id,0,now())?;
    let work=output.join(format!("work-{id}"));tokio::fs::create_dir_all(&work).await?;
    let installer=work.join(format!("SwanRemoteSupport-install.{}",release.format));
    let agent=work.join("swan-agent.exe");
    let artifact_client=swan_agent::http_client(600,true)?;
    download(&artifact_client,&release.artifact_url,&installer,&release.sha256).await?;
    download(&artifact_client,&release.agent_url,&agent,&release.agent_sha256).await?;
    let output=output.to_owned();let id=id.to_owned();let company_key=company_key.to_owned();let release_key=release_key.to_owned();
    tokio::task::spawn_blocking(move ||package_verified(output,&id,work,installer,agent,release,profile,release_envelope,profile_envelope,&release_key,&company_key))
        .await.context("Installer verification and packaging task failed")?
}

fn package_verified(output:PathBuf,id:&str,work:PathBuf,installer:PathBuf,agent:PathBuf,release:Release,profile:CompanyProfile,release_envelope:SignedEnvelope,profile_envelope:SignedEnvelope,release_key:&str,company_key:&str)->Result<(String,String)> {
    verify_windows(&installer,&release,&work)?;
    verify_windows(&agent,&release,&work)?;
    let bootstrap=Bootstrap{schema:SCHEMA,edition:release.edition.clone(),company_id:profile.company_id,management_url:profile.management_url,profile_public_key:company_key.into(),release_public_key:release_key.into()};
    let file_name=format!("SwanRemoteSupport-{}-{}-{}.zip",if release.edition==Edition::Customer{"Customer"}else{"Technician"},release.version,id);
    let temporary=output.join(format!("{file_name}.partial"));
    let mut zip=zip::ZipWriter::new(std::fs::File::create(&temporary)?);
    let options=zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
    for path in [&installer,&agent] {zip.start_file(path.file_name().context("Missing artifact name")?.to_string_lossy(),options)?;zip.write_all(&std::fs::read(path)?)?;}
    for (name,bytes) in [
        ("bootstrap.json",serde_json::to_vec_pretty(&bootstrap)?),
        ("company-profile.json",serde_json::to_vec_pretty(&profile_envelope)?),
        ("release.json",serde_json::to_vec_pretty(&release_envelope)?),
        ("Install-Company.ps1",bundle_text(include_bytes!("../../../deployment/windows/Install-Company.ps1"))?),
        ("Open-Technician.ps1",bundle_text(include_bytes!("../../../deployment/windows/Open-Technician.ps1"))?),
        ("Verify-Package.ps1",bundle_text(include_bytes!("../../../deployment/windows/Verify-Package.ps1"))?),
        ("LICENSE.txt",bundle_text(include_bytes!("../../../LICENCE"))?),
    ] {zip.start_file(name,options)?;zip.write_all(&bytes)?;}
    zip.finish()?.sync_all()?;
    let hash=digest(std::fs::read(&temporary)?);
    let final_name=file_name.replace(".zip",&format!("-{}.zip",&hash[..16]));let target=output.join(&final_name);
    if target.exists(){ensure!(digest(std::fs::read(&target)?)==hash,"Conflicting immutable artifact");std::fs::remove_file(&temporary)?;}else{std::fs::rename(&temporary,target)?;}
    Ok((final_name,hash))
}
pub async fn download(client:&reqwest::Client,url:&str,path:&Path,hash:&str)->Result<()> {
    https_url(url)?;
    let mut response=client.get(url).send().await?.error_for_status()?;
    ensure!(response.content_length().map(|n|n<=512*1024*1024).unwrap_or(true),"Artifact too large");
    let mut bytes=Vec::new();
    while let Some(chunk)=response.chunk().await?{ensure!(bytes.len()+chunk.len()<=512*1024*1024,"Artifact too large");bytes.extend(chunk);}
    ensure!(digest(&bytes).eq_ignore_ascii_case(hash),"Artifact hash mismatch");
    tokio::fs::write(path,bytes).await?;Ok(())
}
#[cfg(windows)]
fn verify_windows(path:&Path,release:&Release,directory:&Path)->Result<()> {
    let verifier=directory.join("Verify-Package.ps1");std::fs::write(&verifier,include_bytes!("../../../deployment/windows/Verify-Package.ps1"))?;
    let status=swan_agent::update::powershell_command()?.args(["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(verifier).arg("-Path").arg(path).arg("-Publisher").arg(&release.publisher).arg("-CertificateSha256").arg(&release.publisher_certificate_sha256).status()?;
    ensure!(status.success(),"Artifact signature or publisher rejected");
    if release.format=="msi" {swan_agent::update::verify_msi_release_identity(path,release)?;}
    Ok(())
}

#[cfg(not(windows))]
fn verify_windows(_path:&Path,_release:&Release,_directory:&Path)->Result<()> {
    anyhow::bail!("Artifact signature verification requires a Windows worker")
}
