//! Installation diagnostics are authenticated reports, never authorization or
//! proof of installed bytes. Signed release verification remains mandatory.
use crate::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpdatePhase { Current, Checking, Downloading, Preparing, Installing, Recovering, Deferred, Failed }

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstallationIdentity {
    pub edition: Edition,
    pub installation_id: String,
}
impl InstallationIdentity {
    pub fn validate(&self) -> Result<()> {
        let bytes = self.installation_id.as_bytes();
        ensure!(bytes.len() == 36 && bytes.iter().enumerate().all(|(index, byte)| {
            if [8, 13, 18, 23].contains(&index) { *byte == b'-' }
            else { byte.is_ascii_digit() || (b'a'..=b'f').contains(byte) }
        }), "Expected canonical installation UUID");
        Ok(())
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InstallationReport {
    pub schema: u32,
    pub identity: InstallationIdentity,
    /// Persisted by the reporter; equal revisions are idempotent, older ones fail.
    pub revision: u64,
    pub installed_sequence: u64,
    pub installed_version: Option<String>,
    pub phase: UpdatePhase,
    pub target_sequence: Option<u64>,
    pub downloaded_bytes: u64,
    pub total_bytes: Option<u64>,
}
impl InstallationReport {
    pub fn validate(&self) -> Result<()> {
        ensure!(self.schema == 1, "Unsupported installation report schema");
        self.identity.validate()?;
        ensure!(self.revision > 0 && self.revision <= i64::MAX as u64, "Invalid installation report revision");
        ensure!((self.installed_sequence == 0) == self.installed_version.is_none(), "Installed version and sequence disagree");
        if let Some(version) = &self.installed_version {
            let parts: Vec<_> = version.split('.').collect();
            ensure!(version.len() <= 32 && parts.len() == 3 && parts.iter().all(|part| !part.is_empty() && part.bytes().all(|byte| byte.is_ascii_digit())), "Invalid reported version");
        }
        ensure!(!matches!(self.phase, UpdatePhase::Current) || self.installed_sequence > 0, "Installed state requires a recorded version");
        ensure!(self.target_sequence != Some(0), "Invalid target sequence");
        ensure!(!matches!(self.phase, UpdatePhase::Downloading | UpdatePhase::Preparing | UpdatePhase::Installing | UpdatePhase::Recovering) || self.target_sequence.is_some(), "Update phase requires a target release");
        const LIMIT: u64 = 4 * 1024 * 1024 * 1024;
        ensure!(self.downloaded_bytes <= LIMIT && self.total_bytes.is_none_or(|total| total <= LIMIT && total >= self.downloaded_bytes), "Invalid download progress");
        ensure!(matches!(self.phase, UpdatePhase::Downloading | UpdatePhase::Preparing) || (self.downloaded_bytes == 0 && self.total_bytes.is_none()), "Byte progress requires a download phase");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reports_reject_unbounded_or_conflicting_status_and_active_content() {
        let report = InstallationReport { schema: 1, identity: InstallationIdentity { edition: Edition::Customer, installation_id: "a0000000-0000-4000-8000-000000000001".into() }, revision: 1, installed_sequence: 9, installed_version: Some("1.5.7".into()), phase: UpdatePhase::Downloading, target_sequence: Some(10), downloaded_bytes: 25, total_bytes: Some(100) };
        report.validate().unwrap();
        for invalid in ["../another-device", "A0000000-0000-4000-8000-000000000001", "a0000000000040008000000000000001"] {
            let mut bad = report.clone(); bad.identity.installation_id = invalid.into(); assert!(bad.validate().is_err());
        }
        let mut bad = report.clone(); bad.installed_version = Some("<script>".into()); assert!(bad.validate().is_err());
        let mut bad = report.clone(); bad.total_bytes = Some(24); assert!(bad.validate().is_err());
        let mut bad = report.clone(); bad.downloaded_bytes = u64::MAX; assert!(bad.validate().is_err());
        let mut bad = report.clone(); bad.revision = u64::MAX; assert!(bad.validate().is_err());
        let mut bad = report.clone(); bad.target_sequence = None; assert!(bad.validate().is_err());
        let mut bad = report.clone(); bad.phase = UpdatePhase::Current; assert!(bad.validate().is_err());
        let mut value = serde_json::to_value(&report).unwrap(); value["device_token"] = "never include credentials".into();
        assert!(serde_json::from_value::<InstallationReport>(value).is_err());
    }
}
