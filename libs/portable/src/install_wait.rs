use std::{io,process::Child};

pub fn installation_exit_code(child:io::Result<Child>)->i32 {
    match child {
        Ok(mut child)=>match child.wait(){Ok(status)=>status.code().unwrap_or(1),Err(error)=>{eprintln!("Installer wait failed: {error}");1}},
        Err(error)=>{eprintln!("Installer launch failed: {error}");1},
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn child_fixture(){
        if let Ok(code)=std::env::var("SWAN_TEST_INSTALL_CHILD") {
            std::thread::sleep(std::time::Duration::from_millis(100));
            std::process::exit(code.parse().unwrap());
        }
    }
    #[test]
    fn waits_for_child_and_preserves_failure_status(){
        let started=std::time::Instant::now();
        let child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","tests::child_fixture"]).env("SWAN_TEST_INSTALL_CHILD","7").spawn();
        assert_eq!(installation_exit_code(child),7);
        assert!(started.elapsed()>=std::time::Duration::from_millis(100));
    }
    #[test]
    fn completed_child_succeeds_and_missing_executable_fails(){
        let child=std::process::Command::new(std::env::current_exe().unwrap()).args(["--exact","tests::child_fixture"]).env("SWAN_TEST_INSTALL_CHILD","0").spawn();
        assert_eq!(installation_exit_code(child),0);
        let missing=std::env::temp_dir().join(format!("swan-missing-installer-{}.exe",std::process::id()));
        assert!(!missing.exists());
        assert_eq!(installation_exit_code(std::process::Command::new(missing).spawn()),1);
    }
}
