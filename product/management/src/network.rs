//! Server-side diagnostics are not evidence of reachability from the Internet.
use serde_json::{json,Value};
use std::{net::IpAddr,time::Duration};
use swan_protocol::{CompanyProfile,SignedEnvelope};

fn address_class(address:IpAddr)->&'static str {
    let private=match address {
        IpAddr::V4(ip)=>{let octets=ip.octets();ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_unspecified() || ip.is_multicast() || ip.is_broadcast() || (octets[0]==100 && (64..=127).contains(&octets[1])) || ip.is_documentation()},
        IpAddr::V6(ip)=>ip.is_loopback() || ip.is_unspecified() || ip.is_multicast() || ip.is_unique_local() || ip.is_unicast_link_local() || ip.to_ipv4().map(|ip|address_class(IpAddr::V4(ip))=="private_or_local").unwrap_or(false),
    };
    if private {"private_or_local"}else{"public_candidate"}
}

async fn tcp(name:&str,host:&str,port:u16)->Value {
    let resolved=tokio::time::timeout(Duration::from_secs(3),tokio::net::lookup_host((host,port))).await;
    let addresses:Vec<_>=match resolved {Ok(Ok(addresses))=>addresses.take(8).collect(),_=>return json!({"name":name,"host":host,"port":port,"reachable":false,"result":"DNS resolution failed or timed out","addresses":[]})};
    let visible:Vec<_>=addresses.iter().map(|address|json!({"ip":address.ip().to_string(),"classification":address_class(address.ip())})).collect();
    for address in addresses.iter().take(4) {
        if matches!(tokio::time::timeout(Duration::from_secs(3),tokio::net::TcpStream::connect(address)).await,Ok(Ok(_))) {
            return json!({"name":name,"host":host,"port":port,"reachable":true,"result":"TCP connection succeeded from this company server","addresses":visible});
        }
    }
    json!({"name":name,"host":host,"port":port,"reachable":false,"result":"TCP connection failed or timed out","addresses":visible})
}

async fn management(profile:&CompanyProfile,key:&ed25519_dalek::VerifyingKey)->bool {
    let result=async {
        let envelope:SignedEnvelope=swan_agent::http_client(5,false)?.get(format!("{}/api/v1/profile",profile.management_url.trim_end_matches('/'))).send().await?.error_for_status()?.json().await?;
        let returned:CompanyProfile=envelope.verify(key)?;
        returned.validate(&profile.company_id,profile.revision,swan_protocol::now())?;
        anyhow::Ok(())
    }.await;
    result.is_ok()
}

pub async fn check(profile:&CompanyProfile,key:&ed25519_dalek::VerifyingKey)->anyhow::Result<Value> {
    let (host,id_port)=swan_protocol::transport_socket(&profile.rendezvous,21116)?;
    let (relay_host,relay_port)=swan_protocol::transport_socket(&profile.relay,21117)?;
    let nat_port=id_port.checked_sub(1).filter(|port|*port>0).ok_or_else(||anyhow::anyhow!("ID server port must allow its preceding NAT-test port"))?;
    let (nat,id,relay,https)=tokio::join!(tcp("NAT-test TCP",&host,nat_port),tcp("Rendezvous TCP",&host,id_port),tcp("Relay TCP",&relay_host,relay_port),management(profile,key));
    Ok(json!({"vantage_point":"company_server","management_url":profile.management_url,"https_and_company_signature_valid":https,"tcp_checks":[nat,id,relay],"external_reachability_verified":false,"udp_reachability_verified":false,"guidance":"Repeat native direct and relay tests from outside this network. Office hosting needs appropriate port forwarding; CGNAT may require a company-controlled public relay. HTTPS-only proxies are not guaranteed. UDP 21116 (or your custom ID port) still requires a separate external test."}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn tcp_diagnostics_detect_local_listener_and_closed_port_without_claiming_public_access(){
        let listener=tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();let port=listener.local_addr().unwrap().port();
        let result=tcp("test","127.0.0.1",port).await;assert_eq!(result["reachable"],true);assert_eq!(result["addresses"][0]["classification"],"private_or_local");
        drop(listener);assert_eq!(tcp("test","127.0.0.1",port).await["reachable"],false);
        assert_eq!(address_class("100.64.1.2".parse().unwrap()),"private_or_local");assert_eq!(address_class("::1".parse().unwrap()),"private_or_local");
        assert_eq!(swan_protocol::transport_socket("support.example.com:443",21116).unwrap(),("support.example.com".into(),443));
        assert!(swan_protocol::transport_socket("support.example.com:0",21116).is_err());
    }
}
