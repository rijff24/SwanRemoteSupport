//! Managed edition authorization is enforced on the receiving device, independent
//! of the ID server, relay, direct transport, or customer approval UI.
use hbb_common::{config::{self, keys}, ResultType};
use swan_agent::{protocol::{self, Edition, ManagedLogin, SignedEnvelope}, AgentState, Lease};

pub fn apply_defaults() {
    *config::APP_NAME.write().unwrap() = "Swan Remote Support".into();
    let state = AgentState::load(&swan_agent::state_directory());
    let profile = state.as_ref().ok().and_then(|s| s.company_profile().ok());
    let technician = state.as_ref().map(|s| s.bootstrap.edition == Edition::Technician).unwrap_or(false);
    let mut overrides = config::OVERWRITE_SETTINGS.write().unwrap();
    // Fail closed before setup; never use an upstream public server or legacy tailnet.
    overrides.insert(keys::OPTION_CUSTOM_RENDEZVOUS_SERVER.into(), profile.as_ref().map(|p|p.rendezvous.clone()).unwrap_or_else(||"unconfigured.invalid".into()));
    overrides.insert(keys::OPTION_RELAY_SERVER.into(), profile.as_ref().map(|p|p.relay.clone()).unwrap_or_else(||"unconfigured.invalid".into()));
    overrides.insert(keys::OPTION_KEY.into(),profile.as_ref().map(|p|p.transport_public_key.clone()).unwrap_or_default());
    overrides.insert(keys::OPTION_API_SERVER.into(),String::new());
    // Company release updates are handled by the configuration agent, not upstream.
    overrides.insert(keys::OPTION_ALLOW_AUTO_UPDATE.into(),"N".into());
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

pub async fn claim(password:&[u8],challenge:&str,peer:&str)->ResultType<Lease> {
    let login=ManagedLogin::from_wire(password)?;
    let state=AgentState::load(&swan_agent::state_directory())?;
    Ok(state.claim(&login,challenge,peer).await?)
}

pub async fn renew(lease:&mut Lease)->ResultType<()> {
    let state=AgentState::load(&swan_agent::state_directory())?;
    state.renew(lease).await?;Ok(())
}

pub fn login(challenge:&str,target:&str)->ResultType<Vec<u8>> {
    // Hold the shared activity lock for the outgoing app process. Incoming leases
    // hold their own locks. The updater acquires the exclusive side before install.
    static ACTIVITY:std::sync::Mutex<Option<std::fs::File>>=std::sync::Mutex::new(None);
    let mut activity=ACTIVITY.lock().unwrap();
    if activity.is_none(){*activity=Some(swan_agent::lock_session(&swan_agent::state_directory())?);}
    drop(activity);
    let state=AgentState::load(&swan_agent::state_directory())?;
    if state.bootstrap.edition!=Edition::Technician {hbb_common::bail!("Technician edition required");}
    let envelope:SignedEnvelope=serde_json::from_str(&std::env::var("SWAN_SESSION_GRANT")?)?;
    let grant:protocol::SessionGrant=envelope.verify(&protocol::public_key(&state.bootstrap.profile_public_key)?)?;
    grant.validate(&state.bootstrap.company_id,&grant.device_id,target,protocol::now())?;
    // A ticket stolen from a connection cannot be used without this ephemeral proof key.
    let key=protocol::signing_key_from_hex(&std::env::var("SWAN_SESSION_PROOF_KEY")?)?;
    Ok(ManagedLogin::prove(envelope,&key,challenge).to_wire()?)
}

pub fn overview()->String {
    let state=AgentState::load(&swan_agent::state_directory());
    match state {
        Ok(state)=>match state.company_profile(){
            Ok(profile)=>{
                let brand=if state.bootstrap.edition==Edition::Technician{&profile.technician}else{&profile.customer};
                serde_json::json!({"configured":true,"enrolled":state.device_id.is_some(),"unattended":state.unattended_consent,"display_name":brand.display_name,"primary_color":brand.primary_color,"logo_svg":brand.logo_svg,"support_url":brand.support_url,"consent_text":brand.consent_text,"domain":profile.management_url}).to_string()
            }
            Err(_)=>"{\"configured\":false}".into()
        },
        Err(_)=>"{\"configured\":false}".into()
    }
}
