//! Certificate types matching Go's `server/fleet/certificates.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// HostCertificate represents a TLS certificate installed on a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostCertificate {
    pub id: u64,
    pub host_id: u32,
    pub not_valid_after: DateTime<Utc>,
    pub not_valid_before: DateTime<Utc>,
    pub certificate_authority: bool,
    pub common_name: String,
    pub key_algorithm: String,
    pub key_strength: i32,
    pub key_usage: String,
    pub serial: String,
    pub signing_algorithm: String,
    pub subject_country: String,
    pub subject_org: String,
    pub subject_org_unit: String,
    pub subject_common_name: String,
}

/// SetupExperienceScript represents a setup experience script configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SetupExperienceScript {
    pub id: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
