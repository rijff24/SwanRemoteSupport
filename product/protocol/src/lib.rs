//! Signed company configuration and authorization. Sign the serialized payload bytes,
//! not a re-serialization, so verifiers never depend on JSON object ordering.
use anyhow::{ensure, Context, Result};
use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha2::{Digest, Sha256};

pub const SCHEMA: u32 = 1;
pub const PRODUCT: &str = "Swan Remote Support";
pub const WIRE_PREFIX: &[u8] = b"SWAN1:";

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum Edition { Customer, Technician }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Branding {
    pub display_name: String,
    pub primary_color: String,
    pub logo_svg: String,
    pub support_url: String,
    pub consent_text: String,
}
impl Default for Branding {
    fn default() -> Self {
        Self { display_name: PRODUCT.into(), primary_color: "#004B6E".into(),
            logo_svg: include_str!("../../../branding/swan-support-mark.svg").into(),
            support_url: String::new(),
            consent_text: "Only allow remote support from a technician you trust. You can stop support or revoke ongoing access at any time.".into() }
    }
}
impl Branding {
    pub fn validate(&self) -> Result<()> {
        ensure!(!self.display_name.trim().is_empty() && self.display_name.len() <= 100, "Invalid display name");
        ensure!(self.primary_color.len() == 7 && self.primary_color.starts_with('#') && self.primary_color[1..].bytes().all(|b| b.is_ascii_hexdigit()), "Invalid color");
        ensure!(self.consent_text.len() <= 4000 && !self.consent_text.trim().is_empty(), "Invalid consent text");
        ensure!(self.logo_svg.len() <= 128 * 1024, "Logo too large");
        // SVG is displayed as an image, never injected as HTML. Reject active/external content as defense in depth.
        let svg = self.logo_svg.to_ascii_lowercase();
        for forbidden in ["<script", "<foreignobject", "<!entity", "<!doctype", "javascript:", "href=", "href =", "url(", "onload", "onerror", "<image", "<use", "<style"] {
            ensure!(!svg.contains(forbidden), "Unsupported SVG content");
        }
        if !self.support_url.is_empty() { https_url(&self.support_url)?; }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CompanyProfile {
    pub schema: u32,
    pub company_id: String,
    pub revision: u64,
    pub issued_at: u64,
    pub expires_at: u64,
    pub management_url: String,
    pub rendezvous: String,
    pub relay: String,
    pub transport_public_key: String,
    pub customer: Branding,
    pub technician: Branding,
    pub allow_unattended: bool,
    pub updates_paused: bool,
    pub rollout_percent: u8,
    pub maintenance_start_utc: u8,
    pub maintenance_end_utc: u8,
    pub update_channel: String,
    pub next_profile_public_key: Option<String>,
}
impl CompanyProfile {
    pub fn validate(&self, company: &str, minimum_revision: u64, now: u64) -> Result<()> {
        ensure!(self.schema == SCHEMA && self.company_id == company, "Wrong company or schema");
        ensure!(self.revision >= minimum_revision && self.revision > 0, "Replayed profile");
        ensure!(self.issued_at <= now + 60 && self.expires_at > now && self.expires_at > self.issued_at, "Expired or future profile");
        https_url(&self.management_url)?;
        validate_host(&self.rendezvous)?;
        validate_host(&self.relay)?;
        public_key(&self.transport_public_key)?;
        self.customer.validate()?;
        self.technician.validate()?;
        ensure!(self.rollout_percent <= 100 && self.maintenance_start_utc <= 23 && self.maintenance_end_utc <= 23, "Invalid update policy");
        ensure!(["stable", "test"].contains(&self.update_channel.as_str()), "Invalid channel");
        if let Some(key) = &self.next_profile_public_key { public_key(key)?; }
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SignedEnvelope { pub payload: String, pub signature: String }
impl SignedEnvelope {
    pub fn sign<T: Serialize>(value: &T, key: &SigningKey) -> Result<Self> {
        let bytes = serde_json::to_vec(value)?;
        Ok(Self { payload: STANDARD.encode(&bytes), signature: STANDARD.encode(key.sign(&bytes).to_bytes()) })
    }
    pub fn verify<T: DeserializeOwned>(&self, key: &VerifyingKey) -> Result<T> {
        ensure!(self.payload.len() <= 1024 * 1024, "Payload too large");
        let bytes = STANDARD.decode(&self.payload)?;
        let signature = Signature::from_slice(&STANDARD.decode(&self.signature)?)?;
        key.verify_strict(&bytes, &signature).context("Invalid signature")?;
        Ok(serde_json::from_slice(&bytes)?)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionPermissions {
    pub keyboard:bool,pub clipboard:bool,pub audio:bool,pub file:bool,
    pub restart:bool,pub recording:bool,pub block_input:bool,pub privacy_mode:bool,
}
impl SessionPermissions {
    pub fn support_default()->Self {Self{keyboard:true,clipboard:true,audio:true,file:true,restart:true,recording:false,block_input:false,privacy_mode:false}}
    pub fn allows(&self,requested:&Self)->bool {
        (!requested.keyboard || self.keyboard) && (!requested.clipboard || self.clipboard) &&
        (!requested.audio || self.audio) && (!requested.file || self.file) &&
        (!requested.restart || self.restart) && (!requested.recording || self.recording) &&
        (!requested.block_input || self.block_input) && (!requested.privacy_mode || self.privacy_mode)
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SessionGrant {
    pub schema: u32,
    pub company_id: String,
    pub grant_id: String,
    pub technician_id: String,
    pub device_id: String,
    pub rustdesk_id: String,
    pub proof_public_key: String,
    pub unattended: bool,
    pub permissions:SessionPermissions,
    pub issued_at: u64,
    pub expires_at: u64,
}
impl SessionGrant {
    pub fn validate(&self, company: &str, device: &str, peer: &str, now: u64) -> Result<()> {
        ensure!(self.schema == SCHEMA && self.company_id == company && self.device_id == device && self.rustdesk_id == peer, "Wrong grant target");
        ensure!(self.issued_at <= now + 30 && self.expires_at > now && self.expires_at <= self.issued_at + 60, "Grant expired or lifetime invalid");
        public_key(&self.proof_public_key)?;
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ManagedLogin { pub grant: SignedEnvelope, pub challenge_signature: String }
impl ManagedLogin {
    pub fn to_wire(&self) -> Result<Vec<u8>> {
        let mut wire = WIRE_PREFIX.to_vec();
        wire.extend(serde_json::to_vec(self)?);
        Ok(wire)
    }
    pub fn from_wire(wire: &[u8]) -> Result<Self> {
        ensure!(wire.len() <= 16384, "Login payload too large");
        ensure!(wire.starts_with(WIRE_PREFIX), "Managed grant required");
        Ok(serde_json::from_slice(&wire[WIRE_PREFIX.len()..])?)
    }
    pub fn prove(grant: SignedEnvelope, key: &SigningKey, challenge: &str) -> Self {
        Self { grant, challenge_signature: STANDARD.encode(key.sign(challenge.as_bytes()).to_bytes()) }
    }
    pub fn verify_proof(&self, grant: &SessionGrant, challenge: &str) -> Result<()> {
        public_key(&grant.proof_public_key)?.verify_strict(challenge.as_bytes(), &Signature::from_slice(&STANDARD.decode(&self.challenge_signature)?)?).context("Invalid session proof")
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Release {
    pub schema: u32,
    pub product: String,
    pub version: String,
    pub sequence: u64,
    pub edition: Edition,
    pub architecture: String,
    pub channel: String,
    pub expires_at: u64,
    pub artifact_url: String,
    pub sha256: String,
    pub installed_sha256: String,
    pub agent_url: String,
    pub agent_sha256: String,
    pub publisher: String,
    pub publisher_certificate_sha256: String,
    pub windows_versions: Vec<String>,
    pub source_url: String,
    pub format: String,
}
impl Release {
    pub fn validate(&self, edition: &Edition, minimum_sequence: u64, now: u64) -> Result<()> {
        ensure!(self.schema == SCHEMA && self.product == PRODUCT && &self.edition == edition, "Wrong product or edition");
        ensure!(self.architecture == "x64" && self.sequence > minimum_sequence && self.expires_at > now, "Incompatible, expired or replayed release");
        ensure!(["stable", "test"].contains(&self.channel.as_str()) && ["exe", "msi"].contains(&self.format.as_str()), "Invalid release type");
        ensure!(self.sha256.len() == 64 && self.sha256.bytes().all(|x| x.is_ascii_hexdigit()) && !self.publisher.is_empty(), "Invalid artifact identity");
        ensure!(self.installed_sha256.len()==64 && self.installed_sha256.bytes().all(|x|x.is_ascii_hexdigit()),"Invalid installed executable identity");
        ensure!(self.publisher_certificate_sha256.len()==64 && self.publisher_certificate_sha256.bytes().all(|x|x.is_ascii_hexdigit()),"Invalid publisher certificate identity");
        ensure!(!self.windows_versions.is_empty() && self.windows_versions.len()<=6 && self.windows_versions.iter().all(|v|WINDOWS_VERSIONS.contains(&v.as_str())),"Invalid Windows compatibility declaration");
        let fields: Vec<_> = self.version.split('.').collect();
        ensure!(fields.len() == 3 && fields.iter().all(|v| !v.is_empty() && v.bytes().all(|c| c.is_ascii_digit())), "Expected numeric major.minor.patch");
        https_url(&self.artifact_url)?;
        https_url(&self.agent_url)?;
        ensure!(self.agent_sha256.len() == 64 && self.agent_sha256.bytes().all(|x| x.is_ascii_hexdigit()), "Invalid agent hash");
        https_url(&self.source_url)?;
        Ok(())
    }
}

pub const WINDOWS_VERSIONS:&[&str]=&["windows_10","windows_11","server_2016","server_2019","server_2022","server_2025"];

pub fn https_url(value: &str) -> Result<url::Url> {
    let url = url::Url::parse(value)?;
    ensure!(url.scheme() == "https" && url.host_str().is_some() && url.username().is_empty() && url.password().is_none() && url.fragment().is_none(), "Expected HTTPS URL without credentials or fragment");
    Ok(url)
}
pub fn validate_host(value: &str) -> Result<()> {
    ensure!(!value.is_empty() && value.len() <= 253, "Invalid transport endpoint");
    ensure!(value.bytes().all(|c| c.is_ascii_alphanumeric() || b".-:[]".contains(&c)), "Invalid transport address");
    let endpoint=url::Url::parse(&format!("tcp://{value}"))?;
    ensure!(endpoint.host_str().is_some() && endpoint.username().is_empty() && endpoint.password().is_none() && endpoint.query().is_none() && endpoint.fragment().is_none() && ["","/"].contains(&endpoint.path()),"Expected transport hostname and optional port");
    Ok(())
}
pub fn public_key(value: &str) -> Result<VerifyingKey> {
    let bytes: [u8; 32] = STANDARD.decode(value)?.try_into().map_err(|_| anyhow::anyhow!("Expected 32-byte key"))?;
    Ok(VerifyingKey::from_bytes(&bytes)?)
}
pub fn random_token() -> String {
    use rand::RngCore;
    let mut bytes = [0u8; 32]; rand::rngs::OsRng.fill_bytes(&mut bytes); hex::encode(bytes)
}
pub fn signing_key_from_hex(value:&str)->Result<SigningKey> {
    let bytes:[u8;32]=hex::decode(value)?.try_into().map_err(|_|anyhow::anyhow!("Expected 32-byte signing key"))?;
    Ok(SigningKey::from_bytes(&bytes))
}
pub fn digest(bytes: impl AsRef<[u8]>) -> String { hex::encode(Sha256::digest(bytes.as_ref())) }
pub fn bundle_text(bytes:&[u8])->Result<Vec<u8>> {
    // Windows and Linux source checkouts must produce identical bundled scripts.
    Ok(std::str::from_utf8(bytes)?.replace("\r\n","\n").into_bytes())
}
pub fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0) }
pub fn maintenance_open(start: u8, end: u8, now: u64) -> bool {
    let hour = ((now / 3600) % 24) as u8;
    if start == end { true } else if start < end { hour >= start && hour < end } else { hour >= start || hour < end }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn signed_payload_rejects_tampering_and_wrong_key() {
        let key = SigningKey::from_bytes(&[7;32]);
        let mut envelope = SignedEnvelope::sign(&"company", &key).unwrap();
        assert_eq!(envelope.verify::<String>(&key.verifying_key()).unwrap(), "company");
        assert!(envelope.verify::<String>(&SigningKey::from_bytes(&[8;32]).verifying_key()).is_err());
        envelope.payload = STANDARD.encode(b"\"attacker\"");
        assert!(envelope.verify::<String>(&key.verifying_key()).is_err());
    }
    #[test]
    fn grant_is_target_bound_expiring_and_requires_challenge_proof() {
        let key = SigningKey::from_bytes(&[3;32]);
        let grant = SessionGrant { schema: 1, company_id: "a".into(), grant_id: "jti".into(), technician_id:"t".into(), device_id:"d".into(), rustdesk_id:"123".into(), proof_public_key: STANDARD.encode(key.verifying_key().as_bytes()), unattended:false, permissions:SessionPermissions::support_default(), issued_at:100, expires_at:160 };
        assert!(grant.validate("a","d","123",101).is_ok());
        assert!(grant.validate("b","d","123",101).is_err());
        assert!(grant.validate("a","d","124",101).is_err());
        assert!(grant.validate("a","d","123",160).is_err());
        let login = ManagedLogin::prove(SignedEnvelope::sign(&grant,&key).unwrap(), &key,"nonce");
        assert!(login.verify_proof(&grant,"nonce").is_ok());
        assert!(login.verify_proof(&grant,"different nonce").is_err());
        assert!(ManagedLogin::from_wire(b"legacy-password").is_err());
    }
    #[test]
    fn maintenance_windows_wrap_midnight() {
        assert!(maintenance_open(22,3,23*3600)); assert!(maintenance_open(22,3,3600));
        assert!(!maintenance_open(22,3,12*3600)); assert!(maintenance_open(0,0,12*3600));
    }
    #[test]
    fn releases_reject_replay_wrong_edition_expiry_and_missing_installed_identity() {
        let mut release=Release {schema:1,product:PRODUCT.into(),version:"1.5.0".into(),sequence:2,edition:Edition::Customer,architecture:"x64".into(),channel:"stable".into(),expires_at:200,artifact_url:"https://releases.example/package.exe".into(),sha256:"a".repeat(64),installed_sha256:"b".repeat(64),agent_url:"https://releases.example/agent.exe".into(),agent_sha256:"c".repeat(64),publisher:"Example".into(),publisher_certificate_sha256:"d".repeat(64),windows_versions:vec!["windows_11".into()],source_url:"https://releases.example/source.tar.gz".into(),format:"exe".into()};
        assert!(release.validate(&Edition::Customer,1,100).is_ok());
        assert!(release.validate(&Edition::Customer,2,100).is_err());
        assert!(release.validate(&Edition::Technician,0,100).is_err());
        assert!(release.validate(&Edition::Customer,0,200).is_err());
        release.windows_versions=vec!["server_core".into()];assert!(release.validate(&Edition::Customer,0,100).is_err());
        release.windows_versions=vec!["windows_11".into()];release.publisher_certificate_sha256="publisher label".into();assert!(release.validate(&Edition::Customer,0,100).is_err());
        release.publisher_certificate_sha256="d".repeat(64);release.installed_sha256.clear();assert!(release.validate(&Edition::Customer,0,100).is_err());
    }
}
