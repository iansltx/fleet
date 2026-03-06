//! Certificate types matching Go's `server/fleet/certificate_authorities.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

// ─── CA Config Asset Types (legacy) ─────────────────────────────────────────

/// CAConfigAssetType represents the type of a CA configuration asset.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CAConfigAssetType {
    #[serde(rename = "ndes")]
    NDES,
    #[serde(rename = "digicert")]
    DigiCert,
    #[serde(rename = "custom_scep_proxy")]
    CustomSCEPProxy,
    #[serde(rename = "smallstep")]
    Smallstep,
}

/// CAConfigAsset represents a CA configuration asset stored in the database.
#[derive(Debug, Clone)]
pub struct CAConfigAsset {
    pub name: String,
    pub value: Vec<u8>,
    pub asset_type: CAConfigAssetType,
}

// ─── CA Type ─────────────────────────────────────────────────────────────────

/// CAType represents a certificate authority type.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum CAType {
    #[serde(rename = "ndes_scep_proxy")]
    NDESSCEPProxy,
    #[serde(rename = "digicert")]
    DigiCert,
    #[serde(rename = "custom_scep_proxy")]
    CustomSCEPProxy,
    #[serde(rename = "hydrant")]
    Hydrant,
    #[serde(rename = "custom_est_proxy")]
    CustomESTProxy,
    #[serde(rename = "smallstep")]
    Smallstep,
}

// ─── Certificate Authority ──────────────────────────────────────────────────

/// CertificateAuthoritySummary is a lightweight summary of a CA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateAuthoritySummary {
    pub id: u32,
    pub name: String,
    #[serde(rename = "type")]
    pub ca_type: String,
}

/// CertificateAuthority represents a certificate authority configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CertificateAuthority {
    pub id: u32,
    #[serde(rename = "type")]
    pub ca_type: String,

    // common fields
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,

    // DigiCert
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_common_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_user_principal_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_seat_id: Option<String>,

    // NDES SCEP Proxy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_url: Option<String>,

    // Smallstep
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_url: Option<String>,

    // Shared: Smallstep, NDES, EST
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,

    // Custom SCEP Proxy
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge: Option<String>,

    // Hydrant
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,

    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// CertificateAuthorityPayload is the payload for creating certificate authorities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateAuthorityPayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digicert: Option<DigiCertCA>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndes_scep_proxy: Option<NDESSCEPProxyCA>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_scep_proxy: Option<CustomSCEPProxyCA>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hydrant: Option<HydrantCA>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_est_proxy: Option<ESTProxyCA>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smallstep: Option<SmallstepSCEPProxyCA>,
}

// ─── Individual CA Types ─────────────────────────────────────────────────────

/// DigiCertCA represents a DigiCert certificate authority configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigiCertCA {
    #[serde(skip)]
    pub id: u32,
    pub name: String,
    pub url: String,
    pub api_token: String,
    pub profile_id: String,
    pub certificate_common_name: String,
    #[serde(default)]
    pub certificate_user_principal_names: Vec<String>,
    pub certificate_seat_id: String,
}

/// ESTProxyCA is the Enrollment over Secure Transport Certificate Authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ESTProxyCA {
    #[serde(skip)]
    pub id: u32,
    pub name: String,
    pub url: String,
    pub username: String,
    pub password: String,
}

/// HydrantCA represents a Hydrant certificate authority configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrantCA {
    #[serde(skip)]
    pub id: u32,
    pub name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub client_id: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub client_secret: String,
}

/// NDESSCEPProxyCA configures SCEP proxy for NDES SCEP server.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NDESSCEPProxyCA {
    #[serde(skip)]
    pub id: u32,
    pub url: String,
    pub admin_url: String,
    pub username: String,
    pub password: String,
}

/// CustomSCEPProxyCA represents a custom SCEP proxy certificate authority.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSCEPProxyCA {
    #[serde(skip)]
    pub id: u32,
    pub name: String,
    pub url: String,
    pub challenge: String,
}

/// SmallstepSCEPProxyCA represents a Smallstep SCEP proxy configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallstepSCEPProxyCA {
    #[serde(skip)]
    pub id: u32,
    pub name: String,
    pub url: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub challenge_url: String,
    pub username: String,
    pub password: String,
}

// ─── Update Payloads ─────────────────────────────────────────────────────────

/// CertificateAuthorityUpdatePayload is the payload for updating certificate authorities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateAuthorityUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub digicert: Option<DigiCertCAUpdatePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndes_scep_proxy: Option<NDESSCEPProxyCAUpdatePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_scep_proxy: Option<CustomSCEPProxyCAUpdatePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub hydrant: Option<HydrantCAUpdatePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub custom_est_proxy: Option<CustomESTCAUpdatePayload>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub smallstep: Option<SmallstepSCEPProxyCAUpdatePayload>,
}

/// DigiCertCAUpdatePayload is the update payload for a DigiCert CA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DigiCertCAUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub api_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_common_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_user_principal_names: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub certificate_seat_id: Option<String>,
}

/// NDESSCEPProxyCAUpdatePayload is the update payload for an NDES SCEP proxy CA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NDESSCEPProxyCAUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub admin_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// CustomSCEPProxyCAUpdatePayload is the update payload for a custom SCEP proxy CA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomSCEPProxyCAUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge: Option<String>,
}

/// HydrantCAUpdatePayload is the update payload for a Hydrant CA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HydrantCAUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub client_secret: Option<String>,
}

/// CustomESTCAUpdatePayload is the update payload for a custom EST CA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomESTCAUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

/// SmallstepSCEPProxyCAUpdatePayload is the update payload for a Smallstep SCEP proxy CA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallstepSCEPProxyCAUpdatePayload {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub challenge_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub username: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
}

// ─── Grouped Certificate Authorities ─────────────────────────────────────────

/// GroupedCertificateAuthorities groups CAs by type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupedCertificateAuthorities {
    pub custom_est_proxy: Vec<ESTProxyCA>,
    pub hydrant: Vec<HydrantCA>,
    pub digicert: Vec<DigiCertCA>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ndes_scep_proxy: Option<NDESSCEPProxyCA>,
    pub custom_scep_proxy: Vec<CustomSCEPProxyCA>,
    pub smallstep: Vec<SmallstepSCEPProxyCA>,
}

/// CertificateAuthoritiesBatchOperations groups the operations for batch processing of CAs.
#[derive(Debug, Clone, Default)]
pub struct CertificateAuthoritiesBatchOperations {
    pub delete: Vec<CertificateAuthority>,
    pub add: Vec<CertificateAuthority>,
    pub update: Vec<CertificateAuthority>,
}

// ─── Request Certificate ─────────────────────────────────────────────────────

/// RequestCertificatePayload is the payload for requesting a certificate from a CA.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RequestCertificatePayload {
    pub id: u32,
    pub csr: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idp_oauth_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idp_token: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub idp_client_id: Option<String>,
}

// ─── Smallstep Challenge Types ───────────────────────────────────────────────

/// SmallstepChallengeRequestBody represents the request body for obtaining a challenge from Smallstep.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallstepChallengeRequestBody {
    pub webhook: SmallstepChallengeWebhook,
    pub event: SmallstepChallengeEvent,
}

/// SmallstepChallengeWebhook represents webhook info in the Smallstep challenge request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallstepChallengeWebhook {
    #[serde(rename = "webhookEvent")]
    pub webhook_event: String,
    pub id: i32,
    #[serde(rename = "eventTimestamp")]
    pub event_timestamp: i64,
    pub name: String,
}

/// SmallstepChallengeEvent represents event info in the Smallstep challenge request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmallstepChallengeEvent {
    #[serde(rename = "scepServerUrl")]
    pub scep_server_url: String,
    #[serde(rename = "payloadIdentifier")]
    pub payload_identifier: String,
    #[serde(rename = "payloadTypes")]
    pub payload_types: Vec<String>,
}

// ─── Host Certificate ────────────────────────────────────────────────────────

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

/// CertificateTemplate represents a certificate template configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificateTemplate {
    pub id: u32,
    pub team_id: u32,
    pub certificate_authority_id: i32,
    pub name: String,
    pub subject_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
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
