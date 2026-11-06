use anyhow::Result;
use std::{ffi::OsString,sync::Mutex,time::Duration};
use windows_service::{define_windows_service,service::{ServiceControl,ServiceControlAccept,ServiceExitCode,ServiceState,ServiceStatus,ServiceType},service_control_handler::{self,ServiceControlHandlerResult},service_dispatcher};
const NAME:&str="SwanCompanyServer";
define_windows_service!(ffi_service_main,service_main);
pub fn dispatch()->Result<()> {service_dispatcher::start(NAME,ffi_service_main)?;Ok(())}
fn service_main(_arguments:Vec<OsString>) {
    let (sender,receiver)=tokio::sync::oneshot::channel();let sender=Mutex::new(Some(sender));
    let handler=move|control|match control {
        ServiceControl::Stop|ServiceControl::Shutdown=>{if let Some(sender)=sender.lock().unwrap().take(){let _=sender.send(());}ServiceControlHandlerResult::NoError},
        ServiceControl::Interrogate=>ServiceControlHandlerResult::NoError,
        _=>ServiceControlHandlerResult::NotImplemented,
    };
    let status_handle=match service_control_handler::register(NAME,handler){Ok(handle)=>handle,Err(error)=>{eprintln!("Service registration failed: {error}");return;}};
    let status=|state,exit|ServiceStatus{service_type:ServiceType::OWN_PROCESS,current_state:state,controls_accepted:if state==ServiceState::Running{ServiceControlAccept::STOP|ServiceControlAccept::SHUTDOWN}else{ServiceControlAccept::empty()},exit_code:ServiceExitCode::Win32(exit),checkpoint:0,wait_hint:Duration::from_secs(10),process_id:None};
    if let Err(error)=status_handle.set_service_status(status(ServiceState::Running,0)){eprintln!("Service status failed: {error}");return;}
    // SCM invokes this entrypoint on its own thread; it owns the only server runtime.
    let outcome=tokio::runtime::Runtime::new().map_err(anyhow::Error::from).and_then(|runtime|runtime.block_on(async {
        match std::env::var("SWAN_COMPONENTS") {
            Ok(value) if value=="1"=>{
                let specs=tokio::task::spawn_blocking(super::components::installed_specs).await??;
                super::components::supervise(specs,receiver,super::run).await
            },
            Err(std::env::VarError::NotPresent)=>super::run(receiver).await,
            _=>anyhow::bail!("Invalid company component service mode"),
        }
    }));
    if let Err(error)=&outcome{eprintln!("Company server failed: {error}");}
    if let Err(error)=status_handle.set_service_status(status(ServiceState::Stopped,if outcome.is_ok(){0}else{1})){eprintln!("Service shutdown status failed: {error}");}
}
