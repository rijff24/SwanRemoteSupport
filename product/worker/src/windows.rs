use anyhow::{ensure,Context,Result};
use std::{ffi::OsString,sync::Mutex,time::Duration};
use windows_service::{define_windows_service,service::{ServiceControl,ServiceControlAccept,ServiceExitCode,ServiceState,ServiceStatus,ServiceType},service_control_handler::{self,ServiceControlHandlerResult},service_dispatcher};
const NAME:&str="SwanInstallerWorker";
define_windows_service!(ffi_service_main,service_main);
pub fn dispatch()->Result<()> {service_dispatcher::start(NAME,ffi_service_main)?;Ok(())}
fn service_log()->Result<std::fs::File> {
    use std::os::windows::{fs::MetadataExt,io::AsRawHandle};
    use windows_sys::Win32::System::Console::{SetStdHandle,STD_ERROR_HANDLE,STD_OUTPUT_HANDLE};
    let parent=std::path::PathBuf::from(std::env::var_os("ProgramData").context("Missing ProgramData")?);
    let directory=parent.join("SwanInstallerWorker");
    for path in [&parent,&directory] {
        let metadata=std::fs::symlink_metadata(path)?;
        ensure!(metadata.is_dir() && metadata.file_attributes()&0x400==0,"Worker log directory must be a regular protected directory");
    }
    let path=directory.join("worker-service.log");
    match std::fs::symlink_metadata(&path) {
        Ok(metadata)=>ensure!(metadata.is_file() && metadata.file_attributes()&0x400==0,"Worker log must be a regular file"),
        Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
        Err(error)=>return Err(error.into()),
    }
    let file=std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    // Keep the file alive throughout the service; the installer protects its
    // parent directory before SCM can launch this process.
    for handle in [STD_ERROR_HANDLE,STD_OUTPUT_HANDLE] {
        if unsafe {SetStdHandle(handle,file.as_raw_handle())}==0 {return Err(std::io::Error::last_os_error().into());}
    }
    Ok(file)
}
fn service_main(_arguments:Vec<OsString>) {
    let (sender,receiver)=tokio::sync::oneshot::channel();let sender=Mutex::new(Some(sender));
    let handler=move|control|match control {
        ServiceControl::Stop|ServiceControl::Shutdown=>{if let Some(sender)=sender.lock().unwrap().take(){let _=sender.send(());}ServiceControlHandlerResult::NoError},
        ServiceControl::Interrogate=>ServiceControlHandlerResult::NoError,
        _=>ServiceControlHandlerResult::NotImplemented,
    };
    let status_handle=match service_control_handler::register(NAME,handler){Ok(handle)=>handle,Err(error)=>{eprintln!("Service registration failed: {error}");return;}};
    let status=|state,exit|ServiceStatus{service_type:ServiceType::OWN_PROCESS,current_state:state,controls_accepted:if state==ServiceState::Running{ServiceControlAccept::STOP|ServiceControlAccept::SHUTDOWN}else{ServiceControlAccept::empty()},exit_code:ServiceExitCode::Win32(exit),checkpoint:0,wait_hint:Duration::from_secs(10),process_id:None};
    let _log=match service_log(){Ok(file)=>file,Err(error)=>{
        eprintln!("Worker log initialization failed: {error}");
        if let Err(error)=status_handle.set_service_status(status(ServiceState::Stopped,1)){eprintln!("Service failure status could not be reported: {error}");}
        return;
    }};
    if let Err(error)=status_handle.set_service_status(status(ServiceState::Running,0)){eprintln!("Service status failed: {error}");return;}
    // SCM invokes this entrypoint on its own thread; it owns the only worker runtime.
    let outcome=tokio::runtime::Runtime::new().map_err(anyhow::Error::from).and_then(|runtime|runtime.block_on(async {
        super::run(receiver).await
    }));
    if let Err(error)=&outcome{eprintln!("Installer worker failed: {error}");}
    if let Err(error)=status_handle.set_service_status(status(ServiceState::Stopped,if outcome.is_ok(){0}else{1})){eprintln!("Service shutdown status failed: {error}");}
}
