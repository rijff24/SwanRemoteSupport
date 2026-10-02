use aes_gcm::{aead::{Aead,KeyInit},Aes256Gcm,Nonce};
use anyhow::{ensure,Context,Result};
use rand::RngCore;
use std::{io::{Cursor,Read,Write},path::Path};

const HEADER:&[u8]=b"SWANBK1";
fn key(password:&str,salt:&[u8])->Result<[u8;32]> {
    ensure!(password.len()>=16 && password.len()<=1024,"Backup passphrase must contain 16–1024 characters");
    let mut key=[0u8;32];argon2::Argon2::default().hash_password_into(password.as_bytes(),salt,&mut key).map_err(|_|anyhow::anyhow!("Backup key derivation failed"))?;Ok(key)
}
pub fn export(data:&Path,destination:&Path,password:&str)->Result<()> {
    export_complete(data,destination,password,None,None,None)
}
pub fn export_complete(data:&Path,destination:&Path,password:&str,transport:Option<&Path>,configuration:Option<&Path>,tls_identity:Option<&Path>)->Result<()> {
    ensure!(!destination.exists(),"Refusing to overwrite an existing backup");
    let db=rusqlite::Connection::open_with_flags(data.join("management.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let snapshot=data.join(format!("backup-{}.sqlite3",swan_protocol::random_token()));
    db.backup(rusqlite::DatabaseName::Main,&snapshot,None)?;
    let mut snapshots=vec![snapshot.clone()];
    let result=(||->Result<()> {
        let mut archive=zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options=zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        let mut files=vec![("management.sqlite3",snapshot.clone()),("profile-key.hex",data.join("profile-key.hex")),("setup-token.txt",data.join("setup-token.txt"))];
        if let Some(directory)=transport {
            files.push(("transport-id_ed25519",directory.join("id_ed25519")));files.push(("transport-id_ed25519.pub",directory.join("id_ed25519.pub")));
            if directory.join("db_v2.sqlite3").exists(){
                let transport_snapshot=data.join(format!("backup-transport-{}.sqlite3",swan_protocol::random_token()));snapshots.push(transport_snapshot.clone());
                let transport_db=rusqlite::Connection::open_with_flags(directory.join("db_v2.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
                transport_db.backup(rusqlite::DatabaseName::Main,&transport_snapshot,None)?;
                files.push(("transport-db.sqlite3",transport_snapshot));
            }
        }
        if let Some(path)=configuration {files.push(("deployment.env",path.to_path_buf()));}
        if let Some(path)=tls_identity {files.push(("tls-identity",path.to_path_buf()));}
        let mut total=0u64;
        for (name,path) in files {
            let size=std::fs::metadata(&path)?.len();total=total.checked_add(size).context("Backup size overflow")?;
            ensure!(total<=512*1024*1024,"Backup contents exceed 512 MiB");
            archive.start_file(name,options)?;archive.write_all(&std::fs::read(path)?)?;
        }
        let bytes=archive.finish()?.into_inner();
        let mut salt=[0u8;16];let mut nonce=[0u8;12];rand::rngs::OsRng.fill_bytes(&mut salt);rand::rngs::OsRng.fill_bytes(&mut nonce);
        let cipher=Aes256Gcm::new_from_slice(&key(password,&salt)?).map_err(|_|anyhow::anyhow!("Invalid backup encryption key"))?;
        let encrypted=cipher.encrypt(Nonce::from_slice(&nonce),bytes.as_ref()).map_err(|_|anyhow::anyhow!("Backup encryption failed"))?;
        let mut options=std::fs::OpenOptions::new();options.create_new(true).write(true);
        #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
        let mut file=options.open(destination)?;file.write_all(HEADER)?;file.write_all(&salt)?;file.write_all(&nonce)?;file.write_all(&encrypted)?;file.sync_all()?;Ok(())
    })();
    for snapshot in snapshots {if snapshot.exists(){std::fs::remove_file(&snapshot).context("Remove plaintext database snapshot after backup")?;}}
    result
}
pub fn restore(data:&Path,source:&Path,password:&str)->Result<()> {
    ensure!(!data.exists() || std::fs::read_dir(data)?.next().is_none(),"Restore requires an empty target directory and a stopped management service");
    ensure!(std::fs::metadata(source)?.len()<=1024*1024*1024,"Backup exceeds size limit");
    let bytes=std::fs::read(source)?;
    ensure!(bytes.len()>HEADER.len()+28 && bytes.starts_with(HEADER),"Invalid backup format");
    let offset=HEADER.len();let salt=&bytes[offset..offset+16];let nonce=&bytes[offset+16..offset+28];
    let cipher=Aes256Gcm::new_from_slice(&key(password,salt)?).map_err(|_|anyhow::anyhow!("Invalid decryption key"))?;
    let plaintext=cipher.decrypt(Nonce::from_slice(nonce),&bytes[offset+28..]).map_err(|_|anyhow::anyhow!("Backup password incorrect or backup modified"))?;
    let mut archive=zip::ZipArchive::new(Cursor::new(plaintext))?;
    ensure!((3..=8).contains(&archive.len()),"Unexpected backup contents");
    let allowed=["management.sqlite3","profile-key.hex","setup-token.txt","transport-id_ed25519","transport-id_ed25519.pub","transport-db.sqlite3","deployment.env","tls-identity"];
    let mut names=std::collections::HashSet::new();
    for index in 0..archive.len(){let entry=archive.by_index(index)?;ensure!(allowed.contains(&entry.name()) && names.insert(entry.name().to_string()),"Unknown or duplicate backup entry");}
    ensure!(names.contains("transport-id_ed25519")==names.contains("transport-id_ed25519.pub"),"Incomplete transport trust key pair");
    for required in &allowed[..3]{ensure!(names.contains(*required),"Missing management backup entry");}
    // Validate all contents before creating anything. Never extract arbitrary archive paths.
    let mut files=Vec::new();
    let mut total=0u64;
    for name in allowed.into_iter().filter(|name|names.contains(*name)) {
        let mut entry=archive.by_name(name)?;total=total.checked_add(entry.size()).context("Backup size overflow")?;ensure!(total<=512*1024*1024,"Backup contents too large");
        let mut content=Vec::new();entry.read_to_end(&mut content)?;files.push((name,content));
    }
    std::fs::create_dir_all(data)?;
    #[cfg(unix)] {use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(data,std::fs::Permissions::from_mode(0o700))?;}
    #[cfg(windows)] {
        let identity=std::process::Command::new("whoami.exe").args(["/user","/fo","csv","/nh"]).output()?;
        ensure!(identity.status.success(),"Cannot identify backup restore owner");
        let identity=String::from_utf8_lossy(&identity.stdout);
        let sid=identity.trim().rsplit(',').next().context("Missing restore owner SID")?.trim_matches('"');
        ensure!(sid.starts_with("S-1-") && sid.bytes().all(|byte|byte==b'S' || byte==b'-' || byte.is_ascii_digit()),"Invalid restore owner SID");
        let status=std::process::Command::new("icacls.exe").arg(data).args(["/inheritance:r","/grant:r"]).arg(format!("*{sid}:(OI)(CI)F")).args(["*S-1-5-18:(OI)(CI)F","*S-1-5-32-544:(OI)(CI)F"]).output()?;
        ensure!(status.status.success(),"Cannot protect restored company keys");
    }
    for (name,content) in files {
        let mut options=std::fs::OpenOptions::new();options.create_new(true).write(true);
        #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
        let mut file=options.open(data.join(name))?;file.write_all(&content)?;file.sync_all()?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn complete_backup_preserves_transport_configuration_and_tls_and_rejects_tampering() {
        let root=std::env::temp_dir().join(format!("swan-full-backup-{}",swan_protocol::random_token()));
        let data=root.join("data");let transport=root.join("transport");std::fs::create_dir_all(&data).unwrap();std::fs::create_dir(&transport).unwrap();
        let db=rusqlite::Connection::open(data.join("management.sqlite3")).unwrap();db.execute_batch("CREATE TABLE identity(value TEXT);INSERT INTO identity VALUES('company identity');").unwrap();drop(db);
        std::fs::write(data.join("profile-key.hex"),"management signing key").unwrap();std::fs::write(data.join("setup-token.txt"),"setup token").unwrap();
        std::fs::write(transport.join("id_ed25519"),b"private transport key").unwrap();std::fs::write(transport.join("id_ed25519.pub"),b"public transport key").unwrap();
        let db=rusqlite::Connection::open(transport.join("db_v2.sqlite3")).unwrap();db.execute_batch("CREATE TABLE peers(id TEXT);INSERT INTO peers VALUES('retained peer');").unwrap();drop(db);
        let configuration=root.join("company.env");let tls=root.join("server.pfx");std::fs::write(&configuration,b"company settings and TLS password").unwrap();std::fs::write(&tls,b"TLS identity fixture").unwrap();
        let archive=root.join("complete.swan-backup");export_complete(&data,&archive,"complete backup passphrase",Some(&transport),Some(&configuration),Some(&tls)).unwrap();
        let destination=root.join("restored");restore(&destination,&archive,"complete backup passphrase").unwrap();
        assert_eq!(std::fs::read(destination.join("transport-id_ed25519")).unwrap(),b"private transport key");assert_eq!(std::fs::read(destination.join("transport-id_ed25519.pub")).unwrap(),b"public transport key");
        assert_eq!(std::fs::read(destination.join("deployment.env")).unwrap(),std::fs::read(configuration).unwrap());assert_eq!(std::fs::read(destination.join("tls-identity")).unwrap(),std::fs::read(tls).unwrap());
        let db=rusqlite::Connection::open(destination.join("transport-db.sqlite3")).unwrap();assert_eq!(db.query_row("SELECT id FROM peers",[],|row|row.get::<_,String>(0)).unwrap(),"retained peer");drop(db);
        let mut modified=std::fs::read(&archive).unwrap();let last=modified.len()-1;modified[last]^=1;let tampered=root.join("tampered.swan-backup");std::fs::write(&tampered,modified).unwrap();
        let rejected=root.join("rejected");assert!(restore(&rejected,&tampered,"complete backup passphrase").is_err());assert!(!rejected.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn backup_roundtrip_rejects_wrong_password_and_nonempty_restore() {
        let root=std::env::temp_dir().join(format!("swan-backup-{}",swan_protocol::random_token()));
        let data=root.join("data");std::fs::create_dir_all(&data).unwrap();
        let db=rusqlite::Connection::open(data.join("management.sqlite3")).unwrap();db.execute_batch("CREATE TABLE evidence(value TEXT);INSERT INTO evidence VALUES('preserved');").unwrap();drop(db);
        std::fs::write(data.join("profile-key.hex"),"key").unwrap();std::fs::write(data.join("setup-token.txt"),"token").unwrap();
        let archive=root.join("backup.swan");export(&data,&archive,"long backup passphrase").unwrap();
        assert!(restore(&root.join("wrong"),&archive,"different long password").is_err());
        restore(&root.join("restored"),&archive,"long backup passphrase").unwrap();
        let db=rusqlite::Connection::open(root.join("restored/management.sqlite3")).unwrap();
        assert_eq!(db.query_row("SELECT value FROM evidence",[],|r|r.get::<_,String>(0)).unwrap(),"preserved");drop(db);
        assert!(restore(&root.join("restored"),&archive,"long backup passphrase").is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
