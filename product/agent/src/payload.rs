//! Verify the signed installed payload, including native libraries and assets.
use anyhow::{ensure,Context,Result};
use std::path::Path;
use swan_protocol::{digest,InstalledFile};

#[cfg(any(windows,test))]
pub(crate) const WINDOWS_BROKER_CACHE:&str="RuntimeBroker_rustdesk.exe";

#[cfg(any(windows,test))]
fn plain_file(path:&Path)->Result<std::fs::Metadata> {
    let metadata=std::fs::symlink_metadata(path)?;
    ensure!(metadata.is_file() && !metadata.file_type().is_symlink(),"Windows component path is not a regular file");
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        ensure!(metadata.file_attributes() & 0x400 == 0,"Windows component path is a reparse point");
    }
    ensure!(metadata.len()>0 && metadata.len()<=512*1024*1024,"Windows component size is invalid");
    Ok(metadata)
}

#[cfg(any(windows,test))]
fn component_bytes(path:&Path)->Result<Vec<u8>> {
    use std::io::Read;
    let metadata=plain_file(path)?;
    let mut bytes=Vec::new();
    std::fs::File::open(path)?.take(512*1024*1024+1).read_to_end(&mut bytes)?;
    ensure!(bytes.len() as u64==metadata.len(),"Windows component changed while reading");
    Ok(bytes)
}

#[cfg(any(windows,test))]
fn windows_cache_path(root:&Path,system_broker:&Path)->Result<std::path::PathBuf> {
    let metadata=std::fs::symlink_metadata(root)?;
    ensure!(metadata.is_dir() && !metadata.file_type().is_symlink(),"Installed directory is not a regular directory");
    #[cfg(windows)] {
        use std::os::windows::fs::MetadataExt;
        ensure!(metadata.file_attributes() & 0x400 == 0,"Installed directory is a reparse point");
    }
    ensure!(system_broker.file_name().and_then(|n|n.to_str()).is_some_and(|n|n.eq_ignore_ascii_case("RuntimeBroker.exe")),"Unexpected Windows component source");
    plain_file(system_broker)?;
    Ok(root.join(WINDOWS_BROKER_CACHE))
}

#[cfg(any(windows,test))]
pub(crate) fn windows_cache_current(root:&Path,system_broker:&Path)->Result<bool> {
    let cache=windows_cache_path(root,system_broker)?;
    match std::fs::symlink_metadata(&cache) {
        Err(error) if error.kind()==std::io::ErrorKind::NotFound=>Ok(false),
        Err(error)=>Err(error.into()),
        Ok(_)=>Ok(component_bytes(&cache)?==component_bytes(system_broker)?),
    }
}

/// The caller holds installation/session exclusion and supplies the OS system
/// directory, never a source from a company profile or downloaded package.
/// Refresh this generated Windows cache without accepting stale bytes as part
/// of the signed project payload or killing an in-use helper.
#[cfg(any(windows,test))]
pub(crate) fn refresh_windows_cache(root:&Path,system_broker:&Path)->Result<bool> {
    use std::io::Write;
    let cache=windows_cache_path(root,system_broker)?;
    if windows_cache_current(root,system_broker)? {return Ok(false);}
    let source=component_bytes(system_broker)?;
    let modified=plain_file(system_broker)?.modified()?;
    let temporary=root.join(format!(".swan-windows-cache-{}.tmp",swan_protocol::random_token()));
    let mut created=false;
    let result=(||->Result<()> {
        let mut file=std::fs::OpenOptions::new().create_new(true).write(true).open(&temporary)?;
        created=true;
        file.write_all(&source)?;
        file.set_times(std::fs::FileTimes::new().set_modified(modified))?;
        file.sync_all()?;drop(file);
        ensure!(component_bytes(system_broker)?==source,"Windows component changed before cache publication");
        crate::replace_state(&temporary,&cache).context("Windows component cache is in use or cannot be replaced")?;
        ensure!(windows_cache_current(root,system_broker)?,"Windows component changed during cache publication");
        Ok(())
    })();
    if created && temporary.exists(){std::fs::remove_file(&temporary).context("Cannot remove unpublished Windows component cache")?;}
    result?;Ok(true)
}

/// Preserve a complete signed payload before an installer can modify it.
/// Only a fully verified, flushed directory becomes visible at destination.
pub(crate) fn snapshot(source:&Path,destination:&Path,files:&[InstalledFile])->Result<()> {
    use std::io::Write;
    ensure!(!destination.exists(),"Rollback snapshot already exists");
    ensure!(!files.is_empty() && files.len()<=4096,"Installed payload manifest required");
    for file in files {file.validate()?;}
    let source=std::fs::canonicalize(source)?;
    let temporary=destination.parent().context("Missing snapshot parent")?.join(format!("swan-snapshot-{}",swan_protocol::random_token()));
    std::fs::create_dir(&temporary)?;
    let result=(||->Result<()> {
        let mut total=0u64;
        for file in files {
            let path=std::fs::canonicalize(source.join(&file.path))?;
            ensure!(path.starts_with(&source) && path.is_file(),"Snapshot source escapes installation");
            let size=std::fs::metadata(&path)?.len();
            ensure!(size<=512*1024*1024,"Snapshot file exceeds size limit");
            total=total.checked_add(size).context("Snapshot size overflow")?;
            ensure!(total<=2*1024*1024*1024,"Snapshot exceeds total size limit");
            let bytes=std::fs::read(path)?;
            ensure!(bytes.len() as u64==size,"Snapshot source changed during copying");
            ensure!(digest(&bytes).eq_ignore_ascii_case(&file.sha256),"Snapshot source differs from signed manifest");
            let target=temporary.join(&file.path);
            std::fs::create_dir_all(target.parent().context("Missing snapshot file parent")?)?;
            let mut output=std::fs::OpenOptions::new().create_new(true).write(true).open(target)?;
            output.write_all(&bytes)?;output.sync_all()?;
        }
        verify(&temporary,files)?;
        std::fs::rename(&temporary,destination).context("Cannot publish complete rollback snapshot")
    })();
    if result.is_err(){std::fs::remove_dir_all(&temporary).context("Cannot clean incomplete rollback snapshot")?;}
    result
}

pub fn verify(root:&Path,files:&[InstalledFile])->Result<()> {
    ensure!(!files.is_empty() && files.len()<=4096,"Installed payload manifest required");
    let root=std::fs::canonicalize(root)?;
    let mut expected=std::collections::HashSet::new();
    for file in files {
        file.validate()?;
        ensure!(expected.insert(file.path.to_ascii_lowercase()),"Duplicate installed file path");
        let path=std::fs::canonicalize(root.join(&file.path)).context("Installed payload file missing")?;
        ensure!(path.starts_with(&root) && path.is_file(),"Installed payload escapes its directory");
        ensure!(std::fs::metadata(&path)?.len()<=512*1024*1024,"Installed payload file exceeds size limit");
        ensure!(digest(std::fs::read(path)?).eq_ignore_ascii_case(&file.sha256),"Installed payload hash differs from signed manifest");
    }
    // Additional native code can be loaded without changing the main EXE.
    // Reject unlisted DLLs/EXEs rather than treating them as ordinary user data.
    let mut directories=vec![(root.clone(),0u32)];let mut count=0usize;
    while let Some((directory,depth))=directories.pop() {
        ensure!(depth<=32,"Installed directory nesting exceeds limit");
        for entry in std::fs::read_dir(directory)? {
            let entry=entry?;count+=1;ensure!(count<=16384,"Installed directory exceeds verification limit");
            let path=entry.path();let canonical=std::fs::canonicalize(&path)?;
            ensure!(canonical.starts_with(&root),"Installed directory contains an external link");
            if canonical.is_dir(){directories.push((path,depth+1));continue;}
            let extension=path.extension().and_then(|value|value.to_str()).unwrap_or("");
            if extension.eq_ignore_ascii_case("exe") || extension.eq_ignore_ascii_case("dll") {
                let relative=path.strip_prefix(&root)?.to_str().context("Invalid installed filename")?.replace('\\',"/").to_ascii_lowercase();
                ensure!(expected.contains(&relative),"Installed native code is absent from signed manifest");
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn windows_owned_cache_refreshes_after_os_change_without_changing_project_files(){
        let base=std::env::temp_dir().join(format!("swan-os-cache-{}",swan_protocol::random_token()));
        let root=base.join("installed");let system=base.join("system");
        std::fs::create_dir_all(&root).unwrap();std::fs::create_dir(&system).unwrap();
        let source=system.join("RuntimeBroker.exe");let cache=root.join(WINDOWS_BROKER_CACHE);
        std::fs::write(root.join("endpoint.exe"),b"approved project bytes").unwrap();
        std::fs::write(&source,b"updated Windows component").unwrap();
        std::fs::write(&cache,b"previous Windows component").unwrap();
        assert!(!windows_cache_current(&root,&source).unwrap());
        assert!(refresh_windows_cache(&root,&source).unwrap());
        assert_eq!(std::fs::read(&cache).unwrap(),std::fs::read(&source).unwrap());
        assert_eq!(std::fs::metadata(&cache).unwrap().modified().unwrap(),std::fs::metadata(&source).unwrap().modified().unwrap());
        assert!(!refresh_windows_cache(&root,&source).unwrap());
        let files=vec![InstalledFile{path:"endpoint.exe".into(),sha256:digest(b"approved project bytes")},InstalledFile{path:WINDOWS_BROKER_CACHE.into(),sha256:digest(b"updated Windows component")}];
        verify(&root,&files).unwrap();
        std::fs::write(root.join("endpoint.exe"),b"tampered project bytes").unwrap();
        assert!(verify(&root,&files).is_err());
        assert!(!std::fs::read_dir(&root).unwrap().any(|e|e.unwrap().file_name().to_string_lossy().starts_with(".swan-windows-cache-")));
        std::fs::remove_dir_all(base).unwrap();
    }
    #[test]
    fn windows_cache_rejects_missing_source_wrong_source_name_and_directory_target(){
        let base=std::env::temp_dir().join(format!("swan-os-cache-reject-{}",swan_protocol::random_token()));
        let root=base.join("installed");std::fs::create_dir_all(&root).unwrap();
        let source=base.join("RuntimeBroker.exe");let cache=root.join(WINDOWS_BROKER_CACHE);
        std::fs::write(&cache,b"original cache").unwrap();
        assert!(refresh_windows_cache(&root,&source).is_err());
        std::fs::write(&source,b"").unwrap();
        assert!(refresh_windows_cache(&root,&source).is_err());
        let wrong=base.join("profile-selected.exe");std::fs::write(&wrong,b"untrusted source").unwrap();
        assert!(refresh_windows_cache(&root,&wrong).is_err());
        assert_eq!(std::fs::read(&cache).unwrap(),b"original cache");
        std::fs::remove_file(&cache).unwrap();std::fs::create_dir(&cache).unwrap();std::fs::write(&source,b"system component").unwrap();
        assert!(refresh_windows_cache(&root,&source).is_err());assert!(cache.is_dir());
        std::fs::remove_dir_all(base).unwrap();
    }
    #[cfg(unix)]
    #[test]
    fn windows_cache_rejects_links_without_changing_external_files(){
        let base=std::env::temp_dir().join(format!("swan-os-cache-links-{}",swan_protocol::random_token()));
        let root=base.join("installed");std::fs::create_dir_all(&root).unwrap();
        let source=base.join("RuntimeBroker.exe");let external=base.join("outside.exe");
        std::fs::write(&source,b"system component").unwrap();std::fs::write(&external,b"must remain unchanged").unwrap();
        std::os::unix::fs::symlink(&external,root.join(WINDOWS_BROKER_CACHE)).unwrap();
        assert!(refresh_windows_cache(&root,&source).is_err());
        assert_eq!(std::fs::read(&external).unwrap(),b"must remain unchanged");
        std::fs::remove_dir_all(base).unwrap();
    }
    #[cfg(windows)]
    #[test]
    fn windows_cache_preserves_a_locked_destination_and_cleans_unpublished_copy(){
        use std::os::windows::fs::OpenOptionsExt;
        let base=std::env::temp_dir().join(format!("swan-os-cache-locked-{}",swan_protocol::random_token()));
        let root=base.join("installed");std::fs::create_dir_all(&root).unwrap();
        let source=base.join("RuntimeBroker.exe");let cache=root.join(WINDOWS_BROKER_CACHE);
        std::fs::write(&source,b"new system component").unwrap();std::fs::write(&cache,b"in-use old component").unwrap();
        let held=std::fs::OpenOptions::new().read(true).share_mode(1).open(&cache).unwrap();
        assert!(refresh_windows_cache(&root,&source).is_err());drop(held);
        assert_eq!(std::fs::read(&cache).unwrap(),b"in-use old component");
        assert!(!std::fs::read_dir(&root).unwrap().any(|e|e.unwrap().file_name().to_string_lossy().starts_with(".swan-windows-cache-")));
        assert!(refresh_windows_cache(&root,&source).unwrap());
        std::fs::remove_dir_all(base).unwrap();
    }
    #[test]
    fn snapshot_preserves_complete_payload_and_never_publishes_tampered_or_partial_source(){
        let root=std::env::temp_dir().join(format!("swan-snapshot-test-{}",swan_protocol::random_token()));
        let source=root.join("installed");std::fs::create_dir_all(source.join("data")).unwrap();
        let files:Vec<_>=[("endpoint.exe",b"signed executable".as_slice()),("library.dll",b"signed library".as_slice()),("data/assets.json",b"signed assets".as_slice())].into_iter().map(|(path,bytes)|{std::fs::write(source.join(path),bytes).unwrap();InstalledFile{path:path.into(),sha256:digest(bytes)}}).collect();
        let saved=root.join("rollback");snapshot(&source,&saved,&files).unwrap();
        std::fs::write(source.join("library.dll"),b"installer replaced library").unwrap();
        verify(&saved,&files).unwrap();
        assert!(snapshot(&source,&saved,&files).is_err());
        assert!(snapshot(&source,&root.join("tampered"),&files).is_err());
        assert!(!root.join("tampered").exists());
        std::fs::write(source.join("library.dll"),b"signed library").unwrap();
        std::fs::remove_file(source.join("data/assets.json")).unwrap();
        assert!(snapshot(&source,&root.join("incomplete"),&files).is_err());
        assert!(!root.join("incomplete").exists());
        let mut traversal=files.clone();traversal[0].path="../escape.exe".into();
        assert!(snapshot(&source,&root.join("unsafe"),&traversal).is_err());
        assert!(!root.join("unsafe").exists());
        assert!(!std::fs::read_dir(&root).unwrap().any(|entry|entry.unwrap().file_name().to_string_lossy().starts_with("swan-snapshot-")));
        std::fs::remove_dir_all(root).unwrap();
    }
    #[test]
    fn payload_rejects_library_asset_tampering_missing_files_and_extra_native_code(){
        let root=std::env::temp_dir().join(format!("swan-payload-{}",swan_protocol::random_token()));
        std::fs::create_dir_all(root.join("data")).unwrap();
        let files:Vec<_>=[("endpoint.exe",b"executable".as_slice()),("library.dll",b"library".as_slice()),("data/assets.json",b"assets".as_slice())].into_iter().map(|(path,bytes)|{std::fs::write(root.join(path),bytes).unwrap();InstalledFile{path:path.into(),sha256:digest(bytes)}}).collect();
        verify(&root,&files).unwrap();
        for (path,original) in [("library.dll",b"library".as_slice()),("data/assets.json",b"assets".as_slice())]{std::fs::write(root.join(path),b"tampered").unwrap();assert!(verify(&root,&files).is_err());std::fs::write(root.join(path),original).unwrap();}
        std::fs::write(root.join("injected.dll"),b"unlisted native code").unwrap();assert!(verify(&root,&files).is_err());std::fs::remove_file(root.join("injected.dll")).unwrap();
        std::fs::remove_file(root.join("library.dll")).unwrap();assert!(verify(&root,&files).is_err());
        std::fs::remove_dir_all(root).unwrap();
    }
}
