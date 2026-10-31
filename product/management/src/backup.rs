use aes_gcm::{aead::{Aead,KeyInit},Aes256Gcm,Nonce};
use anyhow::{ensure,Context,Result};
use rand::RngCore;
use std::{io::{Cursor,Read,Write},path::Path};

#[path="backup_stream.rs"]
mod stream;
const HEADER:&[u8]=b"SWANBK1";
fn key(password:&str,salt:&[u8])->Result<[u8;32]> {
    ensure!(password.len()>=16 && password.len()<=1024,"Backup passphrase must contain 16–1024 characters");
    let mut key=[0u8;32];argon2::Argon2::default().hash_password_into(password.as_bytes(),salt,&mut key).map_err(|_|anyhow::anyhow!("Backup key derivation failed"))?;Ok(key)
}
pub fn export(data:&Path,destination:&Path,password:&str)->Result<()> {
    export_complete(data,destination,password,None,None,None)
}
pub fn export_complete(data:&Path,destination:&Path,password:&str,transport:Option<&Path>,configuration:Option<&Path>,tls_identity:Option<&Path>)->Result<()> {
    export_deployment(data,destination,password,transport,configuration,tls_identity,None)
}
fn tls_entry(name:&str)->bool {
    name.strip_prefix("tls-storage/").is_some_and(|relative| {
        !relative.is_empty() && !relative.contains(['\\',':']) &&
        relative.split('/').count()<=32 && relative.split('/').all(|part| {
            let stem=part.split('.').next().unwrap_or_default().to_ascii_uppercase();
            !part.is_empty() && !part.ends_with(['.',' ']) && !part.chars().any(|c|c.is_control() || "<>\"|?*".contains(c)) &&
            !matches!(stem.as_str(),"CON"|"PRN"|"AUX"|"NUL"|"COM1"|"COM2"|"COM3"|"COM4"|"COM5"|"COM6"|"COM7"|"COM8"|"COM9"|"LPT1"|"LPT2"|"LPT3"|"LPT4"|"LPT5"|"LPT6"|"LPT7"|"LPT8"|"LPT9")
        })
    })
}
fn collect_tls(root:&Path,directory:&Path,files:&mut Vec<(String,std::path::PathBuf)>)->Result<()> {
    let metadata=std::fs::symlink_metadata(directory)?;
    ensure!(!metadata.file_type().is_symlink(),"TLS storage must not contain symbolic links");
    #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"TLS storage must not contain reparse points");}
    ensure!(metadata.is_dir(),"TLS storage must be a directory");
    for entry in std::fs::read_dir(directory)? {
        let path=entry?.path();let metadata=std::fs::symlink_metadata(&path)?;
        ensure!(!metadata.file_type().is_symlink(),"TLS storage must not contain symbolic links");
        #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"TLS storage must not contain reparse points");}
        let relative=path.strip_prefix(root)?.to_str().context("TLS storage filenames must be UTF-8")?.replace('\\',"/");
        let name=format!("tls-storage/{relative}");ensure!(tls_entry(&name),"Unsupported TLS storage path");
        if metadata.is_dir(){collect_tls(root,&path,files)?;}else{
            ensure!(metadata.is_file(),"TLS storage must contain regular files only");
            ensure!(files.len()<4096,"Too many backup files");files.push((name,path));
        }
    }
    Ok(())
}
pub fn export_deployment(data:&Path,destination:&Path,password:&str,transport:Option<&Path>,configuration:Option<&Path>,tls_identity:Option<&Path>,tls_directory:Option<&Path>)->Result<()> {
    export_deployment_with_artifacts(data,destination,password,transport,configuration,tls_identity,tls_directory,None)
}
fn artifact_entry(name:&str)->bool {
    name.strip_prefix("artifacts/").and_then(|name|name.strip_suffix(".zip"))
        .is_some_and(|id|uuid::Uuid::parse_str(id).is_ok_and(|parsed|parsed.to_string()==id))
}
fn snapshot_transport(directory:&Path,data:&Path,destination:&Path)->Result<()> {
    // A stopped WAL database can need SHM initialization or recovery, which
    // cannot occur on a read-only transport mount. Recover a private copy;
    // never ignore WAL frames or change the original transport storage.
    let staging=data.join(format!("backup-transport-source-{}",swan_protocol::random_token()));
    let builder=std::fs::DirBuilder::new();
    #[cfg(unix)] let mut builder=builder;
    #[cfg(unix)] {use std::os::unix::fs::DirBuilderExt;builder.mode(0o700);}
    builder.create(&staging).context("Create private transport backup staging")?;
    let result=(||->Result<()> {
        let mut total=0u64;
        for name in ["db_v2.sqlite3","db_v2.sqlite3-wal","db_v2.sqlite3-journal"] {
            let source=directory.join(name);
            let metadata=match std::fs::symlink_metadata(&source) {
                Ok(value)=>value,
                Err(error) if error.kind()==std::io::ErrorKind::NotFound && name!="db_v2.sqlite3"=>continue,
                Err(error)=>return Err(error).context("Inspect stopped transport backup source"),
            };
            ensure!(metadata.is_file() && !metadata.file_type().is_symlink(),"Transport database backup source must be a regular file");
            #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"Transport database backup source must not be a reparse point");}
            total=total.checked_add(metadata.len()).context("Transport backup source size overflow")?;
            ensure!(total<=512*1024*1024,"Transport backup source exceeds 512 MiB");
            let input=std::fs::File::open(&source)?;
            let mut options=std::fs::OpenOptions::new();options.create_new(true).write(true);
            #[cfg(unix)] {use std::os::unix::fs::OpenOptionsExt;options.mode(0o600);}
            let mut output=options.open(staging.join(name))?;
            ensure!(std::io::copy(&mut input.take(metadata.len()+1),&mut output)?==metadata.len(),"Transport database changed during backup; stop transport before export");
        }
        let transport_db=rusqlite::Connection::open(staging.join("db_v2.sqlite3")).context("Open staged transport database for backup")?;
        transport_db.backup(rusqlite::DatabaseName::Main,destination,None).context("Snapshot staged transport database for backup")?;
        Ok(())
    })();
    // Only remove the fixed files in the directory created by this operation.
    // SQLite handles are dropped before cleanup, including on a failed snapshot.
    for name in ["db_v2.sqlite3","db_v2.sqlite3-wal","db_v2.sqlite3-shm","db_v2.sqlite3-journal"] {
        match std::fs::remove_file(staging.join(name)) {
            Ok(())=>{},Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
            Err(error)=>return Err(error).context("Remove private transport backup staging file"),
        }
    }
    std::fs::remove_dir(&staging).context("Remove private transport backup staging directory")?;
    result
}

pub fn migrate_transport_database(data:&Path,source:&Path)->Result<()> {
    ensure!(data.is_absolute() && source.is_absolute(),"Company and source directories must be absolute");
    let transport=data.join("transport");
    for directory in [data,source,transport.as_path()] {
        for parent in directory.ancestors() {
            let metadata=std::fs::symlink_metadata(parent)?;
            ensure!(metadata.is_dir() && !metadata.file_type().is_symlink(),"Migration directories must not contain links");
            #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"Migration directories must not contain reparse points");}
        }
    }
    for path in [data.join("management.sqlite3"),data.join("management.lock"),transport.join("id_ed25519"),transport.join("id_ed25519.pub")] {
        let metadata=std::fs::symlink_metadata(path)?;
        ensure!(metadata.is_file() && !metadata.file_type().is_symlink(),"Existing company database, lock and transport keys are required");
        #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"Migration inputs must not be reparse points");}
    }
    let lock=std::fs::OpenOptions::new().read(true).write(true).open(data.join("management.lock"))?;
    fs2::FileExt::try_lock_exclusive(&lock).context("Stop company management before transport database migration")?;
    let target=transport.join("db_v2.sqlite3");
    ensure!(matches!(std::fs::symlink_metadata(&target),Err(error) if error.kind()==std::io::ErrorKind::NotFound),"Refusing to overwrite an existing managed transport database");
    let mut sources=Vec::new();
    for name in ["db_v2.sqlite3","db_v2.sqlite3-wal","db_v2.sqlite3-journal"] {
        let path=source.join(name);
        let metadata=match std::fs::symlink_metadata(&path) {
            Ok(metadata)=>metadata,
            Err(error) if error.kind()==std::io::ErrorKind::NotFound && name!="db_v2.sqlite3"=>continue,
            Err(error)=>return Err(error).context("Inspect stopped source transport database"),
        };
        ensure!(metadata.is_file() && !metadata.file_type().is_symlink(),"Migration source must be a regular database file");
        #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"Migration source must not be a reparse point");}
        let mut options=std::fs::OpenOptions::new();options.read(true);
        // Deny an existing or new Windows writer while copying the stopped DB/WAL.
        #[cfg(windows)] {use std::os::windows::fs::OpenOptionsExt;options.share_mode(1);}
        sources.push(options.open(path).context("Stop rendezvous before migrating its database")?);
    }
    let staging=stream::Staging::new(data)?;
    let snapshot=staging.0.join("transport.sqlite3");
    snapshot_transport(source,data,&snapshot)?;
    let database=rusqlite::Connection::open_with_flags(&snapshot,rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY)?;
    let integrity:String=database.query_row("PRAGMA quick_check",[],|row|row.get(0))?;
    ensure!(integrity=="ok","Migrated transport database integrity check failed");
    drop(database);
    std::fs::OpenOptions::new().read(true).write(true).open(&snapshot)?.sync_all()?;
    // Publish atomically without replacing another migration's destination.
    std::fs::hard_link(&snapshot,&target).context("Publish managed transport database without overwriting existing state")?;
    drop(sources);
    Ok(())
}
pub fn export_deployment_with_artifacts(data:&Path,destination:&Path,password:&str,transport:Option<&Path>,configuration:Option<&Path>,tls_identity:Option<&Path>,tls_directory:Option<&Path>,artifacts:Option<&Path>)->Result<()> {
    ensure!(!destination.exists(),"Refusing to overwrite an existing backup");
    let db=rusqlite::Connection::open_with_flags(data.join("management.sqlite3"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).context("Open management database for backup")?;
    let snapshot=data.join(format!("backup-{}.sqlite3",swan_protocol::random_token()));
    db.backup(rusqlite::DatabaseName::Main,&snapshot,None).context("Snapshot management database for backup")?;
    let mut snapshots=vec![snapshot.clone()];
    let result=(||->Result<()> {
        let mut files=vec![("management.sqlite3".to_string(),snapshot.clone()),("profile-key.hex".to_string(),data.join("profile-key.hex")),("setup-token.txt".to_string(),data.join("setup-token.txt"))];
        if let Some(directory)=transport {
            #[cfg(windows)] {crate::components::transport_database_path(directory,&crate::components::legacy_transport_database()?)?;}
            files.push(("transport-id_ed25519".into(),directory.join("id_ed25519")));files.push(("transport-id_ed25519.pub".into(),directory.join("id_ed25519.pub")));
            if directory.join("db_v2.sqlite3").exists(){
                let transport_snapshot=data.join(format!("backup-transport-{}.sqlite3",swan_protocol::random_token()));snapshots.push(transport_snapshot.clone());
                snapshot_transport(directory,data,&transport_snapshot)?;
                files.push(("transport-db.sqlite3".into(),transport_snapshot));
            }
        }
        if let Some(path)=configuration {files.push(("deployment.env".into(),path.to_path_buf()));}
        if let Some(path)=tls_identity {files.push(("tls-identity".into(),path.to_path_buf()));}
        if let Some(directory)=tls_directory {collect_tls(directory,directory,&mut files)?;}
        if let Some(directory)=artifacts {
            let metadata=std::fs::symlink_metadata(directory)?;
            ensure!(metadata.is_dir() && !metadata.file_type().is_symlink(),"Installer artifacts must be a regular directory");
            #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"Installer artifacts must not contain reparse points");}
            for entry in std::fs::read_dir(directory)? {
                let path=entry?.path();let metadata=std::fs::symlink_metadata(&path)?;
                ensure!(metadata.is_file() && !metadata.file_type().is_symlink(),"Installer artifacts must contain regular files only");
                #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"Installer artifacts must not contain reparse points");}
                let name=format!("artifacts/{}",path.file_name().and_then(|name|name.to_str()).context("Installer artifact name must be UTF-8")?);
                ensure!(artifact_entry(&name),"Installer artifact must use its canonical build UUID and ZIP extension; finish uploads before export");
                ensure!(files.len()<4096,"Too many backup files");files.push((name,path));
            }
        }
        let proxy=data.join("Caddyfile");
        match std::fs::symlink_metadata(&proxy) {
            Ok(metadata)=>{
                ensure!(metadata.is_file() && !metadata.file_type().is_symlink(),"HTTPS configuration must be a regular file");
                #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"HTTPS configuration must not be a reparse point");}
                files.push(("Caddyfile".into(),proxy));
            },
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
            Err(error)=>return Err(error).context("Cannot inspect HTTPS configuration for backup"),
        }
        let receipt=data.join("installation.json");
        match std::fs::symlink_metadata(&receipt) {
            Ok(metadata)=>{
                ensure!(metadata.is_file() && !metadata.file_type().is_symlink() && (1..=64*1024).contains(&metadata.len()),"Windows installation receipt must be a bounded regular file");
                #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"Windows installation receipt must not be a reparse point");}
                files.push(("installation.json".into(),receipt));
            },
            Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
            Err(error)=>return Err(error).context("Cannot inspect Windows installation receipt for backup"),
        }
        ensure!(files.len()<=4096,"Too many backup files");
        stream::export(data,files,destination,password)
    })();
    for snapshot in snapshots {if snapshot.exists(){std::fs::remove_file(&snapshot).context("Remove plaintext database snapshot after backup")?;}}
    result
}
fn empty_target(data:&Path)->Result<()> {
    match std::fs::symlink_metadata(data) {
        Ok(metadata)=>{
            ensure!(metadata.is_dir() && !metadata.file_type().is_symlink(),"Restore target must be a regular directory");
            #[cfg(windows)] {use std::os::windows::fs::MetadataExt;ensure!(metadata.file_attributes()&0x400==0,"Restore target must not be a reparse point");}
            ensure!(std::fs::read_dir(data)?.next().is_none(),"Restore requires an empty target directory and a stopped management service");
        },
        Err(error) if error.kind()==std::io::ErrorKind::NotFound=>{},
        Err(error)=>return Err(error.into()),
    }
    Ok(())
}
pub fn restore(data:&Path,source:&Path,password:&str)->Result<()> {
    empty_target(data)?;
    let mut header=[0u8;7];std::fs::File::open(source)?.read_exact(&mut header)?;
    if header.as_slice()==stream::HEADER {return stream::restore(data,source,password);}
    // Legacy archives retain their original bounded-memory reader and limits.
    ensure!(std::fs::metadata(source)?.len()<=1024*1024*1024,"Backup exceeds size limit");
    let bytes=std::fs::read(source)?;
    ensure!(bytes.len()>HEADER.len()+28 && bytes.starts_with(HEADER),"Invalid backup format");
    let offset=HEADER.len();let salt=&bytes[offset..offset+16];let nonce=&bytes[offset+16..offset+28];
    let cipher=Aes256Gcm::new_from_slice(&key(password,salt)?).map_err(|_|anyhow::anyhow!("Invalid decryption key"))?;
    let plaintext=cipher.decrypt(Nonce::from_slice(nonce),&bytes[offset+28..]).map_err(|_|anyhow::anyhow!("Backup password incorrect or backup modified"))?;
    let mut archive=zip::ZipArchive::new(Cursor::new(plaintext))?;
    restore_archive(data,&mut archive,512*1024*1024)
}
fn restore_archive<R:Read+std::io::Seek>(data:&Path,archive:&mut zip::ZipArchive<R>,limit:u64)->Result<()> {
    ensure!((3..=4096).contains(&archive.len()),"Unexpected backup contents");
    let allowed=["management.sqlite3","profile-key.hex","setup-token.txt","transport-id_ed25519","transport-id_ed25519.pub","transport-db.sqlite3","deployment.env","tls-identity","Caddyfile","installation.json"];
    let mut names=std::collections::HashSet::new();
    for index in 0..archive.len(){let entry=archive.by_index(index)?;ensure!((allowed.contains(&entry.name()) || tls_entry(entry.name()) || artifact_entry(entry.name())) && !entry.is_dir() && names.insert(entry.name().to_string()),"Unknown or duplicate backup entry");}
    let mut portable_names=std::collections::HashSet::new();
    for name in &names {ensure!(portable_names.insert(name.to_ascii_lowercase()),"Backup filenames collide on Windows");}
    for name in &portable_names {
        let mut parent=name.as_str();while let Some((prefix,_))=parent.rsplit_once('/') {ensure!(!portable_names.contains(prefix),"Backup file conflicts with a directory");parent=prefix;}
    }
    ensure!(names.contains("transport-id_ed25519")==names.contains("transport-id_ed25519.pub"),"Incomplete transport trust key pair");
    for required in &allowed[..3]{ensure!(names.contains(*required),"Missing management backup entry");}
    // Bound all expanded sizes before writing to private staging. ZIP CRC and
    // exact lengths are checked during extraction, before publishing the target.
    let mut total=0u64;
    for name in &names {
        let entry=archive.by_name(name)?;
        if name=="installation.json" {ensure!((1..=64*1024).contains(&entry.size()),"Invalid Windows installation receipt size");}
        total=total.checked_add(entry.size()).context("Backup size overflow")?;
        ensure!(total<=limit,"Backup contents too large");
    }
    stream::extract(data,archive,names)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn migration_fixture()->(std::path::PathBuf,std::path::PathBuf,std::path::PathBuf) {
        let root=std::env::temp_dir().join(format!("swan-transport-migrate-{}",swan_protocol::random_token()));
        let data=root.join("company");let source=root.join("stopped-source");
        std::fs::create_dir_all(data.join("transport")).unwrap();std::fs::create_dir(&source).unwrap();
        for name in ["management.sqlite3","management.lock","transport/id_ed25519","transport/id_ed25519.pub"] {std::fs::write(data.join(name),b"owned company fixture").unwrap();}
        (root,data,source)
    }
    #[test]
    fn transport_migration_recovers_stopped_wal_without_changing_source_or_keys() {
        let (root,data,source)=migration_fixture();
        let original=root.join("writer.sqlite3");let writer=rusqlite::Connection::open(&original).unwrap();
        writer.execute_batch("PRAGMA journal_mode=WAL; PRAGMA wal_autocheckpoint=0; CREATE TABLE peer(id TEXT PRIMARY KEY); INSERT INTO peer VALUES ('preserved-fixture');").unwrap();
        std::fs::copy(&original,source.join("db_v2.sqlite3")).unwrap();std::fs::copy(root.join("writer.sqlite3-wal"),source.join("db_v2.sqlite3-wal")).unwrap();
        let source_db=std::fs::read(source.join("db_v2.sqlite3")).unwrap();let source_wal=std::fs::read(source.join("db_v2.sqlite3-wal")).unwrap();
        migrate_transport_database(&data,&source).unwrap();
        let migrated=rusqlite::Connection::open(data.join("transport/db_v2.sqlite3")).unwrap();
        assert_eq!(migrated.query_row("SELECT id FROM peer",[],|row|row.get::<_,String>(0)).unwrap(),"preserved-fixture");
        assert_eq!(std::fs::read(source.join("db_v2.sqlite3")).unwrap(),source_db);assert_eq!(std::fs::read(source.join("db_v2.sqlite3-wal")).unwrap(),source_wal);
        assert!(!source.join("db_v2.sqlite3-shm").exists());
        for name in ["id_ed25519","id_ed25519.pub"] {assert_eq!(std::fs::read(data.join("transport").join(name)).unwrap(),b"owned company fixture");}
        assert!(std::fs::read_dir(&data).unwrap().all(|entry|{let entry=entry.unwrap();let name=entry.file_name();let name=name.to_string_lossy();!name.starts_with(".swan-backup-") && !name.starts_with("backup-transport-source-")}));
        drop(migrated);drop(writer);std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn transport_migration_refuses_existing_target_active_management_and_corrupt_source() {
        let (root,data,source)=migration_fixture();let target=data.join("transport/db_v2.sqlite3");
        std::fs::write(&target,b"retained original target").unwrap();
        assert!(migrate_transport_database(&data,&source).is_err());assert_eq!(std::fs::read(&target).unwrap(),b"retained original target");std::fs::remove_file(&target).unwrap();
        let lock=std::fs::OpenOptions::new().read(true).write(true).open(data.join("management.lock")).unwrap();fs2::FileExt::try_lock_exclusive(&lock).unwrap();
        assert!(migrate_transport_database(&data,&source).is_err());assert!(!target.exists());drop(lock);
        std::fs::write(source.join("db_v2.sqlite3"),b"corrupt source fixture").unwrap();
        assert!(migrate_transport_database(&data,&source).is_err());assert!(!target.exists());assert_eq!(std::fs::read(source.join("db_v2.sqlite3")).unwrap(),b"corrupt source fixture");
        assert!(!std::fs::read_dir(&data).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with(".swan-backup-")));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[cfg(windows)]
    #[test]
    fn transport_migration_refuses_a_running_windows_database_writer() {
        let (root,data,source)=migration_fixture();let writer=rusqlite::Connection::open(source.join("db_v2.sqlite3")).unwrap();
        writer.execute_batch("CREATE TABLE peer(id TEXT);").unwrap();
        assert!(migrate_transport_database(&data,&source).is_err());assert!(!data.join("transport/db_v2.sqlite3").exists());
        drop(writer);std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn failed_transport_snapshot_removes_private_staging_without_changing_source() {
        let root=std::env::temp_dir().join(format!("swan-failed-transport-backup-{}",swan_protocol::random_token()));
        let data=root.join("data");let transport=root.join("transport");std::fs::create_dir_all(&data).unwrap();std::fs::create_dir(&transport).unwrap();
        let db=rusqlite::Connection::open(data.join("management.sqlite3")).unwrap();db.execute_batch("CREATE TABLE identity(value TEXT);").unwrap();drop(db);
        std::fs::write(data.join("profile-key.hex"),b"fixture key").unwrap();std::fs::write(data.join("setup-token.txt"),b"fixture token").unwrap();
        let invalid=b"corrupt transport database fixture";
        std::fs::write(transport.join("db_v2.sqlite3"),invalid).unwrap();
        std::fs::write(transport.join("id_ed25519"),b"fixture private key").unwrap();std::fs::write(transport.join("id_ed25519.pub"),b"fixture public key").unwrap();
        let archive=root.join("rejected.backup");
        let error=export_complete(&data,&archive,"failed backup test passphrase",Some(&transport),None,None).unwrap_err();
        assert!(format!("{error:#}").contains("transport database"));assert!(!archive.exists());
        assert_eq!(std::fs::read(transport.join("db_v2.sqlite3")).unwrap(),invalid);
        assert!(std::fs::read_dir(&data).unwrap().all(|entry|!entry.unwrap().file_name().to_string_lossy().starts_with("backup-")),"Failed export must remove plaintext snapshots and staging");
        std::fs::remove_dir_all(root).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn readonly_transport_wal_export_preserves_committed_peers() {
        use std::os::unix::fs::PermissionsExt;
        let root=std::env::temp_dir().join(format!("swan-readonly-wal-backup-{}",swan_protocol::random_token()));
        let data=root.join("data");let original=root.join("original");let transport=root.join("transport");
        for directory in [&data,&original,&transport]{std::fs::create_dir_all(directory).unwrap();}
        let management=rusqlite::Connection::open(data.join("management.sqlite3")).unwrap();
        management.execute_batch("CREATE TABLE identity(value TEXT);INSERT INTO identity VALUES('retained company');").unwrap();drop(management);
        std::fs::write(data.join("profile-key.hex"),b"fixture key").unwrap();std::fs::write(data.join("setup-token.txt"),b"fixture token").unwrap();
        let writer=rusqlite::Connection::open(original.join("db_v2.sqlite3")).unwrap();
        writer.execute_batch("PRAGMA journal_mode=WAL;PRAGMA wal_autocheckpoint=0;CREATE TABLE peers(id TEXT);INSERT INTO peers VALUES('committed WAL peer');").unwrap();
        // Model stopped transport storage with committed WAL frames and no SHM
        // file, as a read-only backup mount may present it after interruption.
        for name in ["db_v2.sqlite3","db_v2.sqlite3-wal"]{std::fs::copy(original.join(name),transport.join(name)).unwrap();}
        let source_database=std::fs::read(transport.join("db_v2.sqlite3")).unwrap();let source_wal=std::fs::read(transport.join("db_v2.sqlite3-wal")).unwrap();
        std::fs::write(transport.join("id_ed25519"),b"fixture private key").unwrap();std::fs::write(transport.join("id_ed25519.pub"),b"fixture public key").unwrap();
        std::fs::set_permissions(&transport,std::fs::Permissions::from_mode(0o555)).unwrap();
        let probe=transport.join("write-probe");
        if std::fs::OpenOptions::new().create_new(true).write(true).open(&probe).is_ok(){
            // Root bypasses Unix mode bits; actual read-only mount coverage is
            // provided by the separate container deployment rehearsal.
            std::fs::set_permissions(&transport,std::fs::Permissions::from_mode(0o755)).unwrap();drop(writer);std::fs::remove_dir_all(root).unwrap();return;
        }
        let archive=root.join("complete.backup");
        let result=export_complete(&data,&archive,"readonly WAL test passphrase",Some(&transport),None,None);
        let source_unchanged=!transport.join("db_v2.sqlite3-shm").exists();
        std::fs::set_permissions(&transport,std::fs::Permissions::from_mode(0o755)).unwrap();drop(writer);
        result.unwrap();assert!(source_unchanged,"Export must not write into read-only transport storage");
        assert_eq!(std::fs::read(transport.join("db_v2.sqlite3")).unwrap(),source_database);assert_eq!(std::fs::read(transport.join("db_v2.sqlite3-wal")).unwrap(),source_wal);
        assert!(std::fs::read_dir(&data).unwrap().all(|entry|!entry.unwrap().file_name().to_string_lossy().starts_with("backup-")),"Plaintext snapshots and staging must be removed");
        let restored=root.join("restored");restore(&restored,&archive,"readonly WAL test passphrase").unwrap();
        let db=rusqlite::Connection::open(restored.join("transport-db.sqlite3")).unwrap();
        assert_eq!(db.query_row("SELECT id FROM peers",[],|row|row.get::<_,String>(0)).unwrap(),"committed WAL peer");drop(db);
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn installer_artifacts_roundtrip_and_reject_unfinished_or_unsafe_entries() {
        let root=std::env::temp_dir().join(format!("swan-artifact-backup-{}",swan_protocol::random_token()));
        let data=root.join("data");let artifacts=data.join("artifacts");std::fs::create_dir_all(&artifacts).unwrap();
        let db=rusqlite::Connection::open(data.join("management.sqlite3")).unwrap();db.execute_batch("CREATE TABLE builds(id TEXT);INSERT INTO builds VALUES('retained');").unwrap();drop(db);
        std::fs::write(data.join("profile-key.hex"),b"key").unwrap();std::fs::write(data.join("setup-token.txt"),b"token").unwrap();
        let id=uuid::Uuid::new_v4().to_string();let name=format!("{id}.zip");std::fs::write(artifacts.join(&name),b"retained installer bytes").unwrap();
        let archive=root.join("complete.backup");
        export_deployment_with_artifacts(&data,&archive,"installer backup test password",None,None,None,None,Some(&artifacts)).unwrap();
        let restored=root.join("restored");restore(&restored,&archive,"installer backup test password").unwrap();
        assert_eq!(std::fs::read(restored.join("artifacts").join(&name)).unwrap(),b"retained installer bytes");
        for name in ["artifacts/../outside.zip","artifacts/not-a-build.zip","artifacts/00000000-0000-0000-0000-000000000000.zip/child","artifacts/00000000-0000-0000-0000-000000000000.partial"] {assert!(!artifact_entry(name));}
        std::fs::write(artifacts.join(format!("{id}.partial")),b"unfinished upload").unwrap();
        let rejected=root.join("unfinished.backup");
        assert!(export_deployment_with_artifacts(&data,&rejected,"installer backup test password",None,None,None,None,Some(&artifacts)).is_err());assert!(!rejected.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn complete_backup_preserves_transport_configuration_and_tls_and_rejects_tampering() {
        let root=std::env::temp_dir().join(format!("swan-full-backup-{}",swan_protocol::random_token()));
        let data=root.join("data");let transport=root.join("transport");std::fs::create_dir_all(&data).unwrap();std::fs::create_dir(&transport).unwrap();
        let db=rusqlite::Connection::open(data.join("management.sqlite3")).unwrap();db.execute_batch("CREATE TABLE identity(value TEXT);INSERT INTO identity VALUES('company identity');").unwrap();drop(db);
        std::fs::write(data.join("profile-key.hex"),"management signing key").unwrap();std::fs::write(data.join("setup-token.txt"),"setup token").unwrap();
        std::fs::write(transport.join("id_ed25519"),b"private transport key").unwrap();std::fs::write(transport.join("id_ed25519.pub"),b"public transport key").unwrap();
        let db=rusqlite::Connection::open(transport.join("db_v2.sqlite3")).unwrap();db.execute_batch("CREATE TABLE peers(id TEXT);INSERT INTO peers VALUES('retained peer');").unwrap();drop(db);
        let configuration=root.join("company.env");let tls=root.join("server.pfx");std::fs::write(&configuration,b"company settings and TLS password").unwrap();std::fs::write(&tls,b"TLS identity fixture").unwrap();
        let tls_storage=root.join("caddy");let certificate=tls_storage.join("certificates/acme.example/company.example/company.key");
        std::fs::create_dir_all(certificate.parent().unwrap()).unwrap();std::fs::write(&certificate,b"ACME private key fixture").unwrap();
        std::fs::write(tls_storage.join("account.json"),b"ACME account fixture").unwrap();
        std::fs::write(data.join("Caddyfile"),b"company.example { reverse_proxy 127.0.0.1:24440 }").unwrap();
        let receipt=br#"{"schema":1,"service_name":"SwanCompanyServer","full_stack":true}"#;
        std::fs::write(data.join("installation.json"),receipt).unwrap();
        let archive=root.join("complete.swan-backup");export_deployment(&data,&archive,"complete backup passphrase",Some(&transport),Some(&configuration),Some(&tls),Some(&tls_storage)).unwrap();
        let destination=root.join("restored");restore(&destination,&archive,"complete backup passphrase").unwrap();
        assert_eq!(std::fs::read(destination.join("transport-id_ed25519")).unwrap(),b"private transport key");assert_eq!(std::fs::read(destination.join("transport-id_ed25519.pub")).unwrap(),b"public transport key");
        assert_eq!(std::fs::read(destination.join("deployment.env")).unwrap(),std::fs::read(configuration).unwrap());assert_eq!(std::fs::read(destination.join("tls-identity")).unwrap(),std::fs::read(tls).unwrap());
        assert_eq!(std::fs::read(destination.join("tls-storage/certificates/acme.example/company.example/company.key")).unwrap(),std::fs::read(certificate).unwrap());
        assert_eq!(std::fs::read(destination.join("tls-storage/account.json")).unwrap(),b"ACME account fixture");
        assert_eq!(std::fs::read(destination.join("Caddyfile")).unwrap(),std::fs::read(data.join("Caddyfile")).unwrap());
        assert_eq!(std::fs::read(destination.join("installation.json")).unwrap(),receipt);
        let db=rusqlite::Connection::open(destination.join("transport-db.sqlite3")).unwrap();assert_eq!(db.query_row("SELECT id FROM peers",[],|row|row.get::<_,String>(0)).unwrap(),"retained peer");drop(db);
        let mut modified=std::fs::read(&archive).unwrap();let last=modified.len()-1;modified[last]^=1;let tampered=root.join("tampered.swan-backup");std::fs::write(&tampered,modified).unwrap();
        let rejected=root.join("rejected");assert!(restore(&rejected,&tampered,"complete backup passphrase").is_err());assert!(!rejected.exists());
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn installation_receipt_rejects_invalid_file_types_and_sizes_before_export() {
        for kind in ["directory","empty","oversized"] {
            let root=std::env::temp_dir().join(format!("swan-receipt-backup-{}",swan_protocol::random_token()));
            let data=root.join("data");std::fs::create_dir_all(&data).unwrap();
            let db=rusqlite::Connection::open(data.join("management.sqlite3")).unwrap();db.execute_batch("CREATE TABLE identity(value TEXT);").unwrap();drop(db);
            std::fs::write(data.join("profile-key.hex"),b"key").unwrap();std::fs::write(data.join("setup-token.txt"),b"token").unwrap();
            let receipt=data.join("installation.json");
            if kind=="directory" {std::fs::create_dir(&receipt).unwrap();}else{std::fs::write(&receipt,vec![b'x';if kind=="empty" {0}else{64*1024+1}]).unwrap();}
            let output=root.join("rejected.backup");assert!(export(&data,&output,"receipt backup test password").is_err());assert!(!output.exists());
            assert!(!std::fs::read_dir(&data).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with("backup-")));
            std::fs::remove_dir_all(root).unwrap();
        }
    }
    #[test]
    fn invalid_receipt_size_in_authenticated_archive_never_publishes_restore() {
        let root=std::env::temp_dir().join(format!("swan-invalid-receipt-restore-{}",swan_protocol::random_token()));std::fs::create_dir(&root).unwrap();
        for size in [0,64*1024+1] {
            let mut archive=zip::ZipWriter::new(Cursor::new(Vec::new()));
            for name in ["management.sqlite3","profile-key.hex","setup-token.txt"] {archive.start_file(name,zip::write::SimpleFileOptions::default()).unwrap();archive.write_all(b"fixture").unwrap();}
            archive.start_file("installation.json",zip::write::SimpleFileOptions::default()).unwrap();archive.write_all(&vec![b'x';size]).unwrap();
            let mut archive=zip::ZipArchive::new(archive.finish().unwrap()).unwrap();let destination=root.join(format!("rejected-{size}"));
            assert!(restore_archive(&destination,&mut archive,1024*1024).is_err());assert!(!destination.exists());
        }
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn tls_archive_paths_cannot_escape_restore_or_collide_with_management() {
        assert!(tls_entry("tls-storage/certificates/company.example/company.key"));
        for name in ["tls-storage/../profile-key.hex","tls-storage/./key","tls-storage//key","tls-storage/C:/key","tls-storage/dir\\key","tls-storage/key\0","tls-storage/","tls-storage/key.","tls-storage/key ","tls-storage/CON.key","tls-storage/NUL","profile-key.hex"] {assert!(!tls_entry(name),"{name:?}");}
        let root=std::env::temp_dir().join(format!("swan-invalid-tls-backup-{}",swan_protocol::random_token()));std::fs::create_dir(&root).unwrap();
        for (index,extras) in [vec!["tls-storage/../outside"],vec!["tls-storage/Key","tls-storage/key"],vec!["tls-storage/file","tls-storage/file/child"]].into_iter().enumerate() {
            let mut archive=zip::ZipWriter::new(Cursor::new(Vec::new()));
            for name in ["management.sqlite3","profile-key.hex","setup-token.txt"].into_iter().chain(extras) {archive.start_file(name,zip::write::SimpleFileOptions::default()).unwrap();archive.write_all(b"fixture").unwrap();}
            let plaintext=archive.finish().unwrap().into_inner();let salt=[1u8;16];let nonce=[2u8;12];
            let cipher=Aes256Gcm::new_from_slice(&key("invalid archive test password",&salt).unwrap()).unwrap();let ciphertext=cipher.encrypt(Nonce::from_slice(&nonce),plaintext.as_ref()).unwrap();
            let mut bytes=HEADER.to_vec();bytes.extend(salt);bytes.extend(nonce);bytes.extend(ciphertext);
            let source=root.join(format!("{index}.backup"));std::fs::write(&source,bytes).unwrap();let destination=root.join(format!("restore-{index}"));
            assert!(restore(&destination,&source,"invalid archive test password").is_err());assert!(!destination.exists());
        }
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
