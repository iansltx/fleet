//! Activity types matching Go's `server/fleet/activities.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Activity represents an audit log entry.
/// Matches Go's `api.Activity` type.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    #[serde(default, skip_serializing_if = "is_zero_u32")]
    pub id: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uuid: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(rename = "type")]
    pub activity_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_full_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_gravatar: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub actor_api_only: Option<bool>,
    #[serde(skip)]
    pub streamed: Option<bool>,
    #[serde(default)]
    pub fleet_initiated: bool,
    pub details: Option<serde_json::Value>,
}

fn is_zero_u32(v: &u32) -> bool {
    *v == 0
}

/// UpcomingActivity is the augmented activity type for upcoming (pending) activities.
/// Matches Go's `fleet.UpcomingActivity`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingActivity {
    #[serde(flatten)]
    pub activity: Activity,
}

/// WellKnownActionType defines the special actions that an upcoming activity
/// may correspond to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(i32)]
pub enum WellKnownActionType {
    None = 0,
    Lock = 1,
    Unlock = 2,
    Wipe = 3,
}

impl Default for WellKnownActionType {
    fn default() -> Self {
        WellKnownActionType::None
    }
}

/// UpcomingActivityMeta is the metadata related to a host's upcoming activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingActivityMeta {
    pub execution_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activated_at: Option<DateTime<Utc>>,
    pub upcoming_activity_type: String,
    #[serde(default)]
    pub well_known_action: WellKnownActionType,
}

/// ActivityListOptions configures activity listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivityListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
    pub streamed: bool,
}

// ---------------------------------------------------------------------------
// Activity type name constants (matching Go's ActivityType*.ActivityName())
// ---------------------------------------------------------------------------
pub const ACTIVITY_CREATED_PACK: &str = "created_pack";
pub const ACTIVITY_EDITED_PACK: &str = "edited_pack";
pub const ACTIVITY_DELETED_PACK: &str = "deleted_pack";
pub const ACTIVITY_APPLIED_SPEC_PACK: &str = "applied_spec_pack";
pub const ACTIVITY_CREATED_POLICY: &str = "created_policy";
pub const ACTIVITY_EDITED_POLICY: &str = "edited_policy";
pub const ACTIVITY_DELETED_POLICY: &str = "deleted_policy";
pub const ACTIVITY_APPLIED_SPEC_POLICY: &str = "applied_spec_policy";
pub const ACTIVITY_CREATED_SAVED_QUERY: &str = "created_saved_query";
pub const ACTIVITY_EDITED_SAVED_QUERY: &str = "edited_saved_query";
pub const ACTIVITY_DELETED_SAVED_QUERY: &str = "deleted_saved_query";
pub const ACTIVITY_DELETED_MULTIPLE_SAVED_QUERY: &str = "deleted_multiple_saved_query";
pub const ACTIVITY_APPLIED_SPEC_SAVED_QUERY: &str = "applied_spec_saved_query";
pub const ACTIVITY_CREATED_TEAM: &str = "created_team";
pub const ACTIVITY_EDITED_TEAM: &str = "edited_team";
pub const ACTIVITY_DELETED_TEAM: &str = "deleted_team";
pub const ACTIVITY_APPLIED_SPEC_TEAM: &str = "applied_spec_team";
pub const ACTIVITY_TRANSFERRED_HOSTS: &str = "transferred_hosts";
pub const ACTIVITY_EDITED_AGENT_OPTIONS: &str = "edited_agent_options";
pub const ACTIVITY_LIVE_QUERY: &str = "live_query";
pub const ACTIVITY_USER_ADDED_BY_SSO: &str = "user_added_by_sso";
pub const ACTIVITY_USER_LOGGED_IN: &str = "user_logged_in";
pub const ACTIVITY_USER_FAILED_LOGIN: &str = "user_failed_login";
pub const ACTIVITY_CREATED_USER: &str = "created_user";
pub const ACTIVITY_DELETED_USER: &str = "deleted_user";
pub const ACTIVITY_DELETED_HOST: &str = "deleted_host";
pub const ACTIVITY_CHANGED_USER_GLOBAL_ROLE: &str = "changed_user_global_role";
pub const ACTIVITY_DELETED_USER_GLOBAL_ROLE: &str = "deleted_user_global_role";
pub const ACTIVITY_CHANGED_USER_TEAM_ROLE: &str = "changed_user_team_role";
pub const ACTIVITY_DELETED_USER_TEAM_ROLE: &str = "deleted_user_team_role";
pub const ACTIVITY_FLEET_ENROLLED: &str = "fleet_enrolled";
pub const ACTIVITY_MDM_ENROLLED: &str = "mdm_enrolled";
pub const ACTIVITY_MDM_UNENROLLED: &str = "mdm_unenrolled";
pub const ACTIVITY_EDITED_MACOS_MIN_VERSION: &str = "edited_macos_min_version";
pub const ACTIVITY_EDITED_IOS_MIN_VERSION: &str = "edited_ios_min_version";
pub const ACTIVITY_EDITED_IPADOS_MIN_VERSION: &str = "edited_ipados_min_version";
pub const ACTIVITY_EDITED_WINDOWS_UPDATES: &str = "edited_windows_updates";
pub const ACTIVITY_ENABLED_MACOS_UPDATE_NEW_HOSTS: &str = "enabled_macos_update_new_hosts";
pub const ACTIVITY_DISABLED_MACOS_UPDATE_NEW_HOSTS: &str = "disabled_macos_update_new_hosts";
pub const ACTIVITY_READ_HOST_DISK_ENCRYPTION_KEY: &str = "read_host_disk_encryption_key";
pub const ACTIVITY_CREATED_MACOS_PROFILE: &str = "created_macos_profile";
pub const ACTIVITY_DELETED_MACOS_PROFILE: &str = "deleted_macos_profile";
pub const ACTIVITY_EDITED_MACOS_PROFILE: &str = "edited_macos_profile";
pub const ACTIVITY_CHANGED_MACOS_SETUP_ASSISTANT: &str = "changed_macos_setup_assistant";
pub const ACTIVITY_DELETED_MACOS_SETUP_ASSISTANT: &str = "deleted_macos_setup_assistant";
pub const ACTIVITY_ENABLED_MACOS_DISK_ENCRYPTION: &str = "enabled_macos_disk_encryption";
pub const ACTIVITY_DISABLED_MACOS_DISK_ENCRYPTION: &str = "disabled_macos_disk_encryption";
pub const ACTIVITY_ENABLED_RECOVERY_LOCK_PASSWORD: &str = "enabled_recovery_lock_password";
pub const ACTIVITY_DISABLED_RECOVERY_LOCK_PASSWORD: &str = "disabled_recovery_lock_password";
pub const ACTIVITY_ENABLED_GITOPS_MODE: &str = "enabled_gitops_mode";
pub const ACTIVITY_DISABLED_GITOPS_MODE: &str = "disabled_gitops_mode";
pub const ACTIVITY_ADDED_BOOTSTRAP_PACKAGE: &str = "added_bootstrap_package";
pub const ACTIVITY_DELETED_BOOTSTRAP_PACKAGE: &str = "deleted_bootstrap_package";
pub const ACTIVITY_ENABLED_MACOS_SETUP_END_USER_AUTH: &str = "enabled_macos_setup_end_user_auth";
pub const ACTIVITY_DISABLED_MACOS_SETUP_END_USER_AUTH: &str = "disabled_macos_setup_end_user_auth";
pub const ACTIVITY_ENABLED_WINDOWS_MDM: &str = "enabled_windows_mdm";
pub const ACTIVITY_DISABLED_WINDOWS_MDM: &str = "disabled_windows_mdm";
pub const ACTIVITY_ENABLED_ANDROID_MDM: &str = "enabled_android_mdm";
pub const ACTIVITY_DISABLED_ANDROID_MDM: &str = "disabled_android_mdm";
pub const ACTIVITY_ENABLED_WINDOWS_MDM_MIGRATION: &str = "enabled_windows_mdm_migration";
pub const ACTIVITY_DISABLED_WINDOWS_MDM_MIGRATION: &str = "disabled_windows_mdm_migration";
pub const ACTIVITY_RAN_SCRIPT: &str = "ran_script";
pub const ACTIVITY_ADDED_SCRIPT: &str = "added_script";
pub const ACTIVITY_DELETED_SCRIPT: &str = "deleted_script";
pub const ACTIVITY_EDITED_SCRIPT: &str = "edited_script";
pub const ACTIVITY_UPDATED_SCRIPT: &str = "updated_script";
pub const ACTIVITY_CREATED_WINDOWS_PROFILE: &str = "created_windows_profile";
pub const ACTIVITY_DELETED_WINDOWS_PROFILE: &str = "deleted_windows_profile";
pub const ACTIVITY_EDITED_WINDOWS_PROFILE: &str = "edited_windows_profile";
pub const ACTIVITY_LOCKED_HOST: &str = "locked_host";
pub const ACTIVITY_UNLOCKED_HOST: &str = "unlocked_host";
pub const ACTIVITY_WIPED_HOST: &str = "wiped_host";
pub const ACTIVITY_CREATED_DECLARATION_PROFILE: &str = "created_declaration_profile";
pub const ACTIVITY_DELETED_DECLARATION_PROFILE: &str = "deleted_declaration_profile";
pub const ACTIVITY_EDITED_DECLARATION_PROFILE: &str = "edited_declaration_profile";
pub const ACTIVITY_CREATED_ANDROID_PROFILE: &str = "created_android_profile";
pub const ACTIVITY_DELETED_ANDROID_PROFILE: &str = "deleted_android_profile";
pub const ACTIVITY_EDITED_ANDROID_PROFILE: &str = "edited_android_profile";
pub const ACTIVITY_EDITED_ANDROID_CERTIFICATE: &str = "edited_android_certificate";
pub const ACTIVITY_RESENT_CONFIGURATION_PROFILE: &str = "resent_configuration_profile";
pub const ACTIVITY_RESENT_CONFIGURATION_PROFILE_BATCH: &str = "resent_configuration_profile_batch";
pub const ACTIVITY_INSTALLED_SOFTWARE: &str = "installed_software";
pub const ACTIVITY_UNINSTALLED_SOFTWARE: &str = "uninstalled_software";
pub const ACTIVITY_ADDED_SOFTWARE: &str = "added_software";
pub const ACTIVITY_EDITED_SOFTWARE: &str = "edited_software";
pub const ACTIVITY_DELETED_SOFTWARE: &str = "deleted_software";
pub const ACTIVITY_ENABLED_VPP: &str = "enabled_vpp";
pub const ACTIVITY_DISABLED_VPP: &str = "disabled_vpp";
pub const ACTIVITY_ADDED_APP_STORE_APP: &str = "added_app_store_app";
pub const ACTIVITY_DELETED_APP_STORE_APP: &str = "deleted_app_store_app";
pub const ACTIVITY_INSTALLED_APP_STORE_APP: &str = "installed_app_store_app";
pub const ACTIVITY_EDITED_APP_STORE_APP: &str = "edited_app_store_app";
pub const ACTIVITY_ADDED_NDES_SCEP_PROXY: &str = "added_ndes_scep_proxy";
pub const ACTIVITY_DELETED_NDES_SCEP_PROXY: &str = "deleted_ndes_scep_proxy";
pub const ACTIVITY_EDITED_NDES_SCEP_PROXY: &str = "edited_ndes_scep_proxy";
pub const ACTIVITY_ADDED_CUSTOM_SCEP_PROXY: &str = "added_custom_scep_proxy";
pub const ACTIVITY_DELETED_CUSTOM_SCEP_PROXY: &str = "deleted_custom_scep_proxy";
pub const ACTIVITY_EDITED_CUSTOM_SCEP_PROXY: &str = "edited_custom_scep_proxy";
pub const ACTIVITY_ADDED_DIGICERT: &str = "added_digicert";
pub const ACTIVITY_DELETED_DIGICERT: &str = "deleted_digicert";
pub const ACTIVITY_EDITED_DIGICERT: &str = "edited_digicert";
pub const ACTIVITY_ADDED_HYDRANT: &str = "added_hydrant";
pub const ACTIVITY_DELETED_HYDRANT: &str = "deleted_hydrant";
pub const ACTIVITY_EDITED_HYDRANT: &str = "edited_hydrant";
pub const ACTIVITY_ADDED_CUSTOM_EST_PROXY: &str = "added_custom_est_proxy";
pub const ACTIVITY_DELETED_CUSTOM_EST_PROXY: &str = "deleted_custom_est_proxy";
pub const ACTIVITY_EDITED_CUSTOM_EST_PROXY: &str = "edited_custom_est_proxy";
pub const ACTIVITY_ADDED_SMALLSTEP: &str = "added_smallstep";
pub const ACTIVITY_DELETED_SMALLSTEP: &str = "deleted_smallstep";
pub const ACTIVITY_EDITED_SMALLSTEP: &str = "edited_smallstep";
pub const ACTIVITY_ENABLED_ACTIVITY_AUTOMATIONS: &str = "enabled_activity_automations";
pub const ACTIVITY_EDITED_ACTIVITY_AUTOMATIONS: &str = "edited_activity_automations";
pub const ACTIVITY_DISABLED_ACTIVITY_AUTOMATIONS: &str = "disabled_activity_automations";
pub const ACTIVITY_CANCELED_RUN_SCRIPT: &str = "canceled_run_script";
pub const ACTIVITY_CANCELED_INSTALL_SOFTWARE: &str = "canceled_install_software";
pub const ACTIVITY_CANCELED_UNINSTALL_SOFTWARE: &str = "canceled_uninstall_software";
pub const ACTIVITY_CANCELED_INSTALL_APP_STORE_APP: &str = "canceled_install_app_store_app";
pub const ACTIVITY_RAN_SCRIPT_BATCH: &str = "ran_script_batch";
pub const ACTIVITY_SCHEDULED_SCRIPT_BATCH: &str = "scheduled_script_batch";
pub const ACTIVITY_CANCELED_SCRIPT_BATCH: &str = "canceled_script_batch";
pub const ACTIVITY_ADDED_CONDITIONAL_ACCESS_INTEGRATION_MICROSOFT: &str = "added_conditional_access_integration_microsoft";
pub const ACTIVITY_DELETED_CONDITIONAL_ACCESS_INTEGRATION_MICROSOFT: &str = "deleted_conditional_access_integration_microsoft";
pub const ACTIVITY_ADDED_CONDITIONAL_ACCESS_OKTA: &str = "added_conditional_access_okta";
pub const ACTIVITY_DELETED_CONDITIONAL_ACCESS_OKTA: &str = "deleted_conditional_access_okta";
pub const ACTIVITY_ENABLED_CONDITIONAL_ACCESS_AUTOMATIONS: &str = "enabled_conditional_access_automations";
pub const ACTIVITY_DISABLED_CONDITIONAL_ACCESS_AUTOMATIONS: &str = "disabled_conditional_access_automations";
pub const ACTIVITY_UPDATE_CONDITIONAL_ACCESS_BYPASS: &str = "update_conditional_access_bypass";
pub const ACTIVITY_HOST_BYPASSED_CONDITIONAL_ACCESS: &str = "host_bypassed_conditional_access";
pub const ACTIVITY_ESCROWED_DISK_ENCRYPTION_KEY: &str = "escrowed_disk_encryption_key";
pub const ACTIVITY_CREATED_CUSTOM_VARIABLE: &str = "created_custom_variable";
pub const ACTIVITY_DELETED_CUSTOM_VARIABLE: &str = "deleted_custom_variable";
pub const ACTIVITY_EDITED_SETUP_EXPERIENCE_SOFTWARE: &str = "edited_setup_experience_software";
pub const ACTIVITY_EDITED_HOST_IDP_DATA: &str = "edited_host_idp_data";
pub const ACTIVITY_EDITED_ENROLL_SECRETS: &str = "edited_enroll_secrets";
pub const ACTIVITY_ADDED_CERTIFICATE: &str = "added_certificate";
pub const ACTIVITY_DELETED_CERTIFICATE: &str = "deleted_certificate";
pub const ACTIVITY_ADDED_MICROSOFT_ENTRA_TENANT: &str = "added_microsoft_entra_tenant";
pub const ACTIVITY_DELETED_MICROSOFT_ENTRA_TENANT: &str = "deleted_microsoft_entra_tenant";

// ---------------------------------------------------------------------------
// Activity detail structs (matching Go's ActivityType* structs)
// ---------------------------------------------------------------------------

/// Details for enabled/edited activity automations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEnabledActivityAutomations {
    pub webhook_url: String,
}

/// Details for edited activity automations.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedActivityAutomations {
    pub webhook_url: String,
}

/// Details for created_pack activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailCreatedPack {
    pub pack_id: u32,
    pub pack_name: String,
}

/// Details for edited_pack activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedPack {
    pub pack_id: u32,
    pub pack_name: String,
}

/// Details for deleted_pack activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedPack {
    pub pack_name: String,
}

/// Details for created_policy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailCreatedPolicy {
    pub policy_id: u32,
    pub policy_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for edited_policy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedPolicy {
    pub policy_id: u32,
    pub policy_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for deleted_policy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedPolicy {
    pub policy_id: u32,
    pub policy_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for applied_spec_policy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAppliedSpecPolicy {
    pub policies: Vec<crate::policy::PolicySpec>,
}

/// Details for created_saved_query activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailCreatedSavedQuery {
    pub query_id: u32,
    pub query_name: String,
    pub team_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for edited_saved_query activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedSavedQuery {
    pub query_id: u32,
    pub query_name: String,
    pub team_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for deleted_saved_query activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedSavedQuery {
    pub query_name: String,
    pub team_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for deleted_multiple_saved_query activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedMultipleSavedQuery {
    pub query_ids: Vec<u32>,
    pub team_id: i64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for applied_spec_saved_query activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAppliedSpecSavedQuery {
    pub specs: Vec<crate::query::QuerySpec>,
}

/// Details for created_team activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailCreatedTeam {
    pub team_id: u32,
    pub team_name: String,
}

/// Details for deleted_team activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedTeam {
    pub team_id: u32,
    pub team_name: String,
}

/// TeamActivityDetail is used in applied_spec_team activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamActivityDetail {
    pub id: u32,
    pub name: String,
}

/// Details for applied_spec_team activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAppliedSpecTeam {
    pub teams: Vec<TeamActivityDetail>,
}

/// Details for transferred_hosts activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailTransferredHosts {
    pub team_id: Option<u32>,
    pub team_name: Option<String>,
    #[serde(default)]
    pub host_ids: Vec<u32>,
    #[serde(default)]
    pub host_display_names: Vec<String>,
}

/// Details for edited_agent_options activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedAgentOptions {
    pub global: bool,
    pub team_id: Option<u32>,
    pub team_name: Option<String>,
}

/// Details for live_query activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailLiveQuery {
    pub targets_count: u32,
    pub query_sql: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub query_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stats: Option<crate::query::AggregatedStats>,
}

/// Details for user_logged_in activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailUserLoggedIn {
    pub public_ip: String,
}

/// Details for user_failed_login activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailUserFailedLogin {
    pub email: String,
    pub public_ip: String,
}

/// Details for created_user activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailCreatedUser {
    pub user_id: u32,
    pub user_name: String,
    pub user_email: String,
}

/// Details for deleted_user activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedUser {
    pub user_id: u32,
    pub user_name: String,
    pub user_email: String,
}

/// DeletedHostTriggeredBy defines what triggered a host deletion.
pub const DELETED_HOST_TRIGGERED_BY_MANUAL: &str = "manual";
pub const DELETED_HOST_TRIGGERED_BY_EXPIRATION: &str = "expiration";

/// Details for deleted_host activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedHost {
    pub host_id: u32,
    pub host_display_name: String,
    pub host_serial: String,
    pub triggered_by: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_expiry_window: Option<i32>,
}

/// Details for changed_user_global_role activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailChangedUserGlobalRole {
    pub user_id: u32,
    pub user_name: String,
    pub user_email: String,
    pub role: String,
}

/// Details for deleted_user_global_role activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedUserGlobalRole {
    pub user_id: u32,
    pub user_name: String,
    pub user_email: String,
    pub role: String,
}

/// Details for changed_user_team_role activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailChangedUserTeamRole {
    pub user_id: u32,
    pub user_name: String,
    pub user_email: String,
    pub role: String,
    pub team_id: u32,
    pub team_name: String,
}

/// Details for deleted_user_team_role activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedUserTeamRole {
    pub user_id: u32,
    pub user_name: String,
    pub user_email: String,
    pub role: String,
    pub team_id: u32,
    pub team_name: String,
}

/// Details for fleet_enrolled activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailFleetEnrolled {
    pub host_id: u32,
    pub host_serial: String,
    pub host_display_name: String,
}

/// Details for mdm_enrolled activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailMDMEnrolled {
    pub host_serial: Option<String>,
    pub host_display_name: String,
    pub installed_from_dep: bool,
    pub mdm_platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment_id: Option<String>,
    pub platform: String,
}

/// Details for mdm_unenrolled activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailMDMUnenrolled {
    pub host_serial: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub enrollment_id: Option<String>,
    pub host_display_name: String,
    pub installed_from_dep: bool,
    pub platform: String,
}

/// Details for edited_macos_min_version activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedMacOSMinVersion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    pub minimum_version: String,
    pub deadline: String,
}

/// Details for edited_ios_min_version activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedIOSMinVersion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    pub minimum_version: String,
    pub deadline: String,
}

/// Details for edited_ipados_min_version activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedIPadOSMinVersion {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    pub minimum_version: String,
    pub deadline: String,
}

/// Details for edited_windows_updates activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedWindowsUpdates {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    pub deadline_days: Option<i32>,
    pub grace_period_days: Option<i32>,
}

/// Details for enabled/disabled_macos_update_new_hosts activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailMacosUpdateNewHosts {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for read_host_disk_encryption_key activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailReadHostDiskEncryptionKey {
    pub host_id: u32,
    pub host_display_name: String,
}

/// Details for created_macos_profile activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailCreatedMacosProfile {
    pub profile_name: String,
    pub profile_identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for deleted_macos_profile activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedMacosProfile {
    pub profile_name: String,
    pub profile_identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for edited_macos_profile activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedMacosProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for changed_macos_setup_assistant activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailChangedMacosSetupAssistant {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for deleted_macos_setup_assistant activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedMacosSetupAssistant {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for team_id/team_name only activities (shared by many activity types).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailTeamIdName {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for added/deleted_bootstrap_package activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailBootstrapPackage {
    pub bootstrap_package_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for ran_script activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailRanScript {
    pub host_id: u32,
    pub host_display_name: String,
    pub script_execution_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub batch_execution_id: Option<String>,
    pub script_name: String,
    pub r#async: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_name: Option<String>,
}

/// Details for added/deleted/updated script activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailScript {
    pub script_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for created/deleted windows profile activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailWindowsProfile {
    pub profile_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for locked_host activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailLockedHost {
    pub host_id: u32,
    pub host_display_name: String,
    pub view_pin: bool,
}

/// Details for unlocked_host activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailUnlockedHost {
    pub host_id: u32,
    pub host_display_name: String,
    pub host_platform: String,
}

/// Details for wiped_host activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailWipedHost {
    pub host_id: u32,
    pub host_display_name: String,
}

/// Details for created/deleted declaration profile activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeclarationProfile {
    pub profile_name: String,
    pub identifier: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for resent_configuration_profile activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailResentConfigurationProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_display_name: Option<String>,
    pub profile_name: String,
}

/// Details for resent_configuration_profile_batch activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailResentConfigurationProfileBatch {
    pub profile_name: String,
    pub host_count: i64,
}

/// Details for installed_software activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailInstalledSoftware {
    pub host_id: u32,
    pub host_display_name: String,
    pub software_title: String,
    pub software_package: String,
    pub self_service: bool,
    pub install_uuid: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_name: Option<String>,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub command_uuid: String,
}

/// Details for uninstalled_software activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailUninstalledSoftware {
    pub host_id: u32,
    pub host_display_name: String,
    pub software_title: String,
    pub script_execution_id: String,
    pub self_service: bool,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

/// ActivitySoftwareLabel holds label info for software-related activities.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivitySoftwareLabel {
    pub name: String,
    pub id: u32,
}

/// Details for added_software activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAddedSoftware {
    pub software_title: String,
    pub software_package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    pub self_service: bool,
    pub software_title_id: u32,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<ActivitySoftwareLabel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_exclude_any: Vec<ActivitySoftwareLabel>,
}

/// Details for edited_software activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedSoftware {
    pub software_title: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_package: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    pub self_service: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<ActivitySoftwareLabel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_exclude_any: Vec<ActivitySoftwareLabel>,
    pub software_title_id: u32,
    pub software_display_name: String,
}

/// Details for deleted_software activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedSoftware {
    pub software_title: String,
    pub software_package: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    pub self_service: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<ActivitySoftwareLabel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_exclude_any: Vec<ActivitySoftwareLabel>,
}

/// Details for enabled_vpp / disabled_vpp activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailVPP {
    pub location: String,
}

/// Details for added_app_store_app activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAddedAppStoreApp {
    pub software_title: String,
    pub software_title_id: u32,
    pub app_store_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    pub platform: String,
    pub self_service: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<ActivitySoftwareLabel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_exclude_any: Vec<ActivitySoftwareLabel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<serde_json::Value>,
}

/// Details for deleted_app_store_app activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedAppStoreApp {
    pub software_title: String,
    pub app_store_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<ActivitySoftwareLabel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_exclude_any: Vec<ActivitySoftwareLabel>,
}

/// Details for installed_app_store_app activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailInstalledAppStoreApp {
    pub host_id: u32,
    pub host_display_name: String,
    pub software_title: String,
    pub app_store_id: String,
    pub command_uuid: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub status: String,
    pub self_service: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_name: Option<String>,
    pub host_platform: String,
    pub from_auto_update: bool,
}

/// Details for edited_app_store_app activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedAppStoreApp {
    pub software_title: String,
    pub software_title_id: u32,
    pub app_store_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    pub platform: String,
    pub self_service: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_icon_url: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<ActivitySoftwareLabel>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_exclude_any: Vec<ActivitySoftwareLabel>,
    pub software_display_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub configuration: Option<serde_json::Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update_window_start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_update_window_end: Option<String>,
}

/// Details for named proxy/cert activities (SCEP, DigiCert, Hydrant, EST, Smallstep).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailNameOnly {
    pub name: String,
}

/// Details for canceled_run_script activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailCanceledRunScript {
    pub host_id: u32,
    pub host_display_name: String,
    pub script_name: String,
}

/// Details for canceled_install_software / canceled_uninstall_software / canceled_install_app_store_app.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailCanceledSoftware {
    pub host_id: u32,
    pub host_display_name: String,
    pub software_title: String,
    pub software_title_id: u32,
}

/// Details for ran_script_batch activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailRanScriptBatch {
    pub script_name: String,
    pub batch_execution_id: String,
    pub host_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
}

/// Details for scheduled_script_batch activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailBatchScriptScheduled {
    pub batch_execution_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_name: Option<String>,
    pub host_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_before: Option<DateTime<Utc>>,
}

/// Details for canceled_script_batch activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailBatchScriptCanceled {
    pub batch_execution_id: String,
    pub script_name: String,
    pub host_count: u32,
    pub canceled_count: u32,
}

/// Details for enabled/disabled_conditional_access_automations activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailConditionalAccessAutomations {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    pub team_name: String,
}

/// Details for update_conditional_access_bypass activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailUpdateConditionalAccessBypass {
    pub bypass_disabled: bool,
}

/// Details for host_bypassed_conditional_access activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailHostBypassedConditionalAccess {
    pub host_id: u32,
    pub host_display_name: String,
    pub idp_full_name: String,
}

/// Details for escrowed_disk_encryption_key activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEscrowedDiskEncryptionKey {
    pub host_id: u32,
    pub host_display_name: String,
}

/// Details for created/deleted custom variable activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailCustomVariable {
    pub custom_variable_id: u32,
    pub custom_variable_name: String,
}

/// Details for edited_setup_experience_software activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedSetupExperienceSoftware {
    pub platform: String,
    pub team_id: u32,
    pub team_name: String,
}

/// Details for created/deleted android profile activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAndroidProfile {
    pub profile_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for edited_host_idp_data activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedHostIdpData {
    pub host_id: u32,
    pub host_display_name: String,
    pub host_idp_username: String,
}

/// Details for added/deleted certificate activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailCertificate {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for added/deleted microsoft entra tenant activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailMicrosoftEntraTenant {
    pub tenant_id: String,
}

/// Details for edited_enroll_secrets activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedEnrollSecrets {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

// ---------------------------------------------------------------------------
// Additional activity detail structs (Go types with team_id/team_name fields)
// ---------------------------------------------------------------------------

/// Details for enabled_macos_disk_encryption activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEnabledMacosDiskEncryption {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for disabled_macos_disk_encryption activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDisabledMacosDiskEncryption {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for enabled_recovery_lock_password activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEnabledRecoveryLockPassword {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for disabled_recovery_lock_password activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDisabledRecoveryLockPassword {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for enabled_macos_setup_end_user_auth activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEnabledMacosSetupEndUserAuth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for disabled_macos_setup_end_user_auth activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDisabledMacosSetupEndUserAuth {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for edited_script activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedScript {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for edited_windows_profile activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedWindowsProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for edited_declaration_profile activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedDeclarationProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for edited_android_profile activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedAndroidProfile {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

/// Details for edited_android_certificate activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedAndroidCertificate {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_name: Option<String>,
}

// --- Empty activity detail structs (no fields) ---

/// Details for disabled_activity_automations activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDisabledActivityAutomations {}

/// Details for applied_spec_pack activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAppliedSpecPack {}

/// Details for user_added_by_sso activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailUserAddedBySSO {}

/// Details for enabled_gitops_mode activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEnabledGitOpsMode {}

/// Details for disabled_gitops_mode activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDisabledGitOpsMode {}

/// Details for enabled_windows_mdm activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEnabledWindowsMDM {}

/// Details for disabled_windows_mdm activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDisabledWindowsMDM {}

/// Details for enabled_windows_mdm_migration activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEnabledWindowsMDMMigration {}

/// Details for disabled_windows_mdm_migration activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDisabledWindowsMDMMigration {}

/// Details for added_ndes_scep_proxy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAddedNDESSCEPProxy {}

/// Details for deleted_ndes_scep_proxy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedNDESSCEPProxy {}

/// Details for edited_ndes_scep_proxy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedNDESSCEPProxy {}

/// Details for enabled_android_mdm activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEnabledAndroidMDM {}

/// Details for disabled_android_mdm activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDisabledAndroidMDM {}

/// Details for added_conditional_access_integration_microsoft activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAddedConditionalAccessIntegrationMicrosoft {}

/// Details for deleted_conditional_access_integration_microsoft activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedConditionalAccessIntegrationMicrosoft {}

/// Details for added_conditional_access_okta activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAddedConditionalAccessOkta {}

/// Details for deleted_conditional_access_okta activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedConditionalAccessOkta {}

// --- Activity detail structs with a single `name` field ---

/// Details for added_custom_scep_proxy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAddedCustomSCEPProxy {
    pub name: String,
}

/// Details for deleted_custom_scep_proxy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedCustomSCEPProxy {
    pub name: String,
}

/// Details for edited_custom_scep_proxy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedCustomSCEPProxy {
    pub name: String,
}

/// Details for added_digicert activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAddedDigiCert {
    pub name: String,
}

/// Details for deleted_digicert activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedDigiCert {
    pub name: String,
}

/// Details for edited_digicert activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedDigiCert {
    pub name: String,
}

/// Details for added_hydrant activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAddedHydrant {
    pub name: String,
}

/// Details for deleted_hydrant activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedHydrant {
    pub name: String,
}

/// Details for edited_hydrant activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedHydrant {
    pub name: String,
}

/// Details for added_custom_est_proxy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAddedCustomESTProxy {
    pub name: String,
}

/// Details for deleted_custom_est_proxy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedCustomESTProxy {
    pub name: String,
}

/// Details for edited_custom_est_proxy activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedCustomESTProxy {
    pub name: String,
}

/// Details for added_smallstep activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailAddedSmallstep {
    pub name: String,
}

/// Details for deleted_smallstep activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailDeletedSmallstep {
    pub name: String,
}

/// Details for edited_smallstep activity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActivityDetailEditedSmallstep {
    pub name: String,
}
