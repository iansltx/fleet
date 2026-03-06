//! Activity types matching Go's `server/fleet/activities.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// Activity represents an audit log entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Activity {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub user_id: Option<u32>,
    pub user_name: String,
    pub user_email: String,
    pub activity_type: String,
    pub details: serde_json::Value,
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
pub const ACTIVITY_APPLIED_SPEC_SAVED_QUERY: &str = "applied_spec_saved_query";
pub const ACTIVITY_CREATED_TEAM: &str = "created_team";
pub const ACTIVITY_EDITED_TEAM: &str = "edited_team";
pub const ACTIVITY_DELETED_TEAM: &str = "deleted_team";
pub const ACTIVITY_APPLIED_SPEC_TEAM: &str = "applied_spec_team";
pub const ACTIVITY_CREATED_USER: &str = "created_user";
pub const ACTIVITY_EDITED_USER: &str = "edited_user";
pub const ACTIVITY_DELETED_USER: &str = "deleted_user";
pub const ACTIVITY_CHANGED_USER_GLOBAL_ROLE: &str = "changed_user_global_role";
pub const ACTIVITY_CHANGED_USER_TEAM_ROLE: &str = "changed_user_team_role";
pub const ACTIVITY_DELETED_USER_TEAM_ROLE: &str = "deleted_user_team_role";
pub const ACTIVITY_LIVE_QUERY: &str = "live_query";
pub const ACTIVITY_EDITED_AGENT_OPTIONS: &str = "edited_agent_options";
pub const ACTIVITY_RAN_SCRIPT: &str = "ran_script";
pub const ACTIVITY_ADDED_SCRIPT: &str = "added_script";
pub const ACTIVITY_DELETED_SCRIPT: &str = "deleted_script";
pub const ACTIVITY_EDITED_SCRIPT: &str = "edited_script";
pub const ACTIVITY_ENABLED_DISK_ENCRYPTION: &str = "enabled_disk_encryption";
pub const ACTIVITY_DISABLED_DISK_ENCRYPTION: &str = "disabled_disk_encryption";
pub const ACTIVITY_TRANSFERRED_HOSTS: &str = "transferred_hosts";
pub const ACTIVITY_LOCKED_HOST: &str = "locked_host";
pub const ACTIVITY_UNLOCKED_HOST: &str = "unlocked_host";
pub const ACTIVITY_WIPED_HOST: &str = "wiped_host";
pub const ACTIVITY_EDITED_MACOS_MIN_VERSION: &str = "edited_macos_min_version";
pub const ACTIVITY_EDITED_WINDOWS_UPDATES: &str = "edited_windows_updates";
pub const ACTIVITY_READ_HOST_DISK_ENCRYPTION_KEY: &str = "read_host_disk_encryption_key";
pub const ACTIVITY_ENABLED_MACOS_DISK_ENCRYPTION: &str = "enabled_macos_disk_encryption";
pub const ACTIVITY_DISABLED_MACOS_DISK_ENCRYPTION: &str = "disabled_macos_disk_encryption";
pub const ACTIVITY_ADDED_BOOTSTRAP_PACKAGE: &str = "added_bootstrap_package";
pub const ACTIVITY_DELETED_BOOTSTRAP_PACKAGE: &str = "deleted_bootstrap_package";
pub const ACTIVITY_ENABLED_MACOS_SETUP_END_USER_AUTH: &str = "enabled_macos_setup_end_user_auth";
pub const ACTIVITY_DISABLED_MACOS_SETUP_END_USER_AUTH: &str = "disabled_macos_setup_end_user_auth";
pub const ACTIVITY_INSTALLED_SOFTWARE: &str = "installed_software";
pub const ACTIVITY_UNINSTALLED_SOFTWARE: &str = "uninstalled_software";
pub const ACTIVITY_ADDED_SOFTWARE: &str = "added_software";
pub const ACTIVITY_DELETED_SOFTWARE: &str = "deleted_software";
pub const ACTIVITY_EDITED_SOFTWARE: &str = "edited_software";
pub const ACTIVITY_RESENT_CONFIGURATION_PROFILE: &str = "resent_configuration_profile";
pub const ACTIVITY_ENABLED_VPP: &str = "enabled_vpp";
pub const ACTIVITY_DISABLED_VPP: &str = "disabled_vpp";

/// ActivityListOptions configures activity listing.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ActivityListOptions {
    #[serde(flatten)]
    pub list_options: crate::ListOptions,
    pub streamed: bool,
}

/// UpcomingActivity represents a pending/scheduled activity for a host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpcomingActivity {
    pub id: u32,
    pub host_id: u32,
    pub user_id: Option<u32>,
    pub activity_type: String,
    pub execution_id: String,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub fleet_initiated: bool,
    pub priority: i32,
}
