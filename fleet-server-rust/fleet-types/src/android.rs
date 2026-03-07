//! Android types matching Go's `server/fleet/android.go`.

use std::collections::HashMap;
use std::sync::LazyLock;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::mdm::{ConfigurationProfileLabel, MDMDeliveryStatus, MDMOperationType};

// ─── Constants ───────────────────────────────────────────────────────────────

pub const ANDROID_WEB_APP_PREFIX: &str = "com.google.enterprise.webapp";

// ─── Forbidden / Premium-only JSON keys ──────────────────────────────────────

/// AndroidForbiddenJSONKeys are keys that may not be included in user-provided Android
/// configuration profiles and associated error messages.
pub static ANDROID_FORBIDDEN_JSON_KEYS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert("statusReportingSettings", "Android configuration profile can't include \"statusReportingSettings\" setting. To get host vitals, use Get host endpoint: https://fleetdm.com/docs/rest-api/rest-api#get-host");
        m.insert("applications", "Android configuration profile can't include \"applications\" setting. Software management is coming soon.");
        m.insert("appFunctions", "Android configuration profile can't include \"appFunctions\" setting. Software management is coming soon.");
        m.insert("playStoreMode", "Android configuration profile can't include \"playStoreMode\" setting. Software management is coming soon.");
        m.insert("installAppsDisabled", "Android configuration profile can't include \"installAppsDisabled\" setting. Software management is coming soon.");
        m.insert("uninstallAppsDisabled", "Android configuration profile can't include \"uninstallAppsDisabled\" setting. Software management is coming soon.");
        m.insert("blockApplicationsEnabled", "Android configuration profile can't include \"blockApplicationsEnabled\" setting. Software management is coming soon.");
        m.insert("appAutoUpdatePolicy", "Android configuration profile can't include \"appAutoUpdatePolicy\" setting. Software management is coming soon.");
        m.insert("kioskCustomLauncherEnabled", "Android configuration profile can't include \"kioskCustomLauncherEnabled\" setting. Currently, only personal hosts are supported.");
        m.insert("kioskCustomization", "Android configuration profile can't include \"kioskCustomization\" setting. Currently, only personal hosts are supported.");
        m.insert("persistentPreferredActivities", "Android configuration profile can't include \"persistentPreferredActivities\" setting. Currently, only personal hosts are supported.");
        m.insert("setupActions", "Android configuration profile can't include \"setupActions\" setting. Currently, setup experience customization isn't supported.");
        m.insert("encryptionPolicy", "Android configuration profile can't include \"encryptionPolicy\" setting. Currently, disk encryption isn't supported.");
        m
    });

/// AndroidPremiumOnlyJSONKeys are keys that may not be included in user-provided Android
/// configuration profiles for non-Premium licenses.
pub static ANDROID_PREMIUM_ONLY_JSON_KEYS: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| {
        let mut m = HashMap::new();
        m.insert(
            "systemUpdate",
            "Android OS updates (\"systemUpdate\") is Fleet Premium only.",
        );
        m
    });

// ─── MDMAndroidConfigProfile ─────────────────────────────────────────────────

/// MDMAndroidConfigProfile represents an Android MDM profile in Fleet.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAndroidConfigProfile {
    pub profile_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    pub name: String,
    #[serde(skip)]
    pub raw_json: Vec<u8>,
    pub auto_increment: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_all: Vec<ConfigurationProfileLabel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<ConfigurationProfileLabel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_exclude_any: Vec<ConfigurationProfileLabel>,
    pub created_at: DateTime<Utc>,
    #[serde(rename = "updated_at")]
    pub uploaded_at: DateTime<Utc>,
}

// ─── MDMAndroidProfilePayload ────────────────────────────────────────────────

/// MDMAndroidProfilePayload represents the payload for an Android MDM profile operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MDMAndroidProfilePayload {
    pub host_uuid: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MDMDeliveryStatus>,
    pub operation_type: MDMOperationType,
    #[serde(default)]
    pub detail: String,
    pub profile_uuid: String,
    pub profile_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_request_uuid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_request_uuid: Option<String>,
    #[serde(default)]
    pub request_fail_count: i32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub included_in_policy_version: Option<i32>,
    #[serde(default)]
    pub last_error_details: String,
    #[serde(default)]
    pub can_reverify: bool,
}

// ─── HostMDMAndroidProfile ───────────────────────────────────────────────────

/// HostMDMAndroidProfile represents the status of an MDM profile for an Android host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostMDMAndroidProfile {
    pub host_uuid: String,
    pub profile_uuid: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<MDMDeliveryStatus>,
    pub operation_type: MDMOperationType,
    #[serde(default)]
    pub detail: String,
}

// ─── AndroidPolicyRequestPayload ─────────────────────────────────────────────

/// AndroidPolicyRequestPayload represents a request to apply an Android policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidPolicyRequestPayload {
    pub policy: serde_json::Value,
    pub metadata: AndroidPolicyRequestPayloadMetadata,
}

/// AndroidPolicyRequestPayloadMetadata contains metadata for an Android policy request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AndroidPolicyRequestPayloadMetadata {
    /// Map of policy setting name to profile UUID.
    #[serde(default)]
    pub settings_origin: HashMap<String, String>,
}
