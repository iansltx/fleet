//! Session types matching Go's `server/fleet/sessions.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Session represents an active user session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub created_at: DateTime<Utc>,
    pub id: u32,
    pub accessed_at: DateTime<Utc>,
    pub user_id: u32,
    pub key: String,
    #[serde(skip)]
    pub api_only: Option<bool>,
}

/// SessionSSOSettings contains SSO information used prior to authentication.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSSOSettings {
    pub idp_name: String,
    pub idp_image_url: String,
    pub sso_enabled: bool,
}

/// SSOSession holds the token and redirect URL for an SSO session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SSOSession {
    pub token: String,
    pub redirect_url: String,
}

/// SAMLAttributeValue holds the type and value of a SAML custom attribute.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SAMLAttributeValue {
    /// Type is the type of attribute value.
    #[serde(rename = "type")]
    pub value_type: String,
    /// Value is the actual value of the attribute.
    pub value: String,
}

/// SAMLAttribute holds the name and values of a SAML custom attribute.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SAMLAttribute {
    pub name: String,
    pub values: Vec<SAMLAttributeValue>,
}

/// SSORolesInfo holds the configuration parsed from SAML custom attributes.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SSORolesInfo {
    /// Global holds the role for the global domain.
    pub global: Option<String>,
    /// Teams holds the roles for teams.
    #[serde(default)]
    pub teams: Vec<crate::team::TeamRole>,
}
