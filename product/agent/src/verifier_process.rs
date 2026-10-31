//! Trust verification must fail closed instead of leaving startup waiting forever.
use anyhow::{bail, Context, Result};
use std::process::{Child, ExitStatus};
use std::time::{Duration, Instant};

pub(crate) fn wait(child: &mut Child, timeout: Duration) -> Result<ExitStatus> {
    wait_with_message(child,timeout,"Publisher verification timed out; no installation or application launch was authorized")
}

pub(crate) fn wait_with_message(child: &mut Child, timeout: Duration, message: &'static str) -> Result<ExitStatus> {
    let started = Instant::now();
    let outcome = (|| {
        loop {
            if let Some(status) = child.try_wait().context("Cannot inspect helper process")? {
                return Ok(status);
            }
            if started.elapsed() >= timeout {
                bail!("{message}");
            }
            std::thread::sleep(Duration::from_millis(50).min(timeout.saturating_sub(started.elapsed())));
        }
    })();
    if outcome.is_err() {
        child.kill().context("Cannot stop stalled helper process")?;
        child.wait().context("Cannot reap stopped helper process")?;
    }
    outcome
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    fn child(mode: &str) -> Child {
        Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "verifier_process::tests::verifier_child"])
            .env("SWAN_VERIFIER_WAIT_TEST", mode)
            .stdin(Stdio::null()).stdout(Stdio::null()).stderr(Stdio::null())
            .spawn().unwrap()
    }

    #[test]
    fn verifier_child() {
        match std::env::var("SWAN_VERIFIER_WAIT_TEST").as_deref() {
            Ok("stall") => std::thread::sleep(Duration::from_secs(30)),
            Ok("reject") => std::process::exit(7),
            _ => {}
        }
    }

    #[test]
    fn completed_and_rejected_verifiers_keep_their_real_exit_status() {
        for (mode, expected) in [("accept", 0), ("reject", 7)] {
            let mut process = child(mode);
            assert_eq!(wait(&mut process, Duration::from_secs(20)).unwrap().code(), Some(expected));
        }
    }

    #[test]
    fn stalled_verification_is_an_error_and_its_process_is_reaped() {
        let mut process = child("stall");
        let error = wait(&mut process, Duration::from_millis(100)).unwrap_err();
        assert!(error.to_string().contains("timed out"));
        assert!(process.try_wait().unwrap().is_some());
    }

    #[test]
    fn interrupted_ui_helper_preserves_its_recovery_diagnostic_and_is_reaped() {
        let mut process=child("stall");
        let error=wait_with_message(&mut process,Duration::from_millis(100),
            "Customer UI recovery helper timed out; durable recovery remains pending").unwrap_err();
        assert!(error.to_string().contains("durable recovery remains pending"));
        assert!(!error.to_string().contains("no installation or application launch was authorized"));
        assert!(process.try_wait().unwrap().is_some());
    }
}
