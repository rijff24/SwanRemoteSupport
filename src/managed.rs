//! Managed edition authorization is enforced on the receiving device, independent
//! of the ID server, relay, direct transport, or customer approval UI.
use hbb_common::{config::{self, keys}, ResultType};
use swan_agent::{protocol::{self, Edition, ManagedLogin, SignedEnvelope}, AgentState, Lease};

struct TechnicianLogin {token:String,company:String,username:String}
struct PendingTicket {envelope:SignedEnvelope,key_hex:String,expires_at:u64,company:String,peer:String}
#[derive(Clone)]
struct ReconnectRequest {device:String,unattended:bool,company:String,peer:String,generation:u64,expires_at:u64}
#[derive(Default)]
struct TechnicianMemory {
    generation:u64,
    login:Option<TechnicianLogin>,
    tickets:std::collections::HashMap<String,PendingTicket>,
    reconnects:std::collections::HashMap<String,ReconnectRequest>,
    update_running:bool,
    update_handed_off:bool,
    update_failed:bool,
}
hbb_common::lazy_static::lazy_static! {
    static ref TECHNICIAN:std::sync::Mutex<TechnicianMemory>=Default::default();
}

pub async fn technician_request(request:&str)->String {
    match technician_request_inner(request).await {
        Ok(value)=>serde_json::json!({"ok":true,"data":value}).to_string(),
        // Avoid including HTTP request or credential details in UI errors/logs.
        Err(_)=>serde_json::json!({"ok":false,"error":"Company request failed. Check login, authenticator code, permissions and server availability."}).to_string(),
    }
}

async fn technician_request_inner(request:&str)->ResultType<serde_json::Value> {
    use serde_json::json;
    use hbb_common::bail;
    use hbb_common::anyhow::anyhow;
    if request.len()>8192 {bail!("Request too large");}
    let input:serde_json::Value=serde_json::from_str(request)?;
    let action=input["action"].as_str().ok_or_else(||anyhow!("Missing action"))?;
    if action=="customer-consent" {
        let enabled=input["enabled"].as_bool().ok_or_else(||anyhow!("Missing consent choice"))?;
        return crate::ipc::set_company_consent(enabled).await;
    }
    let directory=swan_agent::state_directory();
    let state=AgentState::load_for_refresh(&directory)?;
    if state.bootstrap.edition!=Edition::Technician {bail!("Technician edition required");}
    if action=="update-progress" {
        let memory=TECHNICIAN.lock().unwrap();
        return Ok(json!({"running":memory.update_running,"handed_off":memory.update_handed_off,"failed":memory.update_failed}));
    }
    #[cfg(all(windows,feature="flutter"))]
    if action=="resume-update" {
        // Recovery can finish an already authorized installation offline. It
        // cannot select a new release and does not require a reusable login.
        let pending=directory.join("pending-update.json").exists();
        {let memory=TECHNICIAN.lock().unwrap();
            if memory.update_running || memory.update_handed_off{return Ok(json!({"handed_off":memory.update_handed_off,"pending":pending}));}}
        if !pending || !crate::flutter::sessions::get_sessions().is_empty(){return Ok(json!({"handed_off":false,"pending":pending}));}
        let handed_off=hbb_common::tokio::task::spawn_blocking(move ||state.resume_pending_update(&directory,false)).await??;
        return Ok(json!({"handed_off":handed_off,"pending":true}));
    }
    if action=="sync" {
        let mut state=state;state.sync().await?;state.save(&directory)?;
        return Ok(json!({"revision":state.accepted_revision}));
    }
    if action=="status" {
        let memory=TECHNICIAN.lock().unwrap();
        return Ok(json!({"logged_in":memory.login.as_ref().map(|l|l.company==state.bootstrap.company_id).unwrap_or(false),"username":memory.login.as_ref().map(|l|l.username.as_str())}));
    }
    if action=="login" {
        let username=input["username"].as_str().ok_or_else(||anyhow!("Missing username"))?;
        let password=input["password"].as_str().ok_or_else(||anyhow!("Missing password"))?;
        let code=input["code"].as_str().ok_or_else(||anyhow!("Missing code"))?;
        let generation={let mut memory=TECHNICIAN.lock().unwrap();memory.generation=memory.generation.wrapping_add(1);memory.login=None;memory.tickets.clear();memory.reconnects.clear();memory.generation};
        let token=state.technician_login(username,password,code).await?;
        let accepted={let mut memory=TECHNICIAN.lock().unwrap();
            if memory.generation==generation {memory.login=Some(TechnicianLogin{token:token.clone(),company:state.bootstrap.company_id.clone(),username:username.into()});true}else{false}
        };
        if !accepted {state.technician_logout(&token).await?;bail!("Login superseded");}
        return Ok(json!({"username":username}));
    }
    let (token,generation)={let memory=TECHNICIAN.lock().unwrap();
        let login=memory.login.as_ref().ok_or_else(||anyhow!("Login required"))?;
        if login.company!=state.bootstrap.company_id {bail!("Wrong company login");}
        (login.token.clone(),memory.generation)
    };
    if action=="logout" {
        {let mut memory=TECHNICIAN.lock().unwrap();memory.generation=memory.generation.wrapping_add(1);memory.login=None;memory.tickets.clear();memory.reconnects.clear();}
        // Local logout is immediate even if the server cannot receive revocation.
        state.technician_logout(&token).await?;
        return Ok(json!({"logged_in":false}));
    }
    if action=="devices" {return Ok(state.technician_inventory(&token).await?);}
    if action=="history" {return Ok(state.technician_history(&token).await?);}
    #[cfg(all(windows,feature="flutter"))]
    if action=="update" {
        // Include disconnected/reconnecting windows, not just authenticated
        // sessions. The file lock and pending receipt guard other processes.
        if !crate::flutter::sessions::get_sessions().is_empty(){return Ok(json!({"handed_off":false}));}
        {let mut memory=TECHNICIAN.lock().unwrap();
            if memory.generation!=generation || memory.login.is_none(){bail!("Login changed before update");}
            if memory.update_running || memory.update_handed_off{return Ok(json!({"handed_off":memory.update_handed_off}));}
            memory.update_running=true;memory.update_failed=false;
        }
        let mut state=state;
        hbb_common::tokio::spawn(async move {
            // Authentication remains in native memory. Final server approval,
            // receipt creation and session exclusion remain inside the updater.
            let result=state.update(&directory,Some(&token)).await;
            let mut memory=TECHNICIAN.lock().unwrap();
            memory.update_running=false;memory.update_failed=result.is_err();
            memory.update_handed_off=matches!(result,Ok(true));
            if memory.update_handed_off {
                memory.generation=memory.generation.wrapping_add(1);memory.login=None;memory.tickets.clear();memory.reconnects.clear();
            }
        });
        return Ok(json!({"handed_off":false,"started":true}));
    }
    if action=="update-status" {
        let envelope=state.approved_update(Some(&token)).await?;
        let memory=TECHNICIAN.lock().unwrap();
        if memory.generation!=generation || memory.login.is_none(){bail!("Login changed during update discovery");}
        let Some(envelope)=envelope else{return Ok(serde_json::Value::Null);};
        let release:swan_agent::protocol::Release=envelope.verify(&protocol::public_key(&state.bootstrap.release_public_key)?)?;
        return Ok(json!({"version":release.version,"sequence":release.sequence,"channel":release.channel}));
    }
    if action=="connect" {
        let device=input["device_id"].as_str().ok_or_else(||anyhow!("Missing device"))?;
        let key=protocol::signing_key_from_hex(&protocol::random_token())?;
        let (envelope,grant)=state.technician_ticket(&token,device,input["unattended"].as_bool().unwrap_or(false),&key).await?;
        let mut memory=TECHNICIAN.lock().unwrap();
        if memory.generation!=generation || memory.login.is_none(){bail!("Login changed during authorization");}
        memory.tickets.retain(|_,ticket|ticket.expires_at>protocol::now());
        memory.reconnects.retain(|_,request|request.expires_at>protocol::now());
        if memory.tickets.len()>=32 {bail!("Too many pending connections");}
        if memory.reconnects.len()>=1024 {bail!("Too many connection requests; sign in again");}
        // The proof is consumed inside native handle_hash, never sent to Dart,
        // command-line arguments, another app instance or IPC configuration.
        let handle=format!("SWT1.{}",protocol::random_token());
        memory.reconnects.insert(handle.clone(),ReconnectRequest{device:device.into(),unattended:grant.unattended,company:grant.company_id.clone(),peer:grant.rustdesk_id.clone(),generation,expires_at:protocol::now()+86400});
        memory.tickets.insert(handle.clone(),PendingTicket{envelope,key_hex:hex::encode(key.to_bytes()),expires_at:grant.expires_at,company:grant.company_id,peer:grant.rustdesk_id.clone()});
        return Ok(json!({"rustdesk_id":grant.rustdesk_id,"ticket_handle":handle}));
    }
    bail!("Unknown technician action")
}

pub fn apply_defaults() {
    let state = AgentState::load_for_refresh(&swan_agent::state_directory());
    let profile = state.as_ref().ok().and_then(|s| s.company_profile().ok());
    let technician = state.as_ref().map(|s| s.bootstrap.edition == Edition::Technician).unwrap_or(false);
    // Stable edition identifiers keep customer service/configuration separate
    // from a technician app running on the same Windows machine.
    *config::APP_NAME.write().unwrap() = if technician{"Swan Remote Support Technician"}else{"Swan Remote Support"}.into();
    let mut overrides = config::OVERWRITE_SETTINGS.write().unwrap();
    // Fail closed before setup; never use an upstream public server or legacy tailnet.
    overrides.insert(keys::OPTION_CUSTOM_RENDEZVOUS_SERVER.into(), profile.as_ref().map(|p|p.rendezvous.clone()).unwrap_or_else(||"unconfigured.invalid".into()));
    overrides.insert(keys::OPTION_RELAY_SERVER.into(), profile.as_ref().map(|p|p.relay.clone()).unwrap_or_else(||"unconfigured.invalid".into()));
    overrides.insert(keys::OPTION_KEY.into(),profile.as_ref().map(|p|p.transport_public_key.clone()).unwrap_or_default());
    overrides.insert(keys::OPTION_API_SERVER.into(),String::new());
    // Company release updates are handled by the configuration agent, not upstream.
    overrides.insert(keys::OPTION_ALLOW_AUTO_UPDATE.into(),"N".into());
    overrides.insert(keys::OPTION_ENABLE_RECORD_SESSION.into(),"N".into());
    overrides.insert(keys::OPTION_APPROVE_MODE.into(),"click".into());
    overrides.insert(keys::OPTION_ALLOW_ONLY_CONN_WINDOW_OPEN.into(),"N".into());
    drop(overrides);
    let mut hard=config::HARD_SETTINGS.write().unwrap();
    hard.insert("conn-type".into(),if technician{"outgoing"}else{"incoming"}.into());
    hard.insert("disable-account".into(),"Y".into());hard.insert("disable-ab".into(),"Y".into());
    drop(hard);
    let mut builtin=config::BUILTIN_SETTINGS.write().unwrap();
    for key in [keys::OPTION_HIDE_NETWORK_SETTINGS,keys::OPTION_HIDE_SERVER_SETTINGS,keys::OPTION_HIDE_PROXY_SETTINGS,keys::OPTION_DISABLE_CHANGE_ID] {builtin.insert(key.into(),"Y".into());}
    // Consent and signed grants, rather than Windows-lock-screen password fallback, authorize sessions.
    builtin.insert(keys::OPTION_ALLOW_LOGON_SCREEN_PASSWORD.into(),"N".into());
}

pub fn refresh_defaults()->bool {
    static MODIFIED:std::sync::Mutex<Option<std::time::SystemTime>>=std::sync::Mutex::new(None);
    let modified=match std::fs::metadata(swan_agent::state_directory().join("managed-state.json")).and_then(|m|m.modified()) {
        Ok(modified)=>modified,
        Err(_)=>return false,
    };
    let mut last=MODIFIED.lock().unwrap();if *last==Some(modified){return false;}*last=Some(modified);drop(last);
    let before=(config::Config::get_option(keys::OPTION_CUSTOM_RENDEZVOUS_SERVER),config::Config::get_option(keys::OPTION_RELAY_SERVER),config::Config::get_option(keys::OPTION_KEY));
    apply_defaults();
    before!=(config::Config::get_option(keys::OPTION_CUSTOM_RENDEZVOUS_SERVER),config::Config::get_option(keys::OPTION_RELAY_SERVER),config::Config::get_option(keys::OPTION_KEY))
}

pub async fn claim(password:&[u8],challenge:&str,peer:&str)->ResultType<Lease> {
    let login=ManagedLogin::from_wire(password)?;
    let state=AgentState::load(&swan_agent::state_directory())?;
    Ok(state.claim(&login,challenge,peer).await?)
}

pub async fn renew(lease:&mut Lease)->ResultType<()> {
    let state=AgentState::load(&swan_agent::state_directory())?;
    state.renew(lease).await?;Ok(())
}

pub async fn login(challenge:&str,target:&str,local_ticket:&str)->ResultType<Vec<u8>> {
    // The outgoing connection loop retains its own shared lock through close.
    // Recheck pending installation here as well before consuming a proof ticket.
    {let _activity=swan_agent::lock_session(&swan_agent::state_directory())?;}
    let state=AgentState::load(&swan_agent::state_directory())?;
    if state.bootstrap.edition!=Edition::Technician {hbb_common::bail!("Technician edition required");}
    // The opaque handle identifies one window's pending request. It contains
    // no grant or proof key and cannot overwrite another request to this peer.
    let pending=if local_ticket.is_empty(){None}else{
        if !local_ticket.starts_with("SWT1."){hbb_common::bail!("Managed ticket required; legacy passwords are disabled");}
        TECHNICIAN.lock().unwrap().tickets.remove(local_ticket)
    };
    let (envelope,key)=if let Some(ticket)=pending {
        if ticket.company!=state.bootstrap.company_id || ticket.peer!=target || ticket.expires_at<=protocol::now(){hbb_common::bail!("Expired or wrong-target ticket");}
        (ticket.envelope,protocol::signing_key_from_hex(&ticket.key_hex)?)
    }else if !local_ticket.is_empty(){
        // A consumed proof is never replayed. Reconnection requires current
        // native login and a new target-bound server grant and proof key.
        let (request,token)={let memory=TECHNICIAN.lock().unwrap();
            let request=memory.reconnects.get(local_ticket).ok_or_else(||hbb_common::anyhow::anyhow!("Unknown connection request"))?.clone();
            let login=memory.login.as_ref().ok_or_else(||hbb_common::anyhow::anyhow!("Login required for reconnection"))?;
            if request.company!=state.bootstrap.company_id || login.company!=request.company || request.peer!=target || request.generation!=memory.generation || request.expires_at<=protocol::now(){hbb_common::bail!("Expired or wrong-target reconnection");}
            (request,login.token.clone())
        };
        let key=protocol::signing_key_from_hex(&protocol::random_token())?;
        let (envelope,grant)=state.technician_ticket(&token,&request.device,request.unattended,&key).await?;
        {let memory=TECHNICIAN.lock().unwrap();if memory.generation!=request.generation || memory.login.is_none(){hbb_common::bail!("Login changed during reconnection");}}
        if grant.device_id!=request.device || grant.unattended!=request.unattended{hbb_common::bail!("Wrong reconnection grant");}
        (envelope,key)
    }else{
        (serde_json::from_str(&std::env::var("SWAN_SESSION_GRANT")?)?,protocol::signing_key_from_hex(&std::env::var("SWAN_SESSION_PROOF_KEY")?)?)
    };
    let grant:protocol::SessionGrant=envelope.verify(&protocol::public_key(&state.bootstrap.profile_public_key)?)?;
    grant.validate(&state.bootstrap.company_id,&grant.device_id,target,protocol::now())?;
    let _activity=swan_agent::lock_session(&swan_agent::state_directory())?;
    // A ticket stolen from a connection cannot be used without this ephemeral proof key.
    Ok(ManagedLogin::prove(envelope,&key,challenge).to_wire()?)
}

pub fn overview()->String {
    let state=AgentState::load_for_refresh(&swan_agent::state_directory());
    match state {
        Ok(state)=>match state.display_profile(){
            Ok(profile)=>{
                let brand=if state.bootstrap.edition==Edition::Technician{&profile.technician}else{&profile.customer};
                serde_json::json!({"configured":true,"profile_valid":state.company_profile().is_ok(),"edition":state.bootstrap.edition,"enrolled":state.device_id.is_some(),"unattended":state.unattended_consent,"allow_unattended":profile.allow_unattended,"display_name":brand.display_name,"primary_color":brand.primary_color,"logo_svg":brand.logo_svg,"support_url":brand.support_url,"support_contacts":brand.support_contacts,"shortcuts":brand.shortcuts,"consent_text":brand.consent_text,"domain":profile.management_url}).to_string()
            }
            Err(_)=>"{\"configured\":false}".into()
        },
        Err(_)=>"{\"configured\":false}".into()
    }
}
