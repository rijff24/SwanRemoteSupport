use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::{io::Write,path::{Path,PathBuf}};
use swan_agent::Bootstrap;
use swan_protocol::*;

#[cfg(windows)]
mod windows;

fn main()->Result<()> {
    #[cfg(windows)]
    if std::env::args().any(|argument|argument=="--service") {return windows::dispatch();}
    // This executable entrypoint owns the foreground runtime; SCM creates its
    // runtime on the separate service entrypoint thread before calling run.
    let (_stop,receiver)=tokio::sync::oneshot::channel();
    tokio::runtime::Runtime::new()?.block_on(run(receiver))
}

async fn run(mut stop:tokio::sync::oneshot::Receiver<()>)->Result<()> {
    tokio::select! {
        _=&mut stop=>Ok(()),
        outcome=run_worker()=>outcome,
    }
}

async fn run_worker()->Result<()> {
    ensure!(cfg!(windows),"Installer worker requires Windows for Authenticode verification");
    let server=std::env::var("SWAN_MANAGEMENT_URL").context("Set SWAN_MANAGEMENT_URL")?;https_url(&server)?;
    let token=std::env::var("SWAN_WORKER_TOKEN").context("Set SWAN_WORKER_TOKEN")?;
    let release_key=std::env::var("SWAN_RELEASE_PUBLIC_KEY").context("Pin SWAN_RELEASE_PUBLIC_KEY on the worker")?;public_key(&release_key)?;
    let company_key=std::env::var("SWAN_PROFILE_PUBLIC_KEY").context("Pin SWAN_PROFILE_PUBLIC_KEY on the worker")?;public_key(&company_key)?;
    let output=PathBuf::from(std::env::var("SWAN_ARTIFACT_DIR").context("Set worker artifact output directory")?);
    tokio::fs::create_dir_all(&output).await?;
    let client=swan_agent::http_client(600,false)?;
    let recipe=company_bundle_recipe()?;
    loop {
        // Older servers do not understand the claim header. Check their recipe
        // endpoint first so upgrading only the worker cannot strand their jobs.
        if let Err(error)=check_server_recipe(&client,&server,&token,&recipe).await {
            eprintln!("Worker compatibility check failed: {error:#}. Install matching server and worker releases; no build was requested.");
            tokio::time::sleep(std::time::Duration::from_secs(15)).await;
            continue;
        }
        let response=client.post(format!("{}/api/v1/worker/claim",server.trim_end_matches('/'))).bearer_auth(&token).header(COMPANY_BUNDLE_RECIPE_HEADER,&recipe).send().await;
        match response {
            Ok(response) if response.status().is_success()=>{
                if let Err(error)=process_claim(response,&client,&server,&token,&output,&release_key,&company_key).await {
                    eprintln!("Worker job processing failed: {error:#}. Continuing polling; the server retains the job for recovery.");
                }
            }
            Ok(response) if response.status()==reqwest::StatusCode::CONFLICT=>eprintln!("Company bundle recipe mismatch; install matching server and worker releases. No build was claimed."),
            Ok(response)=>eprintln!("Worker polling failed: {}",response.status()),
            Err(_)=>eprintln!("Worker management endpoint unavailable; retrying"),
        }
        tokio::time::sleep(std::time::Duration::from_secs(15)).await;
    }
}

async fn check_server_recipe(client:&reqwest::Client,server:&str,token:&str,recipe:&str)->Result<()> {
    let response=client.get(format!("{}/api/v1/worker/recipe",server.trim_end_matches('/'))).bearer_auth(token).send().await?.error_for_status()?;
    let value:Value=response.json().await?;
    ensure!(value["schema"].as_u64()==Some(1) && value["bundle_recipe"].as_str()==Some(recipe),"Company bundle recipe mismatch");
    Ok(())
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
                            upload_build(client,server,token,id,bytes,&hash).await?
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

#[derive(Debug)]
struct ControlRejection { status:reqwest::StatusCode,reason:&'static str }
impl std::fmt::Display for ControlRejection {
    fn fmt(&self,formatter:&mut std::fmt::Formatter<'_>)->std::fmt::Result {
        write!(formatter,"{}: {}",self.status,self.reason)
    }
}
impl std::error::Error for ControlRejection {}
impl ControlRejection {
    fn invalid_bundle(&self)->bool {
        matches!(self.status,reqwest::StatusCode::BAD_REQUEST|reqwest::StatusCode::PAYLOAD_TOO_LARGE|
            reqwest::StatusCode::UNSUPPORTED_MEDIA_TYPE|reqwest::StatusCode::UNPROCESSABLE_ENTITY)
    }
}

fn public_rejection_reason(body:&[u8])->&'static str {
    // Never copy arbitrary server text, credentials, URLs or headers into logs.
    const REASONS:&[&str]=&[
        "Unexpected ZIP footer","Unsupported ZIP entry layout","Ambiguous ZIP entry headers",
        "Unexpected bundle contents","Missing bundle file","Bundle script or license mismatch",
        "Bundle binary hash mismatch","Wrong company profile","Wrong release metadata",
        "Noncanonical bootstrap configuration","Invalid bootstrap configuration",
        "Expired or invalid bundle profile","Bootstrap trust mismatch","Expired release",
        "Artifact too large","Conflicting artifact retry","Duplicate ZIP entry",
        "Unaccounted ZIP contents","Invalid ZIP bounds","Invalid ZIP directory bounds",
        "Invalid bundle entry type or size","Bundle entry too large","Invalid ZIP artifact",
    ];
    const UNKNOWN:&str="Unrecognized server rejection; review company server logs";
    if body.len()>4096{return UNKNOWN;}
    let Ok(value)=serde_json::from_slice::<Value>(body) else{return UNKNOWN;};
    let Some(reason)=value.get("error").and_then(Value::as_str) else{return UNKNOWN;};
    REASONS.iter().copied().find(|candidate|*candidate==reason).unwrap_or(UNKNOWN)
}

async fn control_rejection(mut response:reqwest::Response)->ControlRejection {
    let status=response.status();
    let details=tokio::time::timeout(std::time::Duration::from_secs(2),async {
        let mut body=Vec::new();
        while let Some(chunk)=response.chunk().await? {
            ensure!(body.len()+chunk.len()<=4096,"Worker rejection details exceed diagnostic limit");
            body.extend_from_slice(&chunk);
        }
        Ok::<_,anyhow::Error>(body)
    }).await;
    let reason=match details {
        Ok(Ok(body))=>public_rejection_reason(&body),
        Ok(Err(_))=>"Server rejection details could not be read within the diagnostic limit",
        Err(_)=>"Server rejection details timed out",
    };
    ControlRejection{status,reason}
}

async fn upload_build(client:&reqwest::Client,server:&str,token:&str,id:&str,bytes:Vec<u8>,hash:&str)->Result<Value> {
    let uploaded=send_idempotent(client.put(format!("{}/api/v1/worker/builds/{id}/artifact",server.trim_end_matches('/')))
        .bearer_auth(token).header("Content-Type","application/zip").body(bytes)).await;
    match uploaded {
        Ok(())=>Ok(json!({"success":true,"artifact_url":format!("{}/api/v1/downloads/{id}",server.trim_end_matches('/')),"sha256":hash,"log":"Verified signed inputs and uploaded company bundle."})),
        Err(error) if error.downcast_ref::<ControlRejection>().is_some_and(ControlRejection::invalid_bundle)=>{
            eprintln!("Build {id} upload rejected: {error}");
            Ok(json!({"success":false,"artifact_url":"","sha256":"","log":format!("Bundle upload rejected ({error}). No package was published; retained output requires review.")}))
        },
        Err(error)=>Err(error),
    }
}

async fn send_idempotent(request:reqwest::RequestBuilder)->Result<()> {
    for attempt in 0..3 {
        let response=request.try_clone().context("Worker retry requires a buffered request")?.send().await;
        match response {
            Ok(response) if response.status().is_success()=>return Ok(()),
            Ok(response) if !response.status().is_server_error() && response.status()!=reqwest::StatusCode::TOO_MANY_REQUESTS=>{
                return Err(control_rejection(response).await.into());
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
    verify_artifacts(&installer,&agent,&release.format,|path,check_msi|verify_windows(path,&release,&work,check_msi))?;
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
    ] {zip.start_file(name,options)?;zip.write_all(&bytes)?;}
    for (name,bytes) in COMPANY_BUNDLE_TEXT_FILES {zip.start_file(name,options)?;zip.write_all(&bundle_text(bytes)?)?;}
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
fn verify_artifacts(installer:&Path,agent:&Path,format:&str,mut verify:impl FnMut(&Path,bool)->Result<()>)->Result<()> {
    verify(installer,format=="msi")?;
    verify(agent,false)
}
#[cfg(windows)]
fn verify_windows(path:&Path,release:&Release,directory:&Path,check_msi:bool)->Result<()> {
    let verifier=directory.join("Verify-Package.ps1");std::fs::write(&verifier,include_bytes!("../../../deployment/windows/Verify-Package.ps1"))?;
    let status=swan_agent::update::powershell_command()?.args(["-NoProfile","-NonInteractive","-ExecutionPolicy","Bypass","-File"]).arg(verifier).arg("-Path").arg(path).arg("-Publisher").arg(&release.publisher).arg("-CertificateSha256").arg(&release.publisher_certificate_sha256).status()?;
    ensure!(status.success(),"Artifact signature or publisher rejected");
    if check_msi {swan_agent::update::verify_msi_release_identity(path,release)?;}
    Ok(())
}

#[cfg(not(windows))]
fn verify_windows(_path:&Path,_release:&Release,_directory:&Path,_check_msi:bool)->Result<()> {
    anyhow::bail!("Artifact signature verification requires a Windows worker")
}

#[cfg(test)]
mod verification_tests {
    use super::*;

    async fn control_server(responses:Vec<(&'static str,String)>)->(String,tokio::task::JoinHandle<Vec<(String,Vec<u8>)>>) {
        use tokio::io::{AsyncReadExt,AsyncWriteExt};
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let address=listener.local_addr().unwrap();
        let server=tokio::spawn(async move {
            let mut requests=Vec::new();
            for (status,body) in responses {
                let (mut stream,_)=listener.accept().await.unwrap();let mut headers=Vec::new();
                while !headers.ends_with(b"\r\n\r\n") {
                    headers.push(stream.read_u8().await.unwrap());assert!(headers.len()<8192);
                }
                let headers=String::from_utf8(headers).unwrap();let lower=headers.to_ascii_lowercase();
                assert!(lower.contains("authorization: bearer fixture-worker\r\n"));
                let length=lower.lines().find_map(|line|line.strip_prefix("content-length:")).map(|value|value.trim().parse::<usize>().unwrap()).unwrap_or(0);
                assert!(length<65536);let mut request_body=vec![0;length];stream.read_exact(&mut request_body).await.unwrap();
                requests.push((headers.lines().next().unwrap().to_owned(),request_body));
                let response=format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
                stream.write_all(response.as_bytes()).await.unwrap();
            }
            requests
        });
        (format!("http://{address}"),server)
    }

    fn control_client()->reqwest::Client {
        reqwest::Client::builder().no_proxy().timeout(std::time::Duration::from_secs(3)).build().unwrap()
    }

    #[tokio::test]
    async fn permanent_bundle_rejection_is_reported_as_failure_without_a_download_link() {
        let (endpoint,server)=control_server(vec![
            ("400 Bad Request",json!({"error":"Unexpected ZIP footer"}).to_string()),
            ("200 OK",json!({"ok":true}).to_string()),
        ]).await;
        let id="00000000-0000-4000-8000-000000000001";let client=control_client();
        let result=upload_build(&client,&endpoint,"fixture-worker",id,b"retained original bundle".to_vec(),&"a".repeat(64)).await.unwrap();
        assert_eq!(result["success"],false);assert_eq!(result["artifact_url"],"");assert_eq!(result["sha256"],"");
        assert!(result["log"].as_str().unwrap().contains("Unexpected ZIP footer"));
        report_completion(&client,&endpoint,"fixture-worker",id,&result).await.unwrap();
        let requests=tokio::time::timeout(std::time::Duration::from_secs(5),server).await.unwrap().unwrap();
        assert_eq!(requests.len(),2);
        assert_eq!(requests[0].0,format!("PUT /api/v1/worker/builds/{id}/artifact HTTP/1.1"));
        assert_eq!(requests[0].1,b"retained original bundle");
        assert_eq!(requests[1].0,format!("PUT /api/v1/worker/builds/{id} HTTP/1.1"));
        assert_eq!(serde_json::from_slice::<Value>(&requests[1].1).unwrap(),result);
    }

    #[tokio::test]
    async fn validation_rejections_fail_but_authentication_and_conflicts_retain_recovery() {
        for (status,failed) in [
            ("400 Bad Request",true),("413 Payload Too Large",true),
            ("415 Unsupported Media Type",true),("422 Unprocessable Entity",true),
            ("401 Unauthorized",false),("403 Forbidden",false),("409 Conflict",false),
        ] {
            let (endpoint,server)=control_server(vec![(status,json!({"error":"fixture-worker private response https://example.invalid/?secret=value"}).to_string())]).await;
            let result=upload_build(&control_client(),&endpoint,"fixture-worker","fixture-job",b"original".to_vec(),&"a".repeat(64)).await;
            if failed {
                let result=result.unwrap();assert_eq!(result["success"],false);
                let log=result["log"].as_str().unwrap();assert!(!log.contains("fixture-worker") && !log.contains("secret=value") && !log.contains("example.invalid"));
            }else{
                let error=result.unwrap_err();let rejected=error.downcast_ref::<ControlRejection>().unwrap();
                assert!(!rejected.invalid_bundle());assert!(!error.to_string().contains("fixture-worker"));
            }
            assert_eq!(tokio::time::timeout(std::time::Duration::from_secs(5),server).await.unwrap().unwrap().len(),1);
        }
    }

    #[tokio::test]
    async fn transient_upload_errors_retry_identical_bytes_before_reporting_success() {
        let (endpoint,server)=control_server(vec![
            ("503 Service Unavailable","{}".into()),("429 Too Many Requests","{}".into()),
            ("200 OK","{}".into()),("200 OK","{}".into()),
        ]).await;
        let client=control_client();let id="00000000-0000-4000-8000-000000000002";let hash="a".repeat(64);
        let result=tokio::time::timeout(std::time::Duration::from_secs(25),upload_build(&client,&endpoint,"fixture-worker",id,b"same verified bytes".to_vec(),&hash)).await.unwrap().unwrap();
        assert_eq!(result["success"],true);assert_eq!(result["sha256"],hash);
        report_completion(&client,&endpoint,"fixture-worker",id,&result).await.unwrap();
        let requests=tokio::time::timeout(std::time::Duration::from_secs(5),server).await.unwrap().unwrap();
        assert_eq!(requests.len(),4);
        for request in &requests[..3]{assert_eq!(request.0,requests[0].0);assert_eq!(request.1,b"same verified bytes");}
        assert_eq!(serde_json::from_slice::<Value>(&requests[3].1).unwrap(),result);
    }

    #[test]
    fn server_error_details_are_bounded_and_never_reflect_arbitrary_remote_text() {
        assert_eq!(public_rejection_reason(br#"{"error":"Unexpected ZIP footer","private":"fixture-secret"}"#),"Unexpected ZIP footer");
        for body in [
            br#"{"error":"fixture-secret"}"#.to_vec(),
            br#"{"error":"Unexpected ZIP footer\nfixture-secret"}"#.to_vec(),
            br#"{"error":{"password":"fixture-secret"}}"#.to_vec(),
            b"not JSON fixture-secret".to_vec(),
            json!({"error":"Unexpected ZIP footer","private":"fixture-secret".repeat(1000)}).to_string().into_bytes(),
        ] {
            let reason=public_rejection_reason(&body);
            assert!(!reason.contains("fixture-secret"));assert_ne!(reason,"Unexpected ZIP footer");
        }
    }

    #[tokio::test]
    async fn server_recipe_preflight_rejects_old_or_incompatible_servers() {
        use tokio::io::{AsyncReadExt,AsyncWriteExt};
        let recipe=company_bundle_recipe().unwrap();
        let client=reqwest::Client::builder().no_proxy().timeout(std::time::Duration::from_secs(3)).build().unwrap();
        for (status,body,accepted) in [
            ("200 OK",json!({"schema":1,"bundle_recipe":recipe}).to_string(),true),
            ("200 OK",json!({"schema":1,"bundle_recipe":"old"}).to_string(),false),
            ("200 OK",json!({"schema":2,"bundle_recipe":recipe}).to_string(),false),
            ("200 OK","null".into(),false),
            ("404 Not Found","{}".into(),false),
            ("401 Unauthorized","{}".into(),false),
        ] {
            let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let address=listener.local_addr().unwrap();
            let server=tokio::spawn(async move {
                let (mut stream,_)=listener.accept().await.unwrap();let mut request=Vec::new();
                while !request.ends_with(b"\r\n\r\n") {
                    let byte=stream.read_u8().await.unwrap();request.push(byte);assert!(request.len()<4096);
                }
                let request=String::from_utf8(request).unwrap();
                assert!(request.starts_with("GET /api/v1/worker/recipe HTTP/1.1\r\n"));
                assert!(request.to_ascii_lowercase().contains("authorization: bearer fixture-worker\r\n"));
                let response=format!("HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",body.len());
                stream.write_all(response.as_bytes()).await.unwrap();
            });
            assert_eq!(check_server_recipe(&client,&format!("http://{address}"),"fixture-worker",&recipe).await.is_ok(),accepted);
            tokio::time::timeout(std::time::Duration::from_secs(5),server).await.unwrap().unwrap();
        }
    }

    #[test]
    fn both_artifacts_require_trust_but_only_msi_installer_requires_msi_identity() {
        let agent=Path::new("swan-agent.exe");
        for format in ["exe","msi"] {
            let installer=PathBuf::from(format!("installer.{format}"));
            let mut checked=Vec::new();
            verify_artifacts(&installer,agent,format,|path,msi|{checked.push((path.to_owned(),msi));Ok(())}).unwrap();
            assert_eq!(checked,vec![(installer.clone(),format=="msi"),(agent.to_owned(),false)]);
            for rejected in [&installer,agent] {
                assert!(verify_artifacts(&installer,agent,format,|path,_|{
                    ensure!(path!=rejected,"Untrusted artifact");Ok(())
                }).is_err());
            }
        }
    }
}
