//! Bounded-memory SWANBK2 envelope. Encryption is RustCrypto STREAM, not
//! independent GCM messages: sequence and the mandatory last block authenticate
//! ordering and completion. The plaintext ZIP only exists in private staging.
use super::*;
use aead::{stream::{DecryptorBE32, EncryptorBE32}, Payload};
use std::{fs::{File, OpenOptions}, io::{BufReader, Seek}, path::PathBuf};

pub(super) const HEADER: &[u8] = b"SWANBK2";
pub(super) const LIMIT: u64 = 64 * 1024 * 1024 * 1024;
const CHUNK: usize = 1024 * 1024;
const TAG: usize = 16;

fn private_file(path: &Path) -> Result<File> {
    let mut options = OpenOptions::new();
    options.create_new(true).read(true).write(true);
    #[cfg(unix)] { use std::os::unix::fs::OpenOptionsExt; options.mode(0o600); }
    Ok(options.open(path)?)
}

pub(super) fn protect_directory(path: &Path) -> Result<()> {
    #[cfg(unix)] {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
    }
    #[cfg(windows)] {
        let identity = std::process::Command::new("whoami.exe").args(["/user", "/fo", "csv", "/nh"]).output()?;
        ensure!(identity.status.success(), "Cannot identify backup restore owner");
        let identity = String::from_utf8_lossy(&identity.stdout);
        let sid = identity.trim().rsplit(',').next().context("Missing restore owner SID")?.trim_matches('"');
        ensure!(sid.starts_with("S-1-") && sid.bytes().all(|byte| byte == b'S' || byte == b'-' || byte.is_ascii_digit()), "Invalid restore owner SID");
        let status = std::process::Command::new("icacls.exe").arg(path)
            .args(["/inheritance:r", "/grant:r"]).arg(format!("*{sid}:(OI)(CI)F"))
            .args(["*S-1-5-18:(OI)(CI)F", "*S-1-5-32-544:(OI)(CI)F"]).output()?;
        ensure!(status.status.success(), "Cannot protect backup staging and restored company keys");
    }
    Ok(())
}

struct Staging(PathBuf);
impl Staging {
    fn new(parent: &Path) -> Result<Self> {
        let path = parent.join(format!(".swan-backup-{}", swan_protocol::random_token()));
        let mut builder = std::fs::DirBuilder::new();
        #[cfg(unix)] { use std::os::unix::fs::DirBuilderExt; builder.mode(0o700); }
        builder.create(&path)?;
        let staging = Self(path);
        protect_directory(&staging.0)?;
        Ok(staging)
    }
    fn cleanup(self) -> Result<()> {
        std::fs::remove_dir_all(&self.0).context("Remove private backup staging")
    }
}
impl Drop for Staging {
    fn drop(&mut self) {
        if let Err(error) = std::fs::remove_dir_all(&self.0) {
            if error.kind() != std::io::ErrorKind::NotFound {
                eprintln!("Cannot remove private backup staging: {error}");
            }
        }
    }
}

// Fill the buffer even when the underlying reader returns short reads.
fn read_block(reader: &mut impl Read, buffer: &mut [u8]) -> Result<usize> {
    let mut count = 0;
    while count < buffer.len() {
        match reader.read(&mut buffer[count..]) {
            Ok(0) => break,
            Ok(read) => count += read,
            Err(error) if error.kind() == std::io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Ok(count)
}

fn encrypt(input: &mut impl Read, output: &mut impl Write, password: &str) -> Result<()> {
    let mut salt = [0u8; 16];
    let mut nonce = [0u8; 7];
    rand::rngs::OsRng.fill_bytes(&mut salt);
    rand::rngs::OsRng.fill_bytes(&mut nonce);
    let mut header = HEADER.to_vec(); header.extend(salt); header.extend(nonce);
    let cipher = Aes256Gcm::new_from_slice(&key(password, &salt)?).map_err(|_| anyhow::anyhow!("Invalid backup encryption key"))?;
    let mut stream = EncryptorBE32::from_aead(cipher, (&nonce).into());
    output.write_all(&header)?;
    let mut buffer = vec![0; CHUNK];
    let mut total = 0u64;
    loop {
        let count = read_block(input, &mut buffer)?;
        total = total.checked_add(count as u64).context("Backup size overflow")?;
        ensure!(total <= LIMIT + 128 * 1024 * 1024, "Backup archive exceeds size limit");
        let payload = Payload { msg: &buffer[..count], aad: &header };
        if count < CHUNK {
            let encrypted = stream.encrypt_last(payload).map_err(|_| anyhow::anyhow!("Backup encryption failed"))?;
            output.write_all(&encrypted)?;
            return Ok(());
        }
        let encrypted = stream.encrypt_next(payload).map_err(|_| anyhow::anyhow!("Backup encryption failed"))?;
        output.write_all(&encrypted)?;
    }
}

fn decrypt(input: &mut impl Read, output: &mut impl Write, password: &str) -> Result<()> {
    let mut header = [0u8; 30];
    input.read_exact(&mut header)?;
    ensure!(header.starts_with(HEADER), "Invalid backup format");
    let cipher = Aes256Gcm::new_from_slice(&key(password, &header[7..23])?).map_err(|_| anyhow::anyhow!("Invalid decryption key"))?;
    let mut stream = DecryptorBE32::from_aead(cipher, header[23..30].into());
    let mut buffer = vec![0; CHUNK + TAG];
    let mut total = 0u64;
    loop {
        let count = read_block(input, &mut buffer)?;
        ensure!(count >= TAG, "Backup truncated: missing authenticated final block");
        total = total.checked_add((count - TAG) as u64).context("Backup size overflow")?;
        ensure!(total <= LIMIT + 128 * 1024 * 1024, "Backup archive exceeds size limit");
        let payload = Payload { msg: &buffer[..count], aad: &header };
        if count < buffer.len() {
            let plaintext = stream.decrypt_last(payload).map_err(|_| anyhow::anyhow!("Backup password incorrect or backup modified"))?;
            output.write_all(&plaintext)?;
            return Ok(());
        }
        let plaintext = stream.decrypt_next(payload).map_err(|_| anyhow::anyhow!("Backup password incorrect or backup modified"))?;
        output.write_all(&plaintext)?;
    }
}

pub(super) fn export(data: &Path, files: Vec<(String, PathBuf)>, destination: &Path, password: &str) -> Result<()> {
    let staging = Staging::new(data)?;
    let result = (|| -> Result<()> {
        let file = private_file(&staging.0.join("archive.zip"))?;
        let mut archive = zip::ZipWriter::new(file);
        let options = zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let mut total = 0u64;
        for (name, path) in files {
            let metadata = std::fs::symlink_metadata(&path)?;
            ensure!(metadata.is_file() && !metadata.file_type().is_symlink(), "Backup source must be a regular file");
            #[cfg(windows)] { use std::os::windows::fs::MetadataExt; ensure!(metadata.file_attributes() & 0x400 == 0, "Backup source must not be a reparse point"); }
            total = total.checked_add(metadata.len()).context("Backup size overflow")?;
            ensure!(total <= LIMIT, "Backup contents exceed 64 GiB");
            archive.start_file(name, options)?;
            let mut input = File::open(&path)?.take(metadata.len() + 1);
            ensure!(std::io::copy(&mut input, &mut archive)? == metadata.len(), "Backup source changed during export; stop writes before export");
        }
        let mut file = archive.finish()?;
        file.rewind()?;
        // create_new is also the overwrite guard; remove only our own incomplete output.
        let mut output = private_file(destination)?;
        let encrypted = encrypt(&mut file, &mut output, password).and_then(|_| Ok(output.sync_all()?));
        drop(output);
        if encrypted.is_err() { std::fs::remove_file(destination).context("Remove incomplete encrypted backup")?; }
        encrypted
    })();
    staging.cleanup()?;
    result
}

pub(super) fn restore(data: &Path, source: &Path, password: &str) -> Result<()> {
    ensure!(std::fs::metadata(source)?.len() <= LIMIT + 132 * 1024 * 1024, "Backup exceeds size limit");
    let parent = data.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let staging = Staging::new(parent)?;
    let result = (|| -> Result<()> {
        let mut file = private_file(&staging.0.join("archive.zip"))?;
        decrypt(&mut BufReader::new(File::open(source)?), &mut file, password)?;
        file.rewind()?;
        let mut archive = zip::ZipArchive::new(file)?;
        super::restore_archive(data, &mut archive, LIMIT)
    })();
    staging.cleanup()?;
    result
}

pub(super) fn extract<R: Read + Seek>(data: &Path, archive: &mut zip::ZipArchive<R>, names: std::collections::HashSet<String>) -> Result<()> {
    let parent = data.parent().filter(|path| !path.as_os_str().is_empty()).unwrap_or(Path::new("."));
    let staging = Staging::new(parent)?;
    let result = (|| -> Result<()> {
        let payload = staging.0.join("restored");
        std::fs::create_dir(&payload)?;
        protect_directory(&payload)?;
        for name in names {
            let mut entry = archive.by_name(&name)?;
            let expected = entry.size();
            let path = payload.join(name);
            if let Some(parent) = path.parent() { std::fs::create_dir_all(parent)?; }
            let mut file = private_file(&path)?;
            ensure!(std::io::copy(&mut entry.by_ref().take(expected + 1), &mut file)? == expected, "Backup entry size mismatch");
            file.sync_all()?;
        }
        // Only a completely authenticated, validated and successfully extracted
        // backup is published. Preserve an existing target on every earlier failure.
        super::empty_target(data)?;
        if data.exists() { std::fs::remove_dir(data)?; }
        std::fs::rename(&payload, data).context("Publish verified backup restore")?;
        Ok(())
    })();
    staging.cleanup()?;
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use sha2::{Digest, Sha256};

    fn fixture() -> (PathBuf, PathBuf) {
        let root = std::env::temp_dir().join(format!("swan-stream-test-{}", swan_protocol::random_token()));
        let data = root.join("data"); std::fs::create_dir_all(&data).unwrap();
        let db = rusqlite::Connection::open(data.join("management.sqlite3")).unwrap();
        db.execute_batch("CREATE TABLE identity(value TEXT); INSERT INTO identity VALUES('retained identity');").unwrap();
        drop(db);
        std::fs::write(data.join("profile-key.hex"), b"fixture signing key").unwrap();
        std::fs::write(data.join("setup-token.txt"), b"fixture setup token").unwrap();
        (root, data)
    }

    #[test]
    fn stream_authenticates_order_finality_header_and_chunk_boundaries() {
        let password = "stream encryption test password";
        for size in [0, 1, CHUNK - 1, CHUNK, CHUNK + 1, 2 * CHUNK + 333] {
            let mut plaintext = vec![0; size]; rand::rngs::OsRng.fill_bytes(&mut plaintext);
            let mut encrypted = Vec::new(); encrypt(&mut plaintext.as_slice(), &mut encrypted, password).unwrap();
            let mut restored = Vec::new(); decrypt(&mut encrypted.as_slice(), &mut restored, password).unwrap();
            assert_eq!(restored, plaintext);
            assert!(decrypt(&mut encrypted.as_slice(), &mut Vec::new(), "different test password").is_err());
            let mut truncated = encrypted.clone(); truncated.pop();
            assert!(decrypt(&mut truncated.as_slice(), &mut Vec::new(), password).is_err());
            let mut modified = encrypted.clone(); let last = modified.len() - 1; modified[last] ^= 1;
            assert!(decrypt(&mut modified.as_slice(), &mut Vec::new(), password).is_err());
            let mut changed_header = encrypted.clone(); changed_header[23] ^= 1;
            assert!(decrypt(&mut changed_header.as_slice(), &mut Vec::new(), password).is_err());
            let mut appended = encrypted.clone(); appended.extend_from_slice(&[0; 16]);
            assert!(decrypt(&mut appended.as_slice(), &mut Vec::new(), password).is_err());
            if size >= CHUNK {
                let boundary = 30 + CHUNK + TAG;
                let mut truncated = &encrypted[..boundary];
                assert!(decrypt(&mut truncated, &mut Vec::new(), password).is_err(), "A complete next block must not substitute for finality");
            }
            if size >= 2 * CHUNK {
                let block = CHUNK + TAG;
                let mut reordered = encrypted.clone();
                for index in 0..block { reordered.swap(30 + index, 30 + block + index); }
                assert!(decrypt(&mut reordered.as_slice(), &mut Vec::new(), password).is_err());
                let mut deleted = encrypted.clone(); deleted.drain(30..30 + block);
                assert!(decrypt(&mut deleted.as_slice(), &mut Vec::new(), password).is_err());
            }
        }
        struct ShortRead<'a>(&'a [u8]);
        impl Read for ShortRead<'_> {
            fn read(&mut self, output: &mut [u8]) -> std::io::Result<usize> {
                let count = output.len().min(511).min(self.0.len());
                output[..count].copy_from_slice(&self.0[..count]); self.0 = &self.0[count..]; Ok(count)
            }
        }
        let bytes = vec![42; CHUNK + 31]; let mut encrypted = Vec::new();
        encrypt(&mut ShortRead(&bytes), &mut encrypted, password).unwrap();
        let mut restored = Vec::new(); decrypt(&mut ShortRead(&encrypted), &mut restored, password).unwrap();
        assert_eq!(bytes, restored);
    }

    #[test]
    fn legacy_backup_restores_and_new_export_never_overwrites() {
        let (root, data) = fixture();
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for name in ["management.sqlite3", "profile-key.hex", "setup-token.txt"] {
            archive.start_file(name, zip::write::SimpleFileOptions::default()).unwrap();
            archive.write_all(&std::fs::read(data.join(name)).unwrap()).unwrap();
        }
        let plaintext = archive.finish().unwrap().into_inner();
        let salt = [1u8; 16]; let nonce = [2u8; 12]; let password = "legacy compatibility test password";
        let cipher = Aes256Gcm::new_from_slice(&key(password, &salt).unwrap()).unwrap();
        let mut bytes = super::super::HEADER.to_vec(); bytes.extend(salt); bytes.extend(nonce);
        bytes.extend(cipher.encrypt(Nonce::from_slice(&nonce), plaintext.as_ref()).unwrap());
        let source = root.join("legacy.backup"); std::fs::write(&source, &bytes).unwrap();
        super::super::restore(&root.join("restored"), &source, password).unwrap();
        assert_eq!(std::fs::read(root.join("restored/profile-key.hex")).unwrap(), b"fixture signing key");
        assert!(super::super::export(&data, &source, password).is_err());
        assert_eq!(std::fs::read(&source).unwrap(), bytes);
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn authenticated_but_invalid_zip_never_publishes_or_leaves_plaintext_staging() {
        let (root, _) = fixture(); let password = "invalid zip test password";
        for (index, invalid) in ["tls-storage/../outside", "unknown-secret"].into_iter().enumerate() {
            let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
            for name in ["management.sqlite3", "profile-key.hex", "setup-token.txt", invalid] {
                archive.start_file(name, zip::write::SimpleFileOptions::default()).unwrap(); archive.write_all(b"fixture").unwrap();
            }
            let bytes = archive.finish().unwrap().into_inner(); let mut encrypted = Vec::new();
            encrypt(&mut bytes.as_slice(), &mut encrypted, password).unwrap();
            let source = root.join(format!("bad-{index}.backup")); std::fs::write(&source, encrypted).unwrap();
            let destination = root.join(format!("restored-{index}"));
            std::fs::create_dir(&destination).unwrap();
            assert!(super::super::restore(&destination, &source, password).is_err());
            assert!(std::fs::read_dir(&destination).unwrap().next().is_none());
            assert!(std::fs::read_dir(&root).unwrap().all(|entry| !entry.unwrap().file_name().to_string_lossy().starts_with(".swan-backup-")));
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn crc_failure_during_extraction_preserves_target_and_failed_export_cleans_up() {
        let (root, data) = fixture(); let password = "crc rejection test password";
        let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
        for name in ["management.sqlite3", "profile-key.hex", "setup-token.txt"] {
            archive.start_file(name, zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored)).unwrap();
            archive.write_all(b"unique CRC payload fixture").unwrap();
        }
        let mut bytes = archive.finish().unwrap().into_inner();
        let offset = bytes.windows(b"unique CRC payload fixture".len()).position(|part| part == b"unique CRC payload fixture").unwrap();
        bytes[offset] ^= 1;
        let mut encrypted = Vec::new(); encrypt(&mut bytes.as_slice(), &mut encrypted, password).unwrap();
        let source = root.join("crc.backup"); std::fs::write(&source, encrypted).unwrap();
        let target = root.join("target"); std::fs::create_dir(&target).unwrap();
        assert!(super::super::restore(&target, &source, password).is_err());
        assert!(std::fs::read_dir(&target).unwrap().next().is_none());
        let failed = root.join("failed.backup");
        assert!(super::super::export(&data, &failed, "short").is_err());
        assert!(!failed.exists());
        for directory in [&root, &data] {
            assert!(std::fs::read_dir(directory).unwrap().all(|entry| {
                let name = entry.unwrap().file_name(); let name = name.to_string_lossy();
                !name.starts_with(".swan-backup-") && !name.starts_with("backup-")
            }));
        }
        std::fs::remove_dir_all(root).unwrap();
    }

    // Required explicitly by CI and suitable for a memory-limited local container.
    // Separate from the default suite because it writes/restores >1 GiB on disk.
    #[test]
    #[ignore = "large backup qualification; run explicitly"]
    fn backup_above_previous_limit_preserves_incompressible_artifact() {
        let (root, data) = fixture(); let artifacts = data.join("artifacts"); std::fs::create_dir(&artifacts).unwrap();
        let name = format!("{}.zip", uuid::Uuid::new_v4()); let mut file = File::create(artifacts.join(&name)).unwrap();
        let mut buffer = vec![0; CHUNK]; let mut expected = Sha256::new();
        for _ in 0..513 { rand::rngs::OsRng.fill_bytes(&mut buffer); file.write_all(&buffer).unwrap(); expected.update(&buffer); }
        file.sync_all().unwrap(); drop(file);
        let source = root.join("large.backup"); let password = "large backup qualification password";
        super::super::export_deployment_with_artifacts(&data, &source, password, None, None, None, None, Some(&artifacts)).unwrap();
        assert!(std::fs::metadata(&source).unwrap().len() > 512 * 1024 * 1024);
        let target = root.join("restored"); super::super::restore(&target, &source, password).unwrap();
        let mut file = File::open(target.join("artifacts").join(&name)).unwrap();
        assert_eq!(file.metadata().unwrap().len(), 513 * 1024 * 1024);
        let mut actual = Sha256::new(); loop { let count = file.read(&mut buffer).unwrap(); if count == 0 { break; } actual.update(&buffer[..count]); }
        assert_eq!(expected.finalize(), actual.finalize()); drop(file);
        assert_eq!(std::fs::read(target.join("profile-key.hex")).unwrap(), b"fixture signing key");
        std::fs::remove_dir_all(root).unwrap();
    }
}
