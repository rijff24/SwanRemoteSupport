//! Verify the signed installed payload, including native libraries and assets.
use anyhow::{ensure,Context,Result};
use std::path::Path;
use swan_protocol::{digest,InstalledFile};

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
