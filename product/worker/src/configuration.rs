use anyhow::{ensure,Context,Result};
use serde_json::Value;
use std::{ffi::OsStr,path::PathBuf};
use swan_protocol::{https_url,public_key};

pub struct Configuration {
    pub management_url:String,
    pub worker_token:String,
    pub release_public_key:String,
    pub profile_public_key:String,
}

impl Configuration {
    fn decode(bytes:&[u8])->Result<Self> {
        ensure!(!bytes.is_empty() && bytes.len()<=16*1024,"Invalid worker configuration size");
        let mut value:Value=serde_json::from_slice(bytes).map_err(|_|anyhow::anyhow!("Invalid worker configuration JSON"))?;
        let fields=value.as_object_mut().context("Worker configuration must be an object")?;
        ensure!(fields.len()==4 && fields.keys().all(|key|matches!(key.as_str(),"management_url"|"worker_token"|"release_public_key"|"profile_public_key")),"Worker configuration requires exactly the documented fields");
        let mut take=|name|match fields.remove(name) {Some(Value::String(value))=>Ok(value),_=>Err(anyhow::anyhow!("Worker configuration fields must be strings"))};
        let result=Self{management_url:take("management_url")?,worker_token:take("worker_token")?,release_public_key:take("release_public_key")?,profile_public_key:take("profile_public_key")?};
        result.validate()?;Ok(result)
    }
    fn validate(&self)->Result<()> {
        ensure!(self.management_url.len()<=2048,"Worker management URL is too long");
        let url=https_url(&self.management_url)?;
        ensure!(url.query().is_none() && url.fragment().is_none(),"Worker requires an HTTPS origin without query or fragment");
        ensure!((32..=256).contains(&self.worker_token.len()) && self.worker_token.bytes().all(|byte|byte.is_ascii_graphic()),"Invalid worker credential format");
        public_key(&self.release_public_key)?;public_key(&self.profile_public_key)?;
        Ok(())
    }
}

fn configuration_path(service:bool,path:Option<&OsStr>)->Result<Option<PathBuf>> {
    if let Some(path)=path {
        let path=PathBuf::from(path);ensure!(path.is_absolute(),"Private worker configuration path must be absolute");
        Ok(Some(path))
    }else{
        ensure!(!service,"Worker services require protected file configuration; revoke legacy registry credentials before upgrading");
        Ok(None)
    }
}

pub fn load(service:bool)->Result<Configuration> {
    let selected=std::env::var_os("SWAN_WORKER_CONFIG");
    if let Some(path)=configuration_path(service,selected.as_deref())? {
        #[cfg(windows)] {return Configuration::decode(&swan_agent::update::read_worker_configuration(&path)?);}
        #[cfg(not(windows))] {let _=path;anyhow::bail!("Private installer worker configuration requires Windows");}
    }
    // Foreground operator processes may supply environment credentials. SCM
    // services must never fall back to registry-persisted secret environment.
    let result=Configuration{
        management_url:std::env::var("SWAN_MANAGEMENT_URL").context("Set worker company HTTPS URL")?,
        worker_token:std::env::var("SWAN_WORKER_TOKEN").context("Set foreground worker credential")?,
        release_public_key:std::env::var("SWAN_RELEASE_PUBLIC_KEY").context("Pin project release public key")?,
        profile_public_key:std::env::var("SWAN_PROFILE_PUBLIC_KEY").context("Pin company profile public key")?,
    };
    result.validate()?;Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture()->Value {serde_json::json!({"management_url":"https://company.example.test","worker_token":"fixture-secret-worker-credential-00000000","release_public_key":"11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo=","profile_public_key":"11qYAYKxCrfVS/7TyWQHOg7hcvPapiMlrwIaaPcHURo="})}
    #[test]
    fn service_cannot_downgrade_to_registry_environment_or_relative_configuration() {
        assert!(configuration_path(true,None).is_err());
        assert!(configuration_path(true,Some(OsStr::new("relative.json"))).is_err());
        assert!(configuration_path(false,None).unwrap().is_none());
        assert!(configuration_path(true,Some(std::env::temp_dir().as_os_str())).unwrap().is_some());
    }
    #[test]
    fn private_configuration_rejects_wrong_fields_and_invalid_trust_without_echoing_secrets() {
        assert!(Configuration::decode(&serde_json::to_vec(&fixture()).unwrap()).is_ok());
        for field in ["management_url","worker_token","release_public_key","profile_public_key"] {
            let mut invalid=fixture();invalid[field]=Value::String("fixture-secret".into());
            let error=Configuration::decode(&serde_json::to_vec(&invalid).unwrap()).err().unwrap();
            assert!(!error.to_string().contains("fixture-secret"));
        }
        let mut invalid=fixture();invalid["command"]=Value::String("fixture-secret".into());
        assert!(Configuration::decode(&serde_json::to_vec(&invalid).unwrap()).is_err());
        assert!(Configuration::decode(&vec![b'x';16385]).is_err());
        assert!(Configuration::decode(b"fixture-secret").is_err());
    }
}
