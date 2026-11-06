use std::{io, path::{Path, PathBuf}};

pub fn directory(root: &Path, metadata: &[u8]) -> io::Result<PathBuf> {
    let text = std::str::from_utf8(metadata)
        .map_err(|error| io::Error::new(io::ErrorKind::InvalidData, error))?;
    let digest = text.lines().find_map(|line| line.strip_prefix("payload_sha256 = "))
        .filter(|value| value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
        .ok_or_else(|| io::Error::new(io::ErrorKind::InvalidData, "Portable payload identity missing or invalid"))?;
    Ok(root.join(format!("payload-{}", digest.to_ascii_lowercase())))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_payload_does_not_reuse_running_previous_runtime() {
        let root = std::env::temp_dir().join(format!("swan-runtime-cache-{}", std::process::id()));
        let old = directory(&root, format!("payload_sha256 = {}\n", "a".repeat(64)).as_bytes()).unwrap();
        let new = directory(&root, format!("payload_sha256 = {}\n", "b".repeat(64)).as_bytes()).unwrap();
        std::fs::create_dir_all(&old).unwrap();
        std::fs::write(old.join("rustdesk.exe"), b"previous runtime still in use").unwrap();
        let mut options = std::fs::OpenOptions::new();
        options.read(true);
        #[cfg(windows)] {
            use std::os::windows::fs::OpenOptionsExt;
            options.share_mode(0);
        }
        let held = options.open(old.join("rustdesk.exe")).unwrap();
        std::fs::create_dir_all(&new).unwrap();
        std::fs::write(new.join("rustdesk.exe"), b"current runtime").unwrap();
        assert_ne!(old, new);
        assert_eq!(std::fs::read(new.join("rustdesk.exe")).unwrap(), b"current runtime");
        drop(held);
        assert_eq!(std::fs::read(old.join("rustdesk.exe")).unwrap(), b"previous runtime still in use");
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn missing_or_unsafe_identity_fails_closed() {
        for text in ["timestamp = 123\n", "payload_sha256 = ../old", "payload_sha256 = ", "payload_sha256 = xx"] {
            assert!(directory(Path::new("cache"), text.as_bytes()).is_err());
        }
        assert!(directory(Path::new("cache"), &[0xff]).is_err());
    }
}
