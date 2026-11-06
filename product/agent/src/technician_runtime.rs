//! Process-lifetime exclusion for explicit technician removal, not session authority.
use anyhow::{ensure, Context, Result};
use std::{fs::File, path::Path};
#[cfg(any(windows, test))]
use std::time::{Duration, Instant};

fn open(directory: &Path) -> Result<File> {
    let mut options = std::fs::OpenOptions::new();
    options.create(true).read(true).write(true);
    #[cfg(windows)] {
        use std::os::windows::fs::OpenOptionsExt;
        // A remover must observe the same file until every native process exits.
        options.share_mode(0x1 | 0x2); // FILE_SHARE_READ | FILE_SHARE_WRITE
    }
    #[cfg(unix)] {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options.open(directory.join("technician-runtime.lock"))
        .context("Cannot open technician runtime registration")
}

pub fn register(directory: &Path) -> Result<File> {
    let marker = directory.join("pending-uninstall");
    ensure!(!marker.try_exists()?, "Technician removal is in progress");
    let file = open(directory)?;
    fs2::FileExt::try_lock_shared(&file).context("Technician removal is in progress")?;
    ensure!(!marker.try_exists()?, "Technician removal began during startup");
    Ok(file)
}

#[cfg(any(windows, test))]
pub(crate) fn wait_for_exit(directory: &Path, timeout: Duration) -> Result<()> {
    let file = open(directory)?;
    let deadline = Instant::now() + timeout;
    loop {
        ensure!(directory.join("pending-uninstall").try_exists()?, "Technician removal was cancelled");
        match fs2::FileExt::try_lock_exclusive(&file) {
            Ok(()) => return Ok(()),
            Err(error) if error.raw_os_error() == fs2::lock_contended_error().raw_os_error() => {
                ensure!(Instant::now() < deadline, "Close the technician application before removing it; no application files were removed");
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(error) => return Err(error).context("Cannot verify technician shutdown"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn directory() -> std::path::PathBuf {
        let path = std::env::temp_dir().join(format!("swan-technician-runtime-{}", crate::protocol::random_token()));
        std::fs::create_dir(&path).unwrap();
        path
    }

    fn cleanup(path: &Path) {
        assert_eq!(std::fs::canonicalize(path).unwrap().parent(), Some(std::fs::canonicalize(std::env::temp_dir()).unwrap().as_path()));
        std::fs::remove_dir_all(path).unwrap();
    }

    #[test]
    fn removal_waits_for_every_registered_process() {
        let path = directory();
        let first = register(&path).unwrap();
        let second = register(&path).unwrap();
        std::fs::write(path.join("pending-uninstall"), b"").unwrap();
        assert!(wait_for_exit(&path, Duration::from_millis(1)).is_err());
        drop(first);
        assert!(wait_for_exit(&path, Duration::from_millis(1)).is_err());
        drop(second);
        wait_for_exit(&path, Duration::from_millis(1)).unwrap();
        cleanup(&path);
    }

    #[test]
    fn cancellation_rejects_startup_and_missing_cancellation_rejects_removal() {
        let path = directory();
        assert!(wait_for_exit(&path, Duration::ZERO).is_err());
        std::fs::write(path.join("pending-uninstall"), b"").unwrap();
        assert!(register(&path).is_err());
        wait_for_exit(&path, Duration::ZERO).unwrap();
        std::fs::remove_file(path.join("pending-uninstall")).unwrap();
        drop(register(&path).unwrap());
        cleanup(&path);
    }

    #[cfg(windows)]
    #[test]
    fn active_registration_prevents_replacing_the_lock_file() {
        let path = directory();
        let runtime = register(&path).unwrap();
        let lock = path.join("technician-runtime.lock");
        assert!(std::fs::remove_file(&lock).is_err());
        assert!(std::fs::rename(&lock, path.join("replaced.lock")).is_err());
        drop(runtime);
        std::fs::remove_file(&lock).unwrap();
        cleanup(&path);
    }
}
