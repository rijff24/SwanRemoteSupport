//! A company-run probe: no accounts, enrollment, database or private server keys.
use anyhow::{ensure,Context,Result};
use serde_json::{json,Value};
use std::{path::{Path,PathBuf},time::Duration};
use swan_agent::Bootstrap;
use swan_protocol::{CompanyProfile,SignedEnvelope};
use tokio::io::AsyncReadExt;

pub const USAGE:&str="Usage: swan-management check-network BOOTSTRAP_JSON [--outside-network] [--ca-file CERTIFICATE]\nRun on a separate Internet connection for outside-network checks. The location flag is an operator declaration, not independent attestation. No company account or server data directory is needed.";

struct Options{bootstrap:PathBuf,ca:Option<PathBuf>,outside:bool}
impl Options{
    fn parse(args:&[String])->Result<Self>{
        let first=args.first().filter(|x|!x.starts_with('-')).context(USAGE)?;
        let mut result=Self{bootstrap:first.into(),ca:None,outside:false};let mut i=1;
        while i<args.len(){match args[i].as_str(){
            "--outside-network"=>{ensure!(!result.outside,"Duplicate outside-network flag");result.outside=true;},
            "--ca-file"=>{ensure!(result.ca.is_none(),"Duplicate CA file");i+=1;result.ca=Some(args.get(i).filter(|x|!x.starts_with('-')).context("CA certificate path missing")?.into());},
            _=>anyhow::bail!("{USAGE}"),
        };i+=1;}
        Ok(result)
    }
}
async fn read_limited(path:&Path,limit:u64)->Result<Vec<u8>>{
    let file=tokio::fs::File::open(path).await.context("Open probe input")?;let mut bytes=Vec::new();
    file.take(limit+1).read_to_end(&mut bytes).await?;ensure!(bytes.len() as u64<=limit,"Probe input exceeds size limit");Ok(bytes)
}
pub async fn run(args:&[String])->Result<Value>{
    let options=Options::parse(args)?;
    let bootstrap:Bootstrap=serde_json::from_slice(&read_limited(&options.bootstrap,16*1024).await?).context("Expected public installer or network bootstrap JSON")?;
    bootstrap.validate()?;
    let management=swan_protocol::https_url(&bootstrap.management_url)?;
    ensure!(management.query().is_none(),"Management base URL must not contain query parameters");
    let mut client=reqwest::Client::builder().https_only(true).redirect(reqwest::redirect::Policy::none()).connect_timeout(Duration::from_secs(3)).timeout(Duration::from_secs(5));
    if let Some(path)=options.ca {
        let bytes=read_limited(&path,1024*1024).await?;
        let certificate=reqwest::Certificate::from_pem(&bytes).or_else(|_|reqwest::Certificate::from_der(&bytes)).context("Invalid company CA certificate")?;
        client=client.add_root_certificate(certificate);
    }
    let mut response=client.build()?.get(format!("{}/api/v1/profile",bootstrap.management_url.trim_end_matches('/'))).send().await.context("Company HTTPS request failed")?.error_for_status()?;
    let mut bytes=Vec::new();while let Some(chunk)=response.chunk().await?{
        ensure!(bytes.len().saturating_add(chunk.len())<=2*1024*1024,"Company profile response exceeds size limit");bytes.extend_from_slice(&chunk);
    }
    let envelope:SignedEnvelope=serde_json::from_slice(&bytes)?;
    let profile:CompanyProfile=envelope.verify(&swan_protocol::public_key(&bootstrap.profile_public_key)?)?;
    profile.validate(&bootstrap.company_id,0,swan_protocol::now())?;
    ensure!(profile.management_url.trim_end_matches('/')==bootstrap.management_url.trim_end_matches('/'),"Company endpoint changed; obtain a fresh public bootstrap");
    let (host,id_port)=swan_protocol::transport_socket(&profile.rendezvous,21116)?;
    let (relay_host,relay_port)=swan_protocol::transport_socket(&profile.relay,21117)?;
    let nat_port=id_port.checked_sub(1).filter(|p|*p>0).context("NAT-test TCP port is invalid")?;
    let https_host=management.host_str().context("Management hostname missing")?;
    let https_port=management.port_or_known_default().context("Management HTTPS port missing")?;
    let (https,nat,id,relay,udp)=tokio::join!(
        super::network::tcp("Management HTTPS TCP",https_host,https_port),super::network::tcp("NAT-test TCP",&host,nat_port),
        super::network::tcp("Rendezvous TCP",&host,id_port),super::network::tcp("Relay TCP",&relay_host,relay_port),super::network::udp(&host,id_port));
    let tcp=[https,nat,id,relay];
    let reachable=tcp.iter().all(|x|x["reachable"]==true) && udp["reachable"]==true;
    let public=tcp.iter().all(|x|x["public_path_candidate"]==true) && udp["public_path_candidate"]==true;
    Ok(json!({"vantage_point":"standalone_probe","outside_network_declared_by_operator":options.outside,"outside_vantage_independently_attested":false,
        "company_id":profile.company_id,"profile_revision":profile.revision,"management_url":profile.management_url,"https_and_company_signature_valid":true,
        "tcp_checks":tcp,"udp_check":udp,"all_endpoint_checks_passed":reachable,"connected_addresses_public_candidates":public,
        "requested_checks_passed":reachable && (!options.outside || public),"native_direct_and_relay_sessions_verified":false,
        "guidance":"Results describe reachability from this probe computer. The outside-network flag does not prove its location. Private/special-use routes fail the outside-network check. Test real customer-approved direct and relay sessions from a separate Internet connection before publishing downloads; CGNAT may require a company-controlled public relay. No shared hosted service is used."}))
}

#[cfg(test)]
mod tests{
    use super::*;
    #[test]
    fn invalid_probe_options_do_not_silently_start_a_server(){
        let args=|s:&[&str]|s.iter().map(|s|s.to_string()).collect::<Vec<_>>();
        assert!(Options::parse(&[]).is_err());assert!(Options::parse(&args(&["--outside-network"])).is_err());
        assert!(Options::parse(&args(&["bootstrap.json","--ca-file"])).is_err());
        assert!(Options::parse(&args(&["bootstrap.json","--outside-network","--outside-network"])).is_err());
        assert!(Options::parse(&args(&["bootstrap.json","--insecure"])).is_err());
        let value=Options::parse(&args(&["bootstrap.json","--ca-file","root.pem","--outside-network"])).unwrap();assert!(value.outside);assert_eq!(value.ca,Some(PathBuf::from("root.pem")));
    }
}
