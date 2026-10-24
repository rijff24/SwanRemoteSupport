use anyhow::{Context, Result};
use std::{ffi::OsString, future::Future, path::PathBuf, process::Stdio, time::Duration};
use tokio::{process::{Child, Command}, sync::oneshot};

pub(crate) struct Component {
    name: &'static str,
    executable: PathBuf,
    arguments: Vec<OsString>,
    directory: PathBuf,
    environment: Vec<(OsString, OsString)>,
    log: Option<std::fs::File>,
    ready_key: Option<PathBuf>,
}

#[cfg(windows)]
pub(crate) fn installed_specs() -> Result<Vec<Component>> {
    use std::io::Read;
    use sha2::{Digest, Sha256};
    let host = std::env::var("SWAN_PUBLIC_HOST").context("Set company public hostname")?;
    anyhow::ensure!(valid_hostname(&host), "Invalid company public hostname");
    let executable = std::env::current_exe()?;
    let packaged = executable.parent().context("Management install directory")?.join("components");
    let data = PathBuf::from(std::env::var("SWAN_DATA_DIR").context("Set company data directory")?);
    anyhow::ensure!(data.is_absolute(), "Company data directory must be absolute");
    let transport = data.join("transport");
    let tls = data.join("tls");
    let logs = data.join("logs");
    for directory in [&transport, &tls, &logs] {std::fs::create_dir_all(directory)?;}
    let config = data.join("Caddyfile");
    anyhow::ensure!(config.is_file(), "Company HTTPS configuration missing");
    let pins = [
        ("hbbs", "2102e17d32af3ab313a4096d4a4db307984630eccf4f9ed03ef3dfbd4fdb8f83"),
        ("hbbr", "5b5fe62f5b5f1fa7df521a243cbfadbeca67bd9df80d5017f23cf62aca59257e"),
        ("caddy", "586d4a4cd74bdfd2951b6b81766a32904ffa69c4f1c3d521da3870a2120f1d31"),
    ];
    let mut result = Vec::new();
    for (name, expected) in pins {
        let binary = packaged.join(format!("{name}.exe"));
        anyhow::ensure!(std::fs::symlink_metadata(&binary)?.is_file(), "Component must be a regular file");
        let mut input = std::fs::File::open(&binary)?;
        let mut hash = Sha256::new();
        let mut buffer = [0u8; 65536];
        loop {let count=input.read(&mut buffer)?;if count==0 {break;}hash.update(&buffer[..count]);}
        anyhow::ensure!(hex::encode(hash.finalize())==expected, "Packaged {name} hash mismatch");
        let (arguments, directory, environment, ready_key) = match name {
            "hbbs" => (vec!["-r".into(), format!("{host}:21117").into(), "-k".into(), "_".into()], transport.clone(), vec![], Some(transport.join("id_ed25519.pub"))),
            "hbbr" => (vec!["-k".into(), "_".into()], transport.clone(), vec![], None),
            _ => (vec!["run".into(), "--config".into(), config.as_os_str().to_owned(), "--adapter".into(), "caddyfile".into()], data.clone(),
                  vec![("XDG_DATA_HOME".into(), tls.clone().into_os_string()), ("XDG_CONFIG_HOME".into(), data.join("proxy-config").into_os_string())], None),
        };
        let log = std::fs::OpenOptions::new().create(true).append(true).open(logs.join(format!("{name}.log")))?;
        result.push(Component{name, executable:binary, arguments, directory, environment, log:Some(log), ready_key});
    }
    Ok(result)
}

fn valid_hostname(host: &str) -> bool {
    host.len()<=253 && host.contains('.') && host.split('.').all(|label|
        !label.is_empty() && label.len()<=63 && !label.starts_with('-') && !label.ends_with('-') &&
        label.bytes().all(|byte|byte.is_ascii_alphanumeric() || byte==b'-'))
}

#[cfg(windows)]
struct ProcessJob(std::os::windows::io::OwnedHandle);
#[cfg(windows)]
impl ProcessJob {
    fn new() -> Result<Self> {
        use std::os::windows::io::{AsRawHandle, FromRawHandle, OwnedHandle};
        use windows_sys::Win32::System::JobObjects::*;
        // An unnamed, non-inheritable job owns only children launched here.
        let raw=unsafe {CreateJobObjectW(std::ptr::null(), std::ptr::null())};
        anyhow::ensure!(!raw.is_null(), "Create component process job: {}", std::io::Error::last_os_error());
        let handle=unsafe {OwnedHandle::from_raw_handle(raw)};
        let mut limits:JOBOBJECT_EXTENDED_LIMIT_INFORMATION=unsafe {std::mem::zeroed()};
        limits.BasicLimitInformation.LimitFlags=JOB_OBJECT_LIMIT_KILL_ON_JOB_CLOSE;
        let set=unsafe {SetInformationJobObject(handle.as_raw_handle(),JobObjectExtendedLimitInformation,
            &limits as *const _ as *const _,std::mem::size_of_val(&limits) as u32)};
        anyhow::ensure!(set!=0, "Configure component process job: {}", std::io::Error::last_os_error());
        Ok(Self(handle))
    }
    fn attach(&self, child: &Child) -> Result<()> {
        use std::os::windows::io::AsRawHandle;
        let handle=child.raw_handle().context("Component exited before job assignment")?;
        let assigned=unsafe {windows_sys::Win32::System::JobObjects::AssignProcessToJobObject(self.0.as_raw_handle(),handle)};
        anyhow::ensure!(assigned!=0,"Assign component process job: {}",std::io::Error::last_os_error());
        Ok(())
    }
}

async fn stop_children(children: &mut [(&'static str, Child)]) -> Result<()> {
    let mut failure = None;
    for (name, child) in children {
        let outcome = async {
            if child.try_wait()?.is_none() {child.start_kill()?;}
            tokio::time::timeout(Duration::from_secs(10), child.wait()).await??;
            Ok::<_, anyhow::Error>(())
        }.await;
        if let Err(error)=outcome {failure=Some(error.context(format!("Stop {name}")));}
    }
    match failure {Some(error)=>Err(error),None=>Ok(())}
}

pub(crate) async fn supervise<F, Fut>(specs: Vec<Component>, mut stop: oneshot::Receiver<()>, run: F) -> Result<()>
where F: FnOnce(oneshot::Receiver<()>) -> Fut, Fut: Future<Output=Result<()>> {
    #[cfg(windows)]
    let job=ProcessJob::new()?;
    let mut children = Vec::new();
    let startup = async {
        for spec in specs {
            let mut command=Command::new(&spec.executable);
            command.args(&spec.arguments).current_dir(&spec.directory).envs(spec.environment)
                .stdin(Stdio::null()).kill_on_drop(true);
            #[cfg(windows)]
            command.creation_flags(0x08000000); // CREATE_NO_WINDOW
            if let Some(log)=spec.log {command.stderr(log.try_clone()?).stdout(log);}else{command.stderr(Stdio::null()).stdout(Stdio::null());}
            let child=command.spawn().with_context(||format!("Start {}",spec.name))?;
            children.push((spec.name,child));
            #[cfg(windows)]
            job.attach(&children.last().context("Started component missing")?.1)?;
            if let Some(key)=spec.ready_key {
                use base64::Engine;
                let deadline=tokio::time::Instant::now()+Duration::from_secs(30);
                loop {
                    if let Ok(value)=tokio::fs::read_to_string(&key).await {
                        if base64::engine::general_purpose::STANDARD.decode(value.trim()).is_ok_and(|value|value.len()==32) {break;}
                    }
                    if children.last_mut().context("Started component missing")?.1.try_wait()?.is_some() {anyhow::bail!("Component exited before transport key became ready");}
                    anyhow::ensure!(tokio::time::Instant::now()<deadline,"Transport key readiness timed out");
                    tokio::select! {_=&mut stop=>anyhow::bail!("Service stopped during component startup"),_=tokio::time::sleep(Duration::from_millis(100))=>{}}
                }
            }
        }
        Ok::<_,anyhow::Error>(())
    }.await;
    if let Err(error)=startup {stop_children(&mut children).await?;return Err(error);}
    let (shutdown, receiver)=oneshot::channel();
    let server=run(receiver);
    tokio::pin!(server);
    let mut monitor=tokio::time::interval(Duration::from_millis(200));
    let mut server_finished=false;
    let outcome=loop {
        tokio::select! {
            result=&mut server=>{server_finished=true;break result;},
            _=&mut stop=>break Ok(()),
            _=monitor.tick()=>{
                let mut failure=None;
                for (name,child) in &mut children {
                    match child.try_wait() {
                        Ok(Some(status))=>{failure=Some(anyhow::anyhow!("Component {name} exited: {status}"));break;},
                        Err(error)=>{failure=Some(error.into());break;},
                        Ok(None)=>{},
                    }
                }
                if let Some(error)=failure {break Err(error);}
            }
        }
    };
    let _=shutdown.send(());
    let server_stop=if server_finished {Ok(())}else{
        match tokio::time::timeout(Duration::from_secs(10),&mut server).await {
            Ok(result)=>result,Err(error)=>Err(anyhow::Error::from(error).context("Management shutdown timed out")),
        }
    };
    let cleanup=stop_children(&mut children).await;
    outcome.and(server_stop).and(cleanup)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rejects_host_configuration_injection() {
        for host in ["localhost", "example.com\n}", "example.com:443", "example.com/path", "-bad.example", "a..example", "example.com;cmd"] {assert!(!valid_hostname(host));}
        assert!(valid_hostname("support.example.com"));
    }
    #[test]
    #[ignore = "child fixture invoked by supervision tests"]
    fn child_fixture() {
        if std::env::var("SWAN_COMPONENT_FIXTURE").as_deref()==Ok("wait") {std::thread::sleep(Duration::from_secs(60));}
    }
    fn fixture(wait: bool) -> Component {
        Component{name:"fixture", executable:std::env::current_exe().unwrap(),
            arguments:vec!["--exact".into(), "components::tests::child_fixture".into(), "--ignored".into()],
            directory:std::env::current_dir().unwrap(), environment:vec![("SWAN_COMPONENT_FIXTURE".into(),if wait {"wait"}else{"exit"}.into())],log:None,ready_key:None}
    }
    #[tokio::test]
    async fn service_stop_reaps_components() {
        let (stop,receiver)=oneshot::channel();
        let result=supervise(vec![fixture(true)],receiver,|shutdown|async {let _=stop.send(());let _=shutdown.await;Ok(())}).await;
        assert!(result.is_ok(),"{result:?}");
    }
    #[tokio::test]
    async fn component_exit_stops_management() {
        let (_stop,receiver)=oneshot::channel();
        let result=supervise(vec![fixture(false)],receiver,|shutdown|async {shutdown.await?;Ok(())}).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Component fixture exited"));
    }
    #[tokio::test]
    async fn partial_start_failure_reaps_started_components() {
        let (_stop,receiver)=oneshot::channel();
        let mut missing=fixture(false);missing.executable=std::env::temp_dir().join(format!("missing-swan-component-{}",uuid::Uuid::new_v4()));
        let result=supervise(vec![fixture(true),missing],receiver,|_|async {panic!("Management started despite component failure")}).await;
        assert!(result.is_err());
    }
    #[cfg(windows)]
    #[test]
    #[ignore = "process-owner fixture invoked by crash containment test"]
    fn job_owner_fixture() {
        let pid_file=PathBuf::from(std::env::var_os("SWAN_JOB_PID_FILE").unwrap());
        let runtime=tokio::runtime::Runtime::new().unwrap();
        runtime.block_on(async {
            let job=ProcessJob::new().unwrap();
            let child=Command::new(std::env::current_exe().unwrap())
                .args(["--exact","components::tests::child_fixture","--ignored"])
                .env("SWAN_COMPONENT_FIXTURE","wait").stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
            job.attach(&child).unwrap();
            let temporary=pid_file.with_extension("tmp");
            std::fs::write(&temporary,child.id().unwrap().to_string()).unwrap();
            std::fs::rename(temporary,&pid_file).unwrap();
            tokio::time::sleep(Duration::from_secs(60)).await;
            drop(job);
        });
    }
    #[cfg(windows)]
    #[test]
    fn process_owner_crash_terminates_children() {
        use std::os::windows::io::{AsRawHandle,FromRawHandle,OwnedHandle};
        use windows_sys::Win32::System::Threading::{OpenProcess,GetExitCodeProcess,PROCESS_QUERY_LIMITED_INFORMATION};
        let directory=std::env::temp_dir().join(format!("swan-job-test-{}",uuid::Uuid::new_v4()));
        std::fs::create_dir(&directory).unwrap();
        let pid_file=directory.join("pid");
        let mut owner=std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact","components::tests::job_owner_fixture","--ignored"])
            .env("SWAN_JOB_PID_FILE",&pid_file).stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null()).spawn().unwrap();
        let deadline=std::time::Instant::now()+Duration::from_secs(10);
        while !pid_file.is_file() {
            if std::time::Instant::now()>=deadline {let _=owner.kill();let _=owner.wait();panic!("Job owner did not report child");}
            std::thread::sleep(Duration::from_millis(20));
        }
        let pid=std::fs::read_to_string(&pid_file).unwrap().parse::<u32>().unwrap();
        let raw=unsafe {OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION,0,pid)};
        let open=!raw.is_null();
        let child=if open {Some(unsafe {OwnedHandle::from_raw_handle(raw)})}else{None};
        owner.kill().unwrap();owner.wait().unwrap();
        assert!(open,"Cannot observe supervised child");
        let child=child.unwrap();
        loop {
            let mut exit=259;
            assert_ne!(unsafe {GetExitCodeProcess(child.as_raw_handle(),&mut exit)},0);
            if exit!=259 {break;}
            assert!(std::time::Instant::now()<deadline,"Child survived owner crash");
            std::thread::sleep(Duration::from_millis(20));
        }
        std::fs::remove_file(pid_file).unwrap();std::fs::remove_dir(directory).unwrap();
    }
}
