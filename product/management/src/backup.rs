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
    ensure!(!destination.exists(),"Refusing to overwrite an existing backup");
    let db=rusqlite::Connection::open(data.join("management.sqlite3"))?;
    let snapshot=data.join(format!("backup-{}.sqlite3",swan_protocol::random_token()));
    db.backup(rusqlite::DatabaseName::Main,&snapshot,None)?;
    let result=(||->Result<()> {
        let mut archive=zip::ZipWriter::new(Cursor::new(Vec::new()));
        let options=zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Deflated);
        for (name,path) in [("management.sqlite3",snapshot.as_path()),("profile-key.hex",data.join("profile-key.hex").as_path()),("setup-token.txt",data.join("setup-token.txt").as_path())] {
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
    std::fs::remove_file(&snapshot).context("Remove plaintext database snapshot after backup")?;
    result
}
pub fn restore(data:&Path,source:&Path,password:&str)->Result<()> {
    ensure!(!data.exists() || std::fs::read_dir(data)?.next().is_none(),"Restore requires an empty target directory and a stopped management service");
    let bytes=std::fs::read(source)?;
    ensure!(bytes.len()>HEADER.len()+28 && bytes.starts_with(HEADER),"Invalid backup format");
    let offset=HEADER.len();let salt=&bytes[offset..offset+16];let nonce=&bytes[offset+16..offset+28];
    let cipher=Aes256Gcm::new_from_slice(&key(password,salt)?).map_err(|_|anyhow::anyhow!("Invalid decryption key"))?;
    let plaintext=cipher.decrypt(Nonce::from_slice(nonce),&bytes[offset+28..]).map_err(|_|anyhow::anyhow!("Backup password incorrect or backup modified"))?;
    let mut archive=zip::ZipArchive::new(Cursor::new(plaintext))?;
    ensure!(archive.len()==3,"Unexpected backup contents");
    // Validate all contents before creating anything. Never extract arbitrary archive paths.
    let mut files=Vec::new();
    for name in ["management.sqlite3","profile-key.hex","setup-token.txt"] {
        let mut entry=archive.by_name(name)?;ensure!(entry.size()<=512*1024*1024,"Backup entry too large");
        let mut content=Vec::new();entry.read_to_end(&mut content)?;files.push((name,content));
    }
    std::fs::create_dir_all(data)?;
    #[cfg(unix)] {use std::os::unix::fs::PermissionsExt;std::fs::set_permissions(data,std::fs::Permissions::from_mode(0o700))?;}
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
