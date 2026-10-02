//! Verify the signed installed payload, including native libraries and assets.
use anyhow::{ensure,Context,Result};
use std::path::Path;
use swan_protocol::{digest,InstalledFile};

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
