//! Main FleetService struct and constructor.
//!
//! This is the primary service struct that holds references to the datastore,
//! Redis, configuration, and other dependencies. It corresponds to Go's
//! `server/service/service.go` Service struct.

use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

use crate::{FleetServiceConfig, ServiceResult};

// ---------------------------------------------------------------------------
// Datastore trait -- abstracts database access (MySQL or other backends).
// This matches the Go `fleet.Datastore` interface.
// ---------------------------------------------------------------------------

#[async_trait]
pub trait Datastore: Send + Sync + 'static {
    // ---- Users ----
    async fn user_by_id(&self, id: u32) -> ServiceResult<fleet_types::User>;
    async fn user_by_email(&self, email: &str) -> ServiceResult<fleet_types::User>;
    async fn list_users(&self, opts: fleet_types::user::UserListOptions) -> ServiceResult<Vec<fleet_types::User>>;
    async fn save_user(&self, user: &fleet_types::User) -> ServiceResult<fleet_types::User>;
    async fn new_user(&self, user: &fleet_types::User) -> ServiceResult<fleet_types::User>;
    async fn delete_user(&self, id: u32) -> ServiceResult<()>;
    async fn has_users(&self) -> ServiceResult<bool>;
    async fn count_global_admins(&self) -> ServiceResult<i64>;
    async fn user_settings(&self, user_id: u32) -> ServiceResult<Option<serde_json::Value>>;
    async fn save_user_settings(&self, user_id: u32, settings: &serde_json::Value) -> ServiceResult<()>;

    // ---- Sessions ----
    async fn new_session(&self, user_id: u32, key_size: usize) -> ServiceResult<fleet_types::Session>;
    async fn session_by_id(&self, id: u32) -> ServiceResult<fleet_types::Session>;
    async fn session_by_key(&self, key: &str) -> ServiceResult<fleet_types::Session>;
    async fn mark_session_accessed(&self, session: &fleet_types::Session) -> ServiceResult<()>;
    async fn destroy_session(&self, session: &fleet_types::Session) -> ServiceResult<()>;
    async fn destroy_all_sessions_for_user(&self, user_id: u32) -> ServiceResult<()>;
    async fn list_sessions_for_user(&self, user_id: u32) -> ServiceResult<Vec<fleet_types::Session>>;

    // ---- Hosts ----
    async fn host(&self, id: u32) -> ServiceResult<fleet_types::Host>;
    async fn host_by_identifier(&self, identifier: &str) -> ServiceResult<fleet_types::Host>;
    async fn list_hosts(&self, opts: fleet_types::HostListOptions) -> ServiceResult<Vec<fleet_types::Host>>;
    async fn delete_host(&self, id: u32) -> ServiceResult<()>;
    async fn host_lite(&self, id: u32) -> ServiceResult<fleet_types::Host>;
    async fn load_host_by_node_key(&self, node_key: &str) -> ServiceResult<fleet_types::Host>;
    async fn mark_host_seen(&self, host_id: u32, seen_at: DateTime<Utc>) -> ServiceResult<()>;
    async fn enroll_host(
        &self,
        host_identifier: &str,
        node_key: &str,
        team_id: Option<u32>,
        hardware_uuid: &str,
        hardware_serial: &str,
    ) -> ServiceResult<fleet_types::Host>;
    async fn host_summary(&self) -> ServiceResult<fleet_types::HostSummary>;
    async fn transfer_hosts_to_team(&self, host_ids: &[u32], team_id: Option<u32>) -> ServiceResult<()>;
    async fn list_os_versions(&self) -> ServiceResult<Vec<fleet_types::OSVersionStats>>;
    async fn os_version(&self, id: u32) -> ServiceResult<fleet_types::OSVersionStats>;
    async fn search_hosts(&self, query: &str, omit_ids: &[u32], limit: u32) -> ServiceResult<Vec<fleet_types::Host>>;

    // ---- Queries ----
    async fn query(&self, id: u32) -> ServiceResult<fleet_types::Query>;
    async fn list_queries(
        &self,
        opts: fleet_types::ListOptions,
        team_id: Option<u32>,
    ) -> ServiceResult<Vec<fleet_types::Query>>;
    async fn new_query(&self, query: &fleet_types::Query) -> ServiceResult<fleet_types::Query>;
    async fn save_query(&self, query: &fleet_types::Query) -> ServiceResult<fleet_types::Query>;
    async fn delete_query(&self, name: &str, team_id: Option<u32>) -> ServiceResult<()>;
    async fn delete_queries(&self, ids: &[u32]) -> ServiceResult<u32>;
    async fn query_by_name(&self, team_id: Option<u32>, name: &str) -> ServiceResult<fleet_types::Query>;
    async fn query_result_rows(&self, query_id: u32) -> ServiceResult<Vec<fleet_types::QueryResultRow>>;

    // ---- Packs ----
    async fn pack(&self, id: u32) -> ServiceResult<fleet_types::Pack>;
    async fn pack_by_name(&self, name: &str) -> ServiceResult<Option<fleet_types::Pack>>;
    async fn list_packs(&self, opts: fleet_types::ListOptions) -> ServiceResult<Vec<fleet_types::Pack>>;
    async fn new_pack(&self, pack: &fleet_types::Pack) -> ServiceResult<fleet_types::Pack>;
    async fn save_pack(&self, pack: &fleet_types::Pack) -> ServiceResult<()>;
    async fn delete_pack(&self, name: &str) -> ServiceResult<()>;

    // ---- Scheduled Queries ----
    async fn scheduled_query(&self, id: u32) -> ServiceResult<fleet_types::ScheduledQuery>;
    async fn list_scheduled_queries_in_pack(&self, pack_id: u32) -> ServiceResult<Vec<fleet_types::ScheduledQuery>>;
    async fn new_scheduled_query(&self, sq: &fleet_types::ScheduledQuery) -> ServiceResult<fleet_types::ScheduledQuery>;
    async fn save_scheduled_query(&self, sq: &fleet_types::ScheduledQuery) -> ServiceResult<fleet_types::ScheduledQuery>;
    async fn delete_scheduled_query(&self, id: u32) -> ServiceResult<()>;
    async fn ensure_global_pack(&self) -> ServiceResult<u32>;
    async fn ensure_team_pack(&self, team_id: u32) -> ServiceResult<u32>;

    // ---- Labels ----
    async fn label(&self, id: u32) -> ServiceResult<fleet_types::Label>;
    async fn label_by_name(&self, name: &str) -> ServiceResult<fleet_types::Label>;
    async fn list_labels(&self, opts: fleet_types::ListOptions) -> ServiceResult<Vec<fleet_types::Label>>;
    async fn new_label(&self, label: &fleet_types::Label) -> ServiceResult<fleet_types::Label>;
    async fn save_label(&self, label: &fleet_types::Label) -> ServiceResult<fleet_types::Label>;
    async fn delete_label(&self, name: &str) -> ServiceResult<()>;
    async fn labels_summary(&self) -> ServiceResult<Vec<fleet_types::LabelSummary>>;
    async fn record_label_membership(&self, label_id: u32, host_id: u32) -> ServiceResult<()>;
    async fn delete_label_membership(&self, label_id: u32, host_id: u32) -> ServiceResult<()>;
    async fn list_labels_for_host(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::Label>>;

    // ---- Policies ----
    async fn policy(&self, id: u32) -> ServiceResult<fleet_types::Policy>;
    async fn list_global_policies(&self, opts: fleet_types::ListOptions) -> ServiceResult<Vec<fleet_types::Policy>>;
    async fn new_global_policy(&self, query: &str, name: &str, description: &str) -> ServiceResult<fleet_types::Policy>;
    async fn save_policy(&self, policy: &fleet_types::Policy) -> ServiceResult<fleet_types::Policy>;
    async fn delete_global_policies(&self, ids: &[u32]) -> ServiceResult<Vec<u32>>;
    async fn list_team_policies(&self, team_id: u32, opts: fleet_types::ListOptions) -> ServiceResult<Vec<fleet_types::Policy>>;
    async fn new_team_policy(&self, team_id: u32, query: &str, name: &str, description: &str) -> ServiceResult<fleet_types::Policy>;
    async fn delete_team_policies(&self, team_id: u32, ids: &[u32]) -> ServiceResult<Vec<u32>>;

    // ---- Teams ----
    async fn team(&self, id: u32) -> ServiceResult<fleet_types::Team>;
    async fn list_teams(&self, opts: fleet_types::ListOptions) -> ServiceResult<Vec<fleet_types::Team>>;
    async fn new_team(&self, team: &fleet_types::Team) -> ServiceResult<fleet_types::Team>;
    async fn save_team(&self, team: &fleet_types::Team) -> ServiceResult<fleet_types::Team>;
    async fn delete_team(&self, id: u32) -> ServiceResult<()>;
    async fn teams_summary(&self) -> ServiceResult<Vec<TeamSummaryInfo>>;
    async fn list_team_users(&self, team_id: u32) -> ServiceResult<Vec<fleet_types::team::TeamUser>>;
    async fn team_enroll_secrets(&self, team_id: u32) -> ServiceResult<Vec<fleet_types::enroll::EnrollSecret>>;
    async fn apply_team_enroll_secrets(&self, team_id: u32, secrets: &[String]) -> ServiceResult<()>;
    async fn add_users_to_team(&self, team_id: u32, users: &[(u32, String)]) -> ServiceResult<()>;
    async fn remove_users_from_team(&self, team_id: u32, user_ids: &[u32]) -> ServiceResult<()>;

    // ---- AppConfig ----
    async fn app_config(&self) -> ServiceResult<AppConfigData>;
    async fn save_app_config(&self, config: &AppConfigData) -> ServiceResult<()>;

    // ---- Secret Variables ----
    async fn list_secret_variables(&self) -> ServiceResult<Vec<fleet_types::config::SecretVariable>>;
    async fn create_secret_variable(&self, name: &str, value: &str) -> ServiceResult<fleet_types::config::SecretVariable>;
    async fn delete_secret_variable(&self, id: u32) -> ServiceResult<()>;
    async fn upsert_secret_variables(&self, secrets: &[(String, String)]) -> ServiceResult<()>;

    // ---- Invites ----
    async fn invite_by_email(&self, email: &str) -> ServiceResult<Option<InviteData>>;
    async fn invite_by_token(&self, token: &str) -> ServiceResult<InviteData>;
    async fn new_invite(&self, invite: &InviteData) -> ServiceResult<InviteData>;
    async fn list_invites(&self, opts: fleet_types::ListOptions) -> ServiceResult<Vec<InviteData>>;
    async fn delete_invite(&self, id: u32) -> ServiceResult<()>;
    async fn update_invite(&self, id: u32, invite: &InviteData) -> ServiceResult<InviteData>;

    // ---- Enroll Secrets ----
    async fn verify_enroll_secret(&self, secret: &str) -> ServiceResult<EnrollSecretInfo>;
    async fn get_enroll_secrets(&self, team_id: Option<u32>) -> ServiceResult<Vec<fleet_types::enroll::EnrollSecret>>;
    async fn apply_enroll_secrets(&self, team_id: Option<u32>, secrets: &[fleet_types::enroll::EnrollSecret]) -> ServiceResult<()>;

    // ---- Password Reset ----
    async fn new_password_reset_request(&self, user_id: u32, expires_at: DateTime<Utc>, token: &str) -> ServiceResult<()>;
    async fn find_password_reset_by_token(&self, token: &str) -> ServiceResult<PasswordResetRequest>;
    async fn delete_password_reset_requests_for_user(&self, user_id: u32) -> ServiceResult<()>;

    // ---- Software ----
    async fn list_software(&self, opts: fleet_types::ListOptions, team_id: Option<u32>) -> ServiceResult<Vec<fleet_types::Software>>;
    async fn software_by_id(&self, id: u32) -> ServiceResult<fleet_types::Software>;
    async fn update_software_title_name(&self, id: u32, name: &str) -> ServiceResult<()>;
    async fn delete_software_installer(&self, title_id: u32) -> ServiceResult<()>;
    async fn delete_software_title_icon(&self, title_id: u32) -> ServiceResult<()>;

    // ---- Email Changes ----
    async fn confirm_pending_email_change(&self, user_id: u32, token: &str) -> ServiceResult<String>;

    // ---- Batch User Operations ----
    async fn save_users(&self, users: &[fleet_types::User]) -> ServiceResult<()>;
    async fn team_by_name(&self, name: &str) -> ServiceResult<fleet_types::Team>;

    // ---- Orbit ----
    async fn load_host_by_orbit_node_key(&self, orbit_node_key: &str) -> ServiceResult<fleet_types::Host>;
    async fn enroll_orbit(
        &self,
        hardware_uuid: &str,
        hardware_serial: &str,
        orbit_node_key: &str,
        team_id: Option<u32>,
    ) -> ServiceResult<fleet_types::Host>;
    async fn set_orbit_node_key(&self, host_id: u32, orbit_node_key: &str) -> ServiceResult<()>;
    async fn get_host_script_execution(&self, execution_id: &str) -> ServiceResult<fleet_types::script::HostScriptResult>;
    async fn save_host_script_result(&self, result: &fleet_types::script::HostScriptResult) -> ServiceResult<()>;
    async fn set_host_disk_encryption_key(&self, host_id: u32, key: &[u8], client_error: Option<&str>) -> ServiceResult<()>;

    // ---- Activities ----
    async fn new_activity(&self, user_id: Option<u32>, activity_type: &str, details: &JsonValue) -> ServiceResult<()>;
    async fn list_activities(&self, limit: u32, offset: u32) -> ServiceResult<Vec<fleet_types::Activity>>;
    async fn count_host_upcoming_activities(&self, host_id: u32) -> ServiceResult<u32>;
    async fn list_host_upcoming_activities(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::UpcomingActivity>>;
    async fn delete_host_upcoming_activity(&self, host_id: u32, activity_id: u32) -> ServiceResult<()>;

    // ---- Device ----
    async fn load_host_by_device_auth_token(&self, token: &str) -> ServiceResult<fleet_types::Host>;
    async fn set_or_update_device_auth_token(&self, host_id: u32, token: &str) -> ServiceResult<()>;
    async fn list_policies_for_host(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::policy::HostPolicy>>;
    async fn list_software_for_host(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::Software>>;
    async fn device_mapping_for_host(&self, host_id: u32) -> ServiceResult<serde_json::Value>;
    async fn set_custom_host_device_mapping(&self, host_id: u32, email: &str) -> ServiceResult<()>;
    async fn delete_host_idp_device_mapping(&self, host_id: u32) -> ServiceResult<()>;
    async fn mark_host_refetch_requested(&self, host_id: u32) -> ServiceResult<()>;

    // ---- Carves ----
    async fn new_carve(&self, carve: &fleet_types::CarveMetadata) -> ServiceResult<fleet_types::CarveMetadata>;
    async fn carve_by_id(&self, id: i64) -> ServiceResult<fleet_types::CarveMetadata>;
    async fn carve_by_session_id(&self, session_id: &str) -> ServiceResult<fleet_types::CarveMetadata>;
    async fn list_carves(&self, include_expired: bool) -> ServiceResult<Vec<fleet_types::CarveMetadata>>;
    async fn update_carve(&self, id: i64, max_block: i64, expired: bool, error: Option<&str>) -> ServiceResult<()>;
    async fn new_carve_block(&self, metadata_id: i64, block_id: i64, data: &[u8]) -> ServiceResult<()>;
    async fn get_carve_block(&self, metadata_id: i64, block_id: i64) -> ServiceResult<Vec<u8>>;

    // ---- Scripts ----
    async fn new_script(&self, team_id: Option<u32>, name: &str, contents: &str) -> ServiceResult<fleet_types::script::Script>;
    async fn script_by_id(&self, id: u32) -> ServiceResult<fleet_types::script::Script>;
    async fn list_scripts(&self, team_id: Option<u32>) -> ServiceResult<Vec<fleet_types::script::Script>>;
    async fn delete_script(&self, id: u32) -> ServiceResult<()>;
    async fn get_script_contents(&self, script_id: u32) -> ServiceResult<String>;

    // ---- Certificates ----
    async fn list_host_certificates(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::certificate::HostCertificate>>;
    async fn create_certificate_template(&self, team_id: u32, ca_id: i32, name: &str, subject_name: &str) -> ServiceResult<fleet_types::certificate::CertificateTemplate>;
    async fn get_certificate_template(&self, id: u32) -> ServiceResult<fleet_types::certificate::CertificateTemplate>;
    async fn list_certificate_templates(&self) -> ServiceResult<Vec<fleet_types::certificate::CertificateTemplate>>;
    async fn delete_certificate_template(&self, id: u32) -> ServiceResult<()>;
    async fn create_certificate_authority(&self, ca_type: &str, name: &str, url: &str) -> ServiceResult<fleet_types::certificate::CertificateAuthority>;
    async fn get_certificate_authority(&self, id: i32) -> ServiceResult<fleet_types::certificate::CertificateAuthority>;
    async fn list_certificate_authorities(&self) -> ServiceResult<Vec<fleet_types::certificate::CertificateAuthority>>;
    async fn delete_certificate_authority(&self, id: i32) -> ServiceResult<()>;
    async fn update_certificate_authority(&self, id: i32, name: &str, url: &str) -> ServiceResult<fleet_types::certificate::CertificateAuthority>;

    // ---- Setup Experience ----
    async fn get_setup_experience_script(&self, team_id: Option<u32>) -> ServiceResult<fleet_types::certificate::SetupExperienceScript>;
    async fn delete_setup_experience_script(&self, team_id: Option<u32>) -> ServiceResult<()>;
    async fn list_setup_experience_software_title_ids(&self, team_id: Option<u32>) -> ServiceResult<Vec<u32>>;
    async fn set_setup_experience_software(&self, team_id: Option<u32>, title_ids: &[u32]) -> ServiceResult<()>;

    // ---- Vulnerabilities ----
    async fn list_vulnerabilities(
        &self,
        team_id: Option<u32>,
        query: Option<&str>,
        exploit: Option<bool>,
        limit: u32,
        offset: u32,
    ) -> ServiceResult<Vec<fleet_types::vulnerability::VulnerabilityWithMetadata>>;
    async fn get_vulnerability(
        &self,
        cve: &str,
        team_id: Option<u32>,
    ) -> ServiceResult<fleet_types::vulnerability::VulnerabilityWithMetadata>;

    // ---- Software Install Results ----
    async fn get_software_install_result(&self, execution_id: &str) -> ServiceResult<fleet_types::software::SoftwareInstallResult>;

    // ---- Fleet Maintained Apps ----
    async fn list_fleet_maintained_apps(
        &self,
        query: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> ServiceResult<Vec<fleet_types::software::FleetMaintainedApp>>;
    async fn get_fleet_maintained_app(&self, id: u32) -> ServiceResult<fleet_types::software::FleetMaintainedApp>;

    // ---- Utilities ----
    async fn list_packs_for_host(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::Pack>>;
    async fn list_software_titles(&self, team_id: Option<u32>, limit: u32, offset: u32) -> ServiceResult<Vec<fleet_types::Software>>;
}

/// TeamSummaryInfo is a minimal team representation used for validation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TeamSummaryInfo {
    pub id: u32,
    pub name: String,
}

/// AppConfigData holds the application configuration stored in the database.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct AppConfigData {
    pub org_name: String,
    pub org_logo_url: String,
    pub org_logo_url_light_background: String,
    pub contact_url: String,
    pub server_url: String,
    pub live_query_disabled: bool,
    pub enable_sso: bool,
    pub sso_entity_id: String,
    pub sso_issuer_uri: String,
    pub sso_idp_name: String,
    pub sso_idp_image_url: String,
    pub sso_metadata: String,
    pub sso_metadata_url: String,
    pub smtp_configured: bool,
    pub smtp_sender_address: String,
    pub smtp_server: String,
    pub smtp_port: u16,
    pub smtp_enable_ssl_tls: bool,
    pub smtp_authentication_type: String,
    pub smtp_user_name: String,
    pub smtp_password: String,
    pub smtp_enable_start_tls: bool,
    pub smtp_domain: String,
    pub smtp_verify_ssl_certs: bool,
    pub host_expiry_enabled: bool,
    pub host_expiry_window: i64,
    pub agent_options: Option<serde_json::Value>,
    pub vulnerability_databases_path: String,
    pub enable_host_users: bool,
    pub enable_software_inventory: bool,
    pub transparency_url: String,
}

/// InviteData represents a user invitation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InviteData {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub invited_by: u32,
    pub email: String,
    pub name: String,
    pub position: String,
    pub token: String,
    pub sso_enabled: bool,
    pub mfa_enabled: bool,
    pub global_role: Option<String>,
    #[serde(default)]
    pub teams: Vec<fleet_types::team::UserTeam>,
}

/// EnrollSecretInfo contains info about a verified enrollment secret.
#[derive(Debug, Clone)]
pub struct EnrollSecretInfo {
    pub secret: String,
    pub team_id: Option<u32>,
}

/// PasswordResetRequest tracks a password reset request.
#[derive(Debug, Clone)]
pub struct PasswordResetRequest {
    pub id: u32,
    pub user_id: u32,
    pub token: String,
    pub expires_at: DateTime<Utc>,
}

// ---------------------------------------------------------------------------
// FleetService -- the main service struct.
// ---------------------------------------------------------------------------

pub struct FleetService {
    pub(crate) ds: Arc<dyn Datastore>,
    pub(crate) config: FleetServiceConfig,
    pub(crate) clock: Arc<dyn Clock>,
}

/// Clock trait for abstracting time (enables testing with fake clocks).
pub trait Clock: Send + Sync {
    fn now(&self) -> DateTime<Utc>;
}

/// Default clock implementation using the system clock.
pub struct SystemClock;

impl Clock for SystemClock {
    fn now(&self) -> DateTime<Utc> {
        Utc::now()
    }
}

impl FleetService {
    pub fn new(ds: Arc<dyn Datastore>, config: FleetServiceConfig) -> Self {
        Self {
            ds,
            config,
            clock: Arc::new(SystemClock),
        }
    }

    pub fn new_with_clock(
        ds: Arc<dyn Datastore>,
        config: FleetServiceConfig,
        clock: Arc<dyn Clock>,
    ) -> Self {
        Self { ds, config, clock }
    }

    pub fn datastore(&self) -> &dyn Datastore {
        self.ds.as_ref()
    }

    pub fn config(&self) -> &FleetServiceConfig {
        &self.config
    }
}
