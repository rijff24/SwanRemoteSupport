use super::service::*;
use axum::{body::Body,http::{Request,StatusCode}};
use base64::{engine::general_purpose::STANDARD,Engine};
use ed25519_dalek::SigningKey;
use http_body_util::BodyExt;
use serde_json::{json,Value};
use std::sync::Arc;
use swan_protocol::*;
use totp_rs::{Algorithm,Secret,TOTP};
use tower::ServiceExt;

async fn request(app:&axum::Router,path:&str,method:&str,token:Option<&str>,body:Value)->(StatusCode,Value) {
    let mut request=Request::builder().uri(path).method(method).header("content-type","application/json");
    if let Some(token)=token {request=request.header("authorization",format!("Bearer {token}"));}
    let response=app.clone().oneshot(request.body(Body::from(body.to_string())).unwrap()).await.unwrap();
    let status=response.status();let bytes=response.into_body().collect().await.unwrap().to_bytes();
    (status,serde_json::from_slice(&bytes).unwrap())
}
fn profile_fixture()->CompanyProfile {
    let key=SigningKey::from_bytes(&[3;32]);
    CompanyProfile{schema:1,company_id:"fixture".into(),revision:1,issued_at:now(),expires_at:now()+3600,management_url:"https://support.example.com".into(),rendezvous:"support.example.com".into(),relay:"support.example.com".into(),transport_public_key:STANDARD.encode(key.verifying_key().as_bytes()),customer:Branding::default(),technician:Branding::default(),allow_unattended:true,updates_paused:true,rollout_percent:10,maintenance_start_utc:22,maintenance_end_utc:5,update_channel:"stable".into(),next_profile_public_key:None}
}

#[tokio::test]
async fn enrollment_mfa_and_grants_fail_closed() {
    let directory=std::env::temp_dir().join(format!("swan-api-{}",random_token()));
    let store=Arc::new(Store::open(&directory).unwrap());let app=router(store.clone());
    assert_eq!(request(&app,"/api/v1/devices","GET",None,Value::Null).await.0,StatusCode::UNAUTHORIZED);
    let secret=Secret::generate_secret().to_encoded().to_string();
    let generator=TOTP::new(Algorithm::SHA1,6,1,30,Secret::Encoded(secret.clone()).to_bytes().unwrap()).unwrap();
    let time=now();let setup_token=std::fs::read_to_string(directory.join("setup-token.txt")).unwrap();
    let setup=json!({"profile":profile_fixture(),"username":"admin","password":"correct horse battery staple","totp_secret":secret,"totp_code":generator.generate(time)});
    assert_eq!(request(&app,"/api/v1/setup","POST",Some(&random_token()),setup.clone()).await.0,StatusCode::FORBIDDEN);
    assert_eq!(request(&app,"/api/v1/setup","POST",Some(&setup_token),setup.clone()).await.0,StatusCode::OK);
    assert_eq!(request(&app,"/api/v1/setup","POST",Some(&setup_token),setup).await.0,StatusCode::CONFLICT);
    let login=json!({"username":"admin","password":"correct horse battery staple","totp_code":generator.generate(time)});
    assert_eq!(request(&app,"/api/v1/login","POST",None,login).await.0,StatusCode::UNAUTHORIZED,"Setup code cannot be replayed for login");
    let login=json!({"username":"admin","password":"correct horse battery staple","totp_code":generator.generate((time/30+1)*30)});
    let (status,result)=request(&app,"/api/v1/login","POST",None,login.clone()).await;assert_eq!(status,StatusCode::OK);
    let admin=result["token"].as_str().unwrap();
    assert_eq!(request(&app,"/api/v1/sessions","GET",None,Value::Null).await.0,StatusCode::UNAUTHORIZED);
    assert_eq!(request(&app,"/api/v1/login","POST",None,login).await.0,StatusCode::UNAUTHORIZED,"MFA codes are single-use");
    let (_,device)=request(&app,"/api/v1/enroll","POST",None,json!({"name":"Customer test PC","rustdesk_id":"123456789","unattended_consent":false})).await;
    let device_id=device["device_id"].as_str().unwrap();let credential=device["device_token"].as_str().unwrap();
    let (status,identity)=request(&app,"/api/v1/device/status","GET",Some(credential),Value::Null).await;
    assert_eq!(status,StatusCode::OK);assert_eq!(identity["device_id"],device_id);assert_eq!(identity["rustdesk_id"],"123456789");assert_eq!(identity["state"],"pending");
    let (status,_)=request(&app,"/api/v1/device/status","GET",Some("wrong-device-token"),Value::Null).await;
    assert_eq!(status,StatusCode::UNAUTHORIZED);
    let proof=SigningKey::from_bytes(&[9;32]);let grant_request=json!({"device_id":device_id,"proof_public_key":STANDARD.encode(proof.verifying_key().as_bytes()),"unattended":false});
    assert_eq!(request(&app,"/api/v1/grants","POST",Some(admin),grant_request.clone()).await.0,StatusCode::FORBIDDEN,"Pending devices cannot be accessed");
    assert_eq!(request(&app,&format!("/api/v1/devices/{device_id}/state"),"PUT",Some(admin),json!({"state":"approved","group":"customers"})).await.0,StatusCode::OK);
    let (status,envelope)=request(&app,"/api/v1/grants","POST",Some(admin),grant_request).await;assert_eq!(status,StatusCode::OK);
    let claim=json!({"grant":envelope});
    assert_eq!(request(&app,"/api/v1/grants/claim","POST",Some(credential),claim.clone()).await.0,StatusCode::OK);
    let original:SignedEnvelope=serde_json::from_value(envelope.clone()).unwrap();
    let issued:SessionGrant=serde_json::from_slice(&STANDARD.decode(&original.payload).unwrap()).unwrap();
    assert!(issued.permissions.keyboard);assert!(!issued.permissions.recording);
    let mut reduced=SessionPermissions::support_default();reduced.keyboard=false;
    assert_eq!(request(&app,"/api/v1/groups/customers/permissions","PUT",Some(admin),serde_json::to_value(&reduced).unwrap()).await.0,StatusCode::OK);
    assert_eq!(request(&app,&format!("/api/v1/grants/{}/renew",issued.grant_id),"POST",Some(credential),Value::Null).await.0,StatusCode::FORBIDDEN,"Reduced capabilities invalidate existing authorization leases");
    assert_eq!(request(&app,"/api/v1/groups/customers/permissions","PUT",None,serde_json::to_value(&reduced).unwrap()).await.0,StatusCode::UNAUTHORIZED);
    assert_eq!(request(&app,"/api/v1/groups/customers/permissions","PUT",Some(admin),serde_json::to_value(SessionPermissions::support_default()).unwrap()).await.0,StatusCode::OK);
    assert_eq!(request(&app,"/api/v1/grants/claim","POST",Some(credential),claim).await.0,StatusCode::FORBIDDEN,"Session ticket is single-use");
    let (_,history)=request(&app,"/api/v1/sessions","GET",Some(admin),Value::Null).await;
    assert_eq!(history.as_array().unwrap().len(),1);
    assert_eq!(history[0]["claimed"],true);
    assert_eq!(request(&app,"/api/v1/grants","POST",Some(admin),json!({"device_id":device_id,"proof_public_key":STANDARD.encode(proof.verifying_key().as_bytes()),"unattended":true})).await.0,StatusCode::FORBIDDEN,"Unattended access needs device consent");
    assert_eq!(request(&app,"/api/v1/device/consent","PUT",Some(credential),json!({"unattended":true,"revision":1})).await.0,StatusCode::OK);
    assert_eq!(request(&app,"/api/v1/device/consent","PUT",Some(credential),json!({"unattended":false,"revision":2})).await.0,StatusCode::OK);
    assert_eq!(request(&app,"/api/v1/device/consent","PUT",Some(credential),json!({"unattended":true,"revision":1})).await.0,StatusCode::CONFLICT,"Delayed enable cannot undo revocation");
    assert_eq!(request(&app,"/api/v1/device/consent","PUT",Some(credential),json!({"unattended":true,"revision":2})).await.0,StatusCode::CONFLICT,"Revocation wins same-revision races");
    assert_eq!(request(&app,"/api/v1/device/consent","PUT",Some(credential),json!({"unattended":true,"revision":3})).await.0,StatusCode::OK);
    assert_eq!(request(&app,"/api/v1/device/consent","PUT",Some(credential),json!({"unattended":false,"revision":2})).await.0,StatusCode::CONFLICT,"Old retry cannot undo newer explicit consent");
    assert_eq!(request(&app,"/api/v1/device/consent","PUT",Some(credential),json!({"unattended":false,"revision":4})).await.0,StatusCode::OK);
    assert_eq!(request(&app,"/api/v1/device/consent","PUT",Some(credential),json!({"unattended":false,"revision":4})).await.0,StatusCode::OK,"Revocation retries are idempotent");
    let (_,new_user)=request(&app,"/api/v1/users","POST",Some(admin),json!({"username":"technician","password":"another long strong password","role":"technician"})).await;
    let tech_secret=new_user["totp_secret"].as_str().unwrap();
    let tech_generator=TOTP::new(Algorithm::SHA1,6,1,30,Secret::Encoded(tech_secret.into()).to_bytes().unwrap()).unwrap();
    let (_,tech_login)=request(&app,"/api/v1/login","POST",None,json!({"username":"technician","password":"another long strong password","totp_code":tech_generator.generate(now())})).await;
    let technician=tech_login["token"].as_str().unwrap();
    assert_eq!(request(&app,"/api/v1/users","GET",Some(technician),Value::Null).await.0,StatusCode::UNAUTHORIZED);
    assert_eq!(request(&app,"/api/v1/grants","POST",Some(technician),json!({"device_id":device_id,"proof_public_key":STANDARD.encode(proof.verifying_key().as_bytes()),"unattended":false})).await.0,StatusCode::FORBIDDEN,"Technicians need explicit group access");
    let (_,devices)=request(&app,"/api/v1/devices","GET",Some(technician),Value::Null).await;assert_eq!(devices.as_array().unwrap().len(),0);
    assert_eq!(request(&app,&format!("/api/v1/groups/customers/users/{}",new_user["id"].as_str().unwrap()),"PUT",Some(admin),Value::Null).await.0,StatusCode::OK);
    let (_,devices)=request(&app,"/api/v1/devices","GET",Some(technician),Value::Null).await;assert_eq!(devices.as_array().unwrap().len(),1);
    assert_eq!(request(&app,&format!("/api/v1/devices/{device_id}/state"),"PUT",Some(admin),json!({"state":"revoked","group":"customers"})).await.0,StatusCode::OK);
    assert_eq!(request(&app,"/api/v1/device/status","GET",Some(credential),Value::Null).await.0,StatusCode::UNAUTHORIZED,"Repair cannot restore a revoked device");
    assert_eq!(request(&app,"/api/v1/grants","POST",Some(admin),json!({"device_id":device_id,"proof_public_key":STANDARD.encode(proof.verifying_key().as_bytes()),"unattended":false})).await.0,StatusCode::FORBIDDEN);
    drop(app);drop(store);std::fs::remove_dir_all(directory).unwrap();
}

#[tokio::test]
async fn profile_revision_is_server_controlled_and_invalid_svg_rejected() {
    let mut brand=Branding::default();brand.logo_svg="<svg><script>alert(1)</script></svg>".into();assert!(brand.validate().is_err());
    let mut profile=profile_fixture();assert!(profile.validate("fixture",1,now()).is_ok());
    assert!(profile.validate("wrong-company",1,now()).is_err());assert!(profile.validate("fixture",2,now()).is_err());
    profile.expires_at=now();assert!(profile.validate("fixture",1,now()).is_err());
}
