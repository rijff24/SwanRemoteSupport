//! Server-side diagnostics are not evidence of reachability from the Internet.
use serde_json::{json,Value};
use std::{net::IpAddr,time::Duration};
use swan_protocol::{CompanyProfile,SignedEnvelope};

fn address_class(address:IpAddr)->&'static str {
    let private=match address {
        IpAddr::V4(ip)=>{let o=ip.octets();ip.is_private() || ip.is_loopback() || ip.is_link_local() || ip.is_documentation() || o[0]==0 || o[0]>=224 || (o[0]==100 && (64..=127).contains(&o[1])) || (o[0]==198 && (18..=19).contains(&o[1])) || (o[0]==192 && o[1]==0 && o[2]==0) || (o[0]==192 && o[1]==88 && o[2]==99)},
        // Treat special-use and documentation ranges conservatively. A public
        // candidate is still not independent proof of an external vantage.
        IpAddr::V6(ip)=>if let Some(v4)=ip.to_ipv4(){address_class(IpAddr::V4(v4))!="public_candidate"}else{let s=ip.segments();s[0]&0xe000!=0x2000 || (s[0]==0x2001 && (s[1]<0x0200 || s[1]==0x0db8)) || s[0]==0x2002 || (s[0]==0x3fff && s[1]&0xf000==0)},
    };
    if private {"private_or_local"}else{"public_candidate"}
}

pub(super) async fn tcp(name:&str,host:&str,port:u16)->Value {
    let resolved=tokio::time::timeout(Duration::from_secs(3),tokio::net::lookup_host((host,port))).await;
    let addresses:Vec<_>=match resolved {Ok(Ok(addresses))=>addresses.take(8).collect(),_=>return json!({"name":name,"host":host,"port":port,"reachable":false,"result":"DNS resolution failed or timed out","addresses":[]})};
    let visible:Vec<_>=addresses.iter().map(|address|json!({"ip":address.ip().to_string(),"classification":address_class(address.ip())})).collect();
    for address in addresses.iter().take(4) {
        if matches!(tokio::time::timeout(Duration::from_secs(3),tokio::net::TcpStream::connect(address)).await,Ok(Ok(_))) {
            return json!({"name":name,"host":host,"port":port,"reachable":true,"result":"TCP connection succeeded from this probe machine","connected_ip":address.ip().to_string(),"public_path_candidate":address_class(address.ip())=="public_candidate","addresses":visible});
        }
    }
    json!({"name":name,"host":host,"port":port,"reachable":false,"result":"TCP connection failed or timed out","addresses":visible})
}

// The pinned UDP handler ignores TestNatRequest (TCP only). An empty-ID
// RegisterPk with nonempty placeholder uuid/pk is rejected BEFORE peer lookup,
// rate-state changes or database access. This is a stateless reachability probe,
// never an enrollment or key registration. UDP carries raw protobuf.
const UDP_REQUEST:&[u8]=&[0x7a,0x06,0x12,0x01,0x00,0x1a,0x01,0x00];
pub(super) async fn udp(host:&str,port:u16)->Value {
    let addresses:Vec<_>=match tokio::time::timeout(Duration::from_secs(3),tokio::net::lookup_host((host,port))).await {
        Ok(Ok(addresses))=>addresses.take(8).collect(),
        _=>return json!({"host":host,"port":port,"reachable":false,"result":"UDP DNS resolution failed or timed out"}),
    };
    for address in addresses.iter().take(4){
        let attempt=async {
            let socket=tokio::net::UdpSocket::bind(if address.is_ipv4(){"0.0.0.0:0"}else{"[::]:0"}).await?;
            socket.connect(address).await?;socket.send(UDP_REQUEST).await?;
            let mut buffer=[0u8;4096];let count=socket.recv(&mut buffer).await?;
            anyhow::Ok(buffer[..count]==[0x82,0x01,0x02,0x08,0x02])
        };
        if let Ok(Ok(true))=tokio::time::timeout(Duration::from_secs(3),attempt).await {
            return json!({"host":host,"port":port,"reachable":true,"result":"Native UDP invalid-ID rejection received from this probe machine","probe":"stateless_empty_id_rejection","connected_ip":address.ip().to_string(),"public_path_candidate":address_class(address.ip())=="public_candidate","response_authenticated":false,"nat_traversal_verified":false});
        }
    }
    json!({"host":host,"port":port,"reachable":false,"result":"Native UDP response missing, malformed or timed out","nat_traversal_verified":false})
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
    let (nat,id,relay,https,udp)=tokio::join!(tcp("NAT-test TCP",&host,nat_port),tcp("Rendezvous TCP",&host,id_port),tcp("Relay TCP",&relay_host,relay_port),management(profile,key),udp(&host,id_port));
    Ok(json!({"vantage_point":"company_server","management_url":profile.management_url,"https_and_company_signature_valid":https,"tcp_checks":[nat,id,relay],"external_reachability_verified":false,"udp_reachability_verified":udp["reachable"]==true,"udp_check":udp,"guidance":"Run the standalone check-network command on a separate Internet connection, then test real native direct and relay sessions. Server-side checks do not prove external access. Office hosting needs port forwarding; CGNAT may require a company-controlled public relay. HTTPS-only proxies are not guaranteed."}))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn special_use_addresses_are_not_public_candidates(){
        for ip in ["0.0.0.1","198.18.0.1","240.0.0.1","2001:db8::1","3fff::1","fec0::1","::ffff:192.168.1.1"]{assert_eq!(address_class(ip.parse().unwrap()),"private_or_local");}
        for ip in ["8.8.8.8","2606:4700:4700::1111"]{assert_eq!(address_class(ip.parse().unwrap()),"public_candidate");}

    }
    #[tokio::test]
    async fn udp_probe_requires_a_native_reply_from_its_connected_endpoint(){
        for (reply,expected) in [(vec![0x82,0x01,0x02,0x08,0x02],true),(vec![0x0a,0],false)]{
            let server=tokio::net::UdpSocket::bind("127.0.0.1:0").await.unwrap();let port=server.local_addr().unwrap().port();
            let task=tokio::spawn(async move{let mut bytes=[0u8;64];let (n,peer)=server.recv_from(&mut bytes).await.unwrap();assert_eq!(&bytes[..n],UDP_REQUEST);server.send_to(&reply,peer).await.unwrap();});
            let value=udp("127.0.0.1",port).await;assert_eq!(value["reachable"],expected);assert_ne!(value["public_path_candidate"],true);task.await.unwrap();
        }
    }
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
