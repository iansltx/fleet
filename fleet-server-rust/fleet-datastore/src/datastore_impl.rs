//! Implementation of the `Datastore` trait from `fleet_service` for `MysqlDatastore`.
//!
//! This module bridges the datastore layer (which uses internal Row types) with
//! the service layer (which uses `fleet_types` types and `ServiceResult`).

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use serde_json::Value as JsonValue;

use fleet_service::fleet_service::{
    AppConfigData, Datastore, EnrollSecretInfo, InviteData, PasswordResetRequest, TeamSummaryInfo,
};
use fleet_service::{ServiceError, ServiceResult};

use crate::error::DatastoreError;
use crate::MysqlDatastore;

// ---------------------------------------------------------------------------
// Error conversion: DatastoreError -> ServiceError
// ---------------------------------------------------------------------------

impl From<DatastoreError> for ServiceError {
    fn from(e: DatastoreError) -> Self {
        match e {
            DatastoreError::NotFound { entity, id, name } => {
                let msg = match (id, name) {
                    (Some(id), _) => format!("{} with id {}", entity, id),
                    (_, Some(name)) => format!("{} with name {}", entity, name),
                    _ => entity,
                };
                ServiceError::NotFound(msg)
            }
            DatastoreError::AlreadyExists { entity, name } => {
                ServiceError::Conflict(format!("{} already exists: {}", entity, name))
            }
            DatastoreError::ForeignKey { entity, name } => {
                ServiceError::Conflict(format!("foreign key constraint: {} ({})", entity, name))
            }
            DatastoreError::Validation(msg) => ServiceError::BadRequest(msg),
            _ => ServiceError::Internal(e.to_string()),
        }
    }
}

// ---------------------------------------------------------------------------
// Row-to-type conversion helpers
// ---------------------------------------------------------------------------

fn user_row_to_user(row: crate::users::UserRow) -> fleet_types::User {
    fleet_types::User {
        id: row.id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        password: row.password,
        salt: row.salt,
        name: row.name,
        email: row.email,
        admin_forced_password_reset: row.admin_forced_password_reset,
        gravatar_url: row.gravatar_url,
        position: row.position,
        sso_enabled: row.sso_enabled,
        mfa_enabled: row.mfa_enabled,
        global_role: row.global_role,
        api_only: row.api_only,
        teams: Vec::new(),
        settings: None,
    }
}

fn activity_row_to_activity(row: crate::activities::ActivityRow) -> fleet_types::Activity {
    fleet_types::Activity {
        id: row.id,
        created_at: row.created_at,
        user_id: row.user_id,
        user_name: row.user_name.unwrap_or_default(),
        user_email: row.user_email.unwrap_or_default(),
        activity_type: row.activity_type,
        details: row.details.unwrap_or(serde_json::json!({})),
    }
}

fn session_row_to_session(row: crate::sessions::SessionRow) -> fleet_types::Session {
    fleet_types::Session {
        id: row.id,
        created_at: row.created_at,
        accessed_at: row.accessed_at,
        user_id: row.user_id,
        key: row.key,
        api_only: row.api_only,
    }
}

fn host_row_to_host(row: crate::hosts::HostRow) -> fleet_types::Host {
    fleet_types::Host {
        id: row.id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        detail_updated_at: row.detail_updated_at,
        label_updated_at: row.label_updated_at,
        policy_updated_at: row.policy_updated_at,
        hostname: row.hostname,
        computer_name: row.computer_name,
        uuid: row.uuid,
        platform: row.platform,
        platform_like: row.platform_like,
        osquery_version: row.osquery_version,
        os_version: row.os_version,
        uptime: row.uptime,
        memory: row.memory,
        cpu_type: row.cpu_type,
        cpu_subtype: row.cpu_subtype,
        cpu_brand: row.cpu_brand,
        cpu_physical_cores: row.cpu_physical_cores,
        cpu_logical_cores: row.cpu_logical_cores,
        hardware_vendor: row.hardware_vendor,
        hardware_model: row.hardware_model,
        hardware_version: row.hardware_version,
        hardware_serial: row.hardware_serial,
        team_id: row.team_id,
        distributed_interval: row.distributed_interval as u32,
        logger_tls_period: row.logger_tls_period as u32,
        config_tls_refresh: row.config_tls_refresh as u32,
        primary_ip: row.primary_ip,
        primary_mac: row.primary_mac,
        public_ip: row.public_ip,
        refetch_requested: row.refetch_requested,
        refetch_critical_queries_until: row.refetch_critical_queries_until,
        last_enrolled_at: row.last_enrolled_at.unwrap_or_default(),
        last_restarted_at: row.last_restarted_at.unwrap_or_default(),
        // Default values for fields not in HostRow
        seen_time: DateTime::<Utc>::default(),
        host_software: fleet_types::host::HostSoftware::default(),
        orbit_version: None,
        desktop_version: None,
        scripts_enabled: None,
        build: String::new(),
        code_name: String::new(),
        timezone: None,
        primary_ip_id: None,
        team_name: None,
        additional: None,
        users: Vec::new(),
        gigs_disk_space_available: 0.0,
        percent_disk_space_available: 0.0,
        gigs_total_disk_space: 0.0,
        gigs_all_disk_space: None,
        disk_encryption_enabled: None,
        issues: fleet_types::host::HostIssues::default(),
        device_mapping: None,
        mdm: fleet_types::host::MDMHostData::default(),
        dep_assigned_to_fleet: None,
        policies: None,
        pack_stats: Vec::new(),
    }
}

fn query_row_to_query(row: crate::queries::QueryRow) -> fleet_types::Query {
    fleet_types::Query {
        id: row.id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        team_id: row.team_id,
        name: row.name,
        description: row.description,
        query: row.query,
        author_id: row.author_id,
        saved: row.saved,
        observer_can_run: row.observer_can_run,
        interval: row.schedule_interval,
        platform: row.platform,
        min_osquery_version: row.min_osquery_version,
        automations_enabled: row.automations_enabled,
        logging: row.logging_type,
        discard_data: row.discard_data,
        author_name: row.author_name,
        author_email: row.author_email,
        packs: Vec::new(),
        aggregated_stats: fleet_types::query::AggregatedStats {
            user_time_p50: row.user_time_p50,
            user_time_p95: row.user_time_p95,
            system_time_p50: row.system_time_p50,
            system_time_p95: row.system_time_p95,
            total_executions: row.total_executions,
        },
        labels_include_any: Vec::new(),
    }
}

fn pack_row_to_pack(row: crate::packs::PackRow) -> fleet_types::Pack {
    fleet_types::Pack {
        id: row.id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        name: row.name,
        description: row.description.unwrap_or_default(),
        platform: row.platform.unwrap_or_default(),
        disabled: row.disabled,
        pack_type: row.pack_type,
        labels: Vec::new(),
        label_ids: Vec::new(),
        hosts: Vec::new(),
        host_ids: Vec::new(),
        teams: Vec::new(),
        team_ids: Vec::new(),
    }
}

fn sq_row_to_scheduled_query(row: crate::packs::ScheduledQueryFullRow) -> fleet_types::ScheduledQuery {
    fleet_types::ScheduledQuery {
        id: row.id,
        pack_id: row.pack_id,
        query_id: row.query_id,
        query_name: row.query_name,
        query: String::new(),
        name: row.name,
        description: row.description,
        interval: row.interval,
        snapshot: row.snapshot,
        removed: row.removed,
        platform: row.platform.unwrap_or_default(),
        version: row.version.unwrap_or_default(),
        shard: row.shard,
        denylist: row.denylist,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn label_row_to_label(row: crate::labels::LabelRow) -> fleet_types::Label {
    fleet_types::Label {
        id: row.id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        name: row.name,
        description: row.description,
        query: row.query,
        platform: row.platform,
        label_type: match row.label_type {
            1 => fleet_types::LabelType::BuiltIn,
            _ => fleet_types::LabelType::Regular,
        },
        label_membership_type: match row.label_membership_type {
            1 => fleet_types::label::LabelMembershipType::Manual,
            2 => fleet_types::label::LabelMembershipType::HostVitals,
            _ => fleet_types::label::LabelMembershipType::Dynamic,
        },
        host_count: row.host_count.unwrap_or(0),
        team_id: row.team_id,
        author_id: None,
        criteria: None,
    }
}

fn policy_row_to_policy(row: crate::policies::PolicyRow) -> fleet_types::Policy {
    fleet_types::Policy {
        policy_data: fleet_types::PolicyData {
            id: row.id,
            name: row.name,
            query: row.query,
            critical: row.critical,
            description: row.description,
            author_id: row.author_id,
            author_name: row.author_name,
            author_email: row.author_email,
            team_id: row.team_id,
            resolution: row.resolution,
            platform: row.platforms,
            labels_include_any: Vec::new(),
            labels_exclude_any: Vec::new(),
            calendar_events_enabled: row.calendar_events_enabled,
            conditional_access_enabled: false,
            conditional_access_bypass_enabled: None,
            created_at: row.created_at,
            updated_at: row.updated_at,
        },
        passing_host_count: row.passing_host_count.unwrap_or(0),
        failing_host_count: row.failing_host_count.unwrap_or(0),
        host_count_updated_at: row.host_count_updated_at,
        install_software: None,
        run_script: None,
    }
}

fn team_row_to_team(row: crate::teams::TeamRow) -> fleet_types::Team {
    let config: fleet_types::TeamConfig =
        serde_json::from_value(row.config).unwrap_or_default();
    fleet_types::Team {
        id: row.id,
        created_at: row.created_at,
        name: row.name,
        description: row.description,
        gitops_filename: row.filename,
        config,
        user_count: 0,
        users: Vec::new(),
        host_count: 0,
        hosts: Vec::new(),
        secrets: None,
    }
}

fn invite_row_to_invite_data(row: crate::invites::InviteRow) -> InviteData {
    InviteData {
        id: row.id,
        created_at: row.created_at,
        updated_at: row.updated_at,
        invited_by: row.invited_by,
        email: row.email,
        name: row.name,
        position: row.position,
        token: row.token,
        sso_enabled: row.sso_enabled,
        mfa_enabled: row.mfa_enabled,
        global_role: row.global_role,
        teams: Vec::new(),
    }
}

fn software_row_to_software(row: crate::software::SoftwareRow) -> fleet_types::Software {
    fleet_types::Software {
        id: row.id,
        name: row.name.clone(),
        version: row.version,
        bundle_identifier: row.bundle_identifier.unwrap_or_default(),
        source: row.source,
        extension_id: String::new(),
        extension_for: String::new(),
        browser: String::new(),
        release: row.sw_release,
        vendor: row.vendor,
        arch: row.arch,
        generated_cpe: String::new(),
        vulnerabilities: Vec::new(),
        hosts_count: 0,
        last_opened_at: None,
        application_id: None,
        upgrade_code: None,
        display_name: row.name,
    }
}

fn software_title_row_to_software(row: crate::software::SoftwareTitleRow) -> fleet_types::Software {
    fleet_types::Software {
        id: row.id,
        name: row.name.clone(),
        version: String::new(),
        bundle_identifier: String::new(),
        source: row.source,
        extension_id: String::new(),
        extension_for: String::new(),
        browser: row.browser,
        release: String::new(),
        vendor: String::new(),
        arch: String::new(),
        generated_cpe: String::new(),
        vulnerabilities: Vec::new(),
        hosts_count: row.hosts_count.unwrap_or(0) as i32,
        last_opened_at: None,
        application_id: None,
        upgrade_code: None,
        display_name: row.name,
    }
}

/// Enriches a slice of Software with CVE data from the software_cve table.
async fn enrich_software_with_cves(ds: &MysqlDatastore, software: &mut [fleet_types::Software]) {
    let ids: Vec<u32> = software.iter().map(|s| s.id).collect();
    if ids.is_empty() {
        return;
    }
    let cve_rows = match MysqlDatastore::list_cves_for_software_ids(ds, &ids).await {
        Ok(rows) => rows,
        Err(_) => return, // Silently skip CVE enrichment on error
    };
    // Group CVEs by software_id
    let mut cve_map: std::collections::HashMap<u32, Vec<fleet_types::vulnerability::CVE>> =
        std::collections::HashMap::new();
    for row in cve_rows {
        cve_map
            .entry(row.software_id)
            .or_default()
            .push(fleet_types::vulnerability::CVE {
                cve: row.cve.clone(),
                details_link: format!("https://nvd.nist.gov/vuln/detail/{}", row.cve),
                created_at: row.created_at,
                cvss_score: None,
                epss_probability: None,
                cisa_known_exploit: None,
                cve_published: None,
                description: None,
                resolved_in_version: row.resolved_in_version.map(Some),
            });
    }
    for sw in software.iter_mut() {
        if let Some(cves) = cve_map.remove(&sw.id) {
            sw.vulnerabilities = cves;
        }
    }
}

fn carve_row_to_carve(row: crate::carves::CarveRow) -> fleet_types::CarveMetadata {
    fleet_types::CarveMetadata {
        id: row.id,
        created_at: row.created_at,
        host_id: row.host_id,
        name: row.name,
        block_count: row.block_count,
        block_size: row.block_size,
        carve_size: row.carve_size,
        carve_id: row.carve_id,
        request_id: row.request_id,
        session_id: row.session_id,
        expired: row.expired,
        error: row.error,
        max_block: row.max_block,
    }
}

fn script_row_to_script(row: crate::scripts::ScriptRow) -> fleet_types::script::Script {
    fleet_types::script::Script {
        id: row.id,
        team_id: row.team_id,
        name: row.name,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

fn app_config_json_to_data(val: serde_json::Value) -> AppConfigData {
    serde_json::from_value(val).unwrap_or_default()
}

fn app_config_data_to_json(data: &AppConfigData) -> serde_json::Value {
    serde_json::to_value(data).unwrap_or_default()
}

fn sv_row_to_secret_variable(row: crate::app_config::SecretVariableRow) -> fleet_types::config::SecretVariable {
    fleet_types::config::SecretVariable {
        id: row.id,
        name: row.name,
        value: row.value,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

/// Convert a host platform string to a Fleet platform (matching Go's PlatformFromHost).
fn fleet_platform_from_host(platform: &str) -> String {
    match platform {
        "darwin" | "windows" | "CrOS" | "chrome" | "ios" | "ipados" | "android" => {
            platform.to_string()
        }
        p if is_linux(p) => "linux".to_string(),
        _ => String::new(),
    }
}

fn is_linux(platform: &str) -> bool {
    matches!(
        platform,
        "linux" | "ubuntu" | "debian" | "rhel" | "centos" | "sles" | "kali"
            | "gentoo" | "amzn" | "pop" | "arch" | "linuxmint" | "void"
            | "nixos" | "endeavouros" | "manjaro" | "opensuse-leap"
            | "opensuse-tumbleweed" | "tuxedo" | "fedora"
    )
}

/// Row type for host policy queries.
#[derive(Debug, sqlx::FromRow)]
struct HostPolicyRow {
    id: u32,
    team_id: Option<u32>,
    resolution: String,
    name: String,
    query: String,
    description: String,
    author_id: Option<u32>,
    platform: String,
    critical: bool,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
    conditional_access_enabled: bool,
    conditional_access_bypass_enabled: Option<bool>,
    author_name: String,
    author_email: String,
    response: String,
}

fn host_policy_row_to_host_policy(row: HostPolicyRow) -> fleet_types::policy::HostPolicy {
    fleet_types::policy::HostPolicy {
        policy_data: fleet_types::PolicyData {
            id: row.id,
            name: row.name,
            query: row.query,
            critical: row.critical,
            description: row.description,
            author_id: row.author_id,
            author_name: row.author_name,
            author_email: row.author_email,
            team_id: row.team_id,
            resolution: Some(row.resolution),
            platform: row.platform,
            labels_include_any: Vec::new(),
            labels_exclude_any: Vec::new(),
            calendar_events_enabled: false,
            conditional_access_enabled: row.conditional_access_enabled,
            conditional_access_bypass_enabled: row.conditional_access_bypass_enabled,
            created_at: row.created_at,
            updated_at: row.updated_at,
        },
        response: row.response,
    }
}

/// Row type for software-per-host queries.
#[derive(Debug, sqlx::FromRow)]
struct SoftwareForHostRow {
    id: u32,
    name: String,
    version: String,
    source: String,
    extension_for: String,
    bundle_identifier: String,
    release: String,
    vendor: String,
    arch: String,
    extension_id: String,
    upgrade_code: Option<String>,
    last_opened_at: Option<DateTime<Utc>>,
}

fn software_for_host_row_to_software(row: SoftwareForHostRow) -> fleet_types::Software {
    fleet_types::Software {
        id: row.id,
        name: row.name.clone(),
        version: row.version,
        bundle_identifier: row.bundle_identifier,
        source: row.source,
        extension_id: row.extension_id,
        extension_for: row.extension_for,
        browser: String::new(),
        release: row.release,
        vendor: row.vendor,
        arch: row.arch,
        generated_cpe: String::new(),
        vulnerabilities: Vec::new(),
        hosts_count: 0,
        last_opened_at: row.last_opened_at,
        application_id: None,
        upgrade_code: row.upgrade_code,
        display_name: row.name,
    }
}

/// Row type for device mapping queries.
#[derive(Debug, sqlx::FromRow)]
struct DeviceMappingRow {
    id: u32,
    host_id: u32,
    email: String,
    source: String,
}

/// Row type for host script results.
#[derive(Debug, sqlx::FromRow)]
struct HostScriptResultRow {
    id: u32,
    host_id: u32,
    execution_id: String,
    script_contents: String,
    script_id: Option<u32>,
    output: String,
    runtime: i32,
    exit_code: Option<i64>,
    host_timeout: bool,
    host_deleted_at: Option<DateTime<Utc>>,
    created_at: DateTime<Utc>,
    updated_at: DateTime<Utc>,
}

fn host_script_result_row_to_result(
    row: HostScriptResultRow,
) -> fleet_types::script::HostScriptResult {
    fleet_types::script::HostScriptResult {
        id: row.id,
        host_id: row.host_id,
        execution_id: row.execution_id,
        script_id: row.script_id,
        script_contents: row.script_contents,
        output: row.output,
        runtime: row.runtime,
        exit_code: row.exit_code,
        message: None,
        host_timeout: row.host_timeout,
        host_deleted_at: row.host_deleted_at,
        created_at: row.created_at,
        updated_at: row.updated_at,
    }
}

/// Helper to convert sqlx errors into ServiceError for inline queries.
fn ds_error(e: sqlx::Error) -> ServiceError {
    ServiceError::Internal(e.to_string())
}

// ---------------------------------------------------------------------------
// Datastore trait implementation
// ---------------------------------------------------------------------------

#[async_trait]
impl Datastore for MysqlDatastore {
    // ---- Users ----

    async fn user_by_id(&self, id: u32) -> ServiceResult<fleet_types::User> {
        let row = self.user_by_id(id).await.map_err(ServiceError::from)?;
        Ok(user_row_to_user(row))
    }

    async fn user_by_email(&self, email: &str) -> ServiceResult<fleet_types::User> {
        let row = self.user_by_email(email).await.map_err(ServiceError::from)?;
        Ok(user_row_to_user(row))
    }

    async fn list_users(
        &self,
        opts: fleet_types::user::UserListOptions,
    ) -> ServiceResult<Vec<fleet_types::User>> {
        let match_query_str = if opts.list_options.match_query.is_empty() {
            None
        } else {
            Some(opts.list_options.match_query.as_str())
        };
        let order_desc = matches!(
            opts.list_options.order_direction,
            fleet_types::OrderDirection::Descending
        );
        let rows = self
            .list_users(
                opts.team_id,
                match_query_str,
                &opts.list_options.order_key,
                order_desc,
                opts.list_options.per_page,
                opts.list_options.page * opts.list_options.per_page,
            )
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(user_row_to_user).collect())
    }

    async fn save_user(&self, user: &fleet_types::User) -> ServiceResult<fleet_types::User> {
        let params = crate::users::SaveUserParams {
            id: user.id,
            password: user.password.clone(),
            salt: user.salt.clone(),
            name: user.name.clone(),
            email: user.email.clone(),
            admin_forced_password_reset: user.admin_forced_password_reset,
            gravatar_url: user.gravatar_url.clone(),
            position: user.position.clone(),
            sso_enabled: user.sso_enabled,
            mfa_enabled: user.mfa_enabled,
            api_only: user.api_only,
            global_role: user.global_role.clone(),
            settings_json: user
                .settings
                .as_ref()
                .and_then(|s| serde_json::to_vec(s).ok()),
        };
        MysqlDatastore::save_user(self, params)
            .await
            .map_err(ServiceError::from)?;
        // Return the updated user by re-fetching
        let row = MysqlDatastore::user_by_id(self, user.id)
            .await
            .map_err(ServiceError::from)?;
        Ok(user_row_to_user(row))
    }

    async fn new_user(&self, user: &fleet_types::User) -> ServiceResult<fleet_types::User> {
        let params = crate::users::NewUserParams {
            password: user.password.clone(),
            salt: user.salt.clone(),
            name: user.name.clone(),
            email: user.email.clone(),
            admin_forced_password_reset: user.admin_forced_password_reset,
            gravatar_url: user.gravatar_url.clone(),
            position: user.position.clone(),
            sso_enabled: user.sso_enabled,
            mfa_enabled: user.mfa_enabled,
            api_only: user.api_only,
            global_role: user.global_role.clone(),
            invite_id: None,
        };
        let row = MysqlDatastore::new_user(self, params)
            .await
            .map_err(ServiceError::from)?;
        Ok(user_row_to_user(row))
    }

    async fn delete_user(&self, id: u32) -> ServiceResult<()> {
        MysqlDatastore::delete_user(self, id)
            .await
            .map_err(ServiceError::from)
    }

    async fn has_users(&self) -> ServiceResult<bool> {
        MysqlDatastore::has_users(self).await.map_err(ServiceError::from)
    }

    async fn count_global_admins(&self) -> ServiceResult<i64> {
        MysqlDatastore::count_global_admins(self).await.map_err(ServiceError::from)
    }

    async fn user_settings(&self, user_id: u32) -> ServiceResult<Option<serde_json::Value>> {
        let raw = MysqlDatastore::user_settings(self, user_id).await.map_err(ServiceError::from)?;
        match raw {
            Some(bytes) => {
                let val = serde_json::from_slice(&bytes).unwrap_or(serde_json::json!({}));
                Ok(Some(val))
            }
            None => Ok(None),
        }
    }

    async fn save_user_settings(&self, user_id: u32, settings: &serde_json::Value) -> ServiceResult<()> {
        let bytes = serde_json::to_vec(settings).map_err(|e| ServiceError::internal(e.to_string()))?;
        sqlx::query("UPDATE users SET settings = ? WHERE id = ?")
            .bind(&bytes)
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(|e| ServiceError::internal(e.to_string()))?;
        Ok(())
    }

    // ---- Sessions ----

    async fn new_session(
        &self,
        user_id: u32,
        key_size: usize,
    ) -> ServiceResult<fleet_types::Session> {
        // Generate a random session key
        use rand::Rng;
        let key: String = rand::thread_rng()
            .sample_iter(&rand::distributions::Alphanumeric)
            .take(key_size)
            .map(char::from)
            .collect();
        let row = MysqlDatastore::new_session(self, user_id, &key)
            .await
            .map_err(ServiceError::from)?;
        Ok(session_row_to_session(row))
    }

    async fn session_by_id(&self, id: u32) -> ServiceResult<fleet_types::Session> {
        let row = MysqlDatastore::session_by_id(self, id)
            .await
            .map_err(ServiceError::from)?;
        Ok(session_row_to_session(row))
    }

    async fn session_by_key(&self, key: &str) -> ServiceResult<fleet_types::Session> {
        let row = MysqlDatastore::session_by_key(self, key)
            .await
            .map_err(ServiceError::from)?;
        Ok(session_row_to_session(row))
    }

    async fn mark_session_accessed(&self, session: &fleet_types::Session) -> ServiceResult<()> {
        MysqlDatastore::mark_session_accessed(self, session.id)
            .await
            .map_err(ServiceError::from)
    }

    async fn destroy_session(&self, session: &fleet_types::Session) -> ServiceResult<()> {
        MysqlDatastore::destroy_session(self, session.id)
            .await
            .map_err(ServiceError::from)
    }

    async fn destroy_all_sessions_for_user(&self, user_id: u32) -> ServiceResult<()> {
        MysqlDatastore::destroy_all_sessions_for_user(self, user_id)
            .await
            .map_err(ServiceError::from)
    }

    async fn list_sessions_for_user(&self, user_id: u32) -> ServiceResult<Vec<fleet_types::Session>> {
        let rows = MysqlDatastore::list_sessions_for_user(self, user_id)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(session_row_to_session).collect())
    }

    // ---- Hosts ----

    async fn host(&self, id: u32) -> ServiceResult<fleet_types::Host> {
        let row = self.host_by_id(id).await.map_err(ServiceError::from)?;
        Ok(host_row_to_host(row))
    }

    async fn host_by_identifier(&self, identifier: &str) -> ServiceResult<fleet_types::Host> {
        // Try parsing as u32 ID first, then fall back to node_key
        if let Ok(id) = identifier.parse::<u32>() {
            let row = self.host_by_id(id).await.map_err(ServiceError::from)?;
            return Ok(host_row_to_host(row));
        }
        let row = self
            .authenticate_host(identifier)
            .await
            .map_err(ServiceError::from)?;
        Ok(host_row_to_host(row))
    }

    async fn list_hosts(
        &self,
        opts: fleet_types::HostListOptions,
    ) -> ServiceResult<Vec<fleet_types::Host>> {
        let rows = MysqlDatastore::list_hosts(
            self,
            opts.team_filter,
            opts.list_options.per_page,
            opts.list_options.page * opts.list_options.per_page,
        )
        .await
        .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(host_row_to_host).collect())
    }

    async fn delete_host(&self, id: u32) -> ServiceResult<()> {
        MysqlDatastore::delete_host(self, id)
            .await
            .map_err(ServiceError::from)
    }

    async fn host_lite(&self, id: u32) -> ServiceResult<fleet_types::Host> {
        // Use the same host_by_id for now (lite variant not separately implemented)
        let row = self.host_by_id(id).await.map_err(ServiceError::from)?;
        Ok(host_row_to_host(row))
    }

    async fn load_host_by_node_key(&self, node_key: &str) -> ServiceResult<fleet_types::Host> {
        let row = self
            .authenticate_host(node_key)
            .await
            .map_err(ServiceError::from)?;
        Ok(host_row_to_host(row))
    }

    async fn mark_host_seen(
        &self,
        host_id: u32,
        seen_at: DateTime<Utc>,
    ) -> ServiceResult<()> {
        MysqlDatastore::mark_host_seen(self, host_id, seen_at)
            .await
            .map_err(ServiceError::from)
    }

    async fn enroll_host(
        &self,
        host_identifier: &str,
        node_key: &str,
        team_id: Option<u32>,
        hardware_uuid: &str,
        hardware_serial: &str,
    ) -> ServiceResult<fleet_types::Host> {
        let host_id = MysqlDatastore::enroll_host(
            self,
            host_identifier,
            node_key,
            team_id,
            hardware_uuid,  // used as hostname in the datastore method
            "",             // platform
            hardware_serial,
        )
        .await
        .map_err(ServiceError::from)?;
        // Re-fetch the full host
        self.host(host_id).await
    }

    async fn host_summary(&self) -> ServiceResult<fleet_types::HostSummary> {
        let row = MysqlDatastore::host_summary(self)
            .await
            .map_err(ServiceError::from)?;
        Ok(fleet_types::HostSummary {
            totals_hosts_count: row.totals_count as u32,
            online_count: Some(row.online_count as u32),
            offline_count: Some(row.offline_count as u32),
            mia_count: Some(row.mia_count as u32),
            missing_30_days_count: Some(row.mia_count as u32),
            new_count: Some(row.new_count as u32),
            platforms: vec![
                fleet_types::host::HostSummaryPlatform { platform: "linux".to_string(), hosts_count: row.linux_count as u32 },
                fleet_types::host::HostSummaryPlatform { platform: "darwin".to_string(), hosts_count: row.macos_count as u32 },
                fleet_types::host::HostSummaryPlatform { platform: "windows".to_string(), hosts_count: row.windows_count as u32 },
                fleet_types::host::HostSummaryPlatform { platform: "chrome".to_string(), hosts_count: row.chrome_count as u32 },
            ],
        })
    }

    async fn transfer_hosts_to_team(&self, host_ids: &[u32], team_id: Option<u32>) -> ServiceResult<()> {
        if host_ids.is_empty() {
            return Ok(());
        }
        let placeholders: Vec<&str> = host_ids.iter().map(|_| "?").collect();
        let sql = format!(
            "UPDATE hosts SET team_id = ? WHERE id IN ({})",
            placeholders.join(",")
        );
        let mut query = sqlx::query(&sql).bind(team_id);
        for &id in host_ids {
            query = query.bind(id);
        }
        query.execute(self.pool()).await.map_err(ds_error)?;
        Ok(())
    }

    async fn list_os_versions(&self) -> ServiceResult<Vec<fleet_types::OSVersionStats>> {
        #[derive(sqlx::FromRow)]
        struct OsRow {
            os_version: String,
            platform: String,
            cnt: i64,
        }
        let rows: Vec<OsRow> = sqlx::query_as(
            "SELECT os_version, platform, COUNT(*) as cnt FROM hosts WHERE os_version != '' GROUP BY os_version, platform ORDER BY cnt DESC"
        )
        .fetch_all(self.pool())
        .await
        .map_err(ds_error)?;

        let mut result = Vec::with_capacity(rows.len());
        for (i, row) in rows.into_iter().enumerate() {
            let name_only = row.os_version.split_whitespace().next().unwrap_or("").to_string();
            let version = row.os_version.split_whitespace().skip(1).collect::<Vec<_>>().join(" ");
            result.push(fleet_types::OSVersionStats {
                id: (i + 1) as u32,
                name: row.os_version.clone(),
                name_only,
                version,
                platform: row.platform,
                hosts_count: row.cnt as u32,
                ..Default::default()
            });
        }
        Ok(result)
    }

    async fn os_version(&self, id: u32) -> ServiceResult<fleet_types::OSVersionStats> {
        let all = self.list_os_versions().await?;
        all.into_iter()
            .find(|v| v.id == id)
            .ok_or_else(|| ServiceError::not_found("os version"))
    }

    async fn search_hosts(&self, query: &str, omit_ids: &[u32], limit: u32) -> ServiceResult<Vec<fleet_types::Host>> {
        let search = format!("%{}%", query);
        let limit = limit.min(100);
        let base_sql = "SELECT * FROM hosts WHERE (hostname LIKE ? OR computer_name LIKE ? OR hardware_serial LIKE ?)";

        if omit_ids.is_empty() {
            let sql = format!("{} ORDER BY hostname LIMIT ?", base_sql);
            let rows: Vec<crate::hosts::HostRow> = sqlx::query_as(&sql)
                .bind(&search)
                .bind(&search)
                .bind(&search)
                .bind(limit)
                .fetch_all(self.pool())
                .await
                .map_err(ds_error)?;
            Ok(rows.into_iter().map(host_row_to_host).collect())
        } else {
            let placeholders: Vec<&str> = omit_ids.iter().map(|_| "?").collect();
            let sql = format!(
                "{} AND id NOT IN ({}) ORDER BY hostname LIMIT ?",
                base_sql,
                placeholders.join(",")
            );
            let mut q = sqlx::query_as::<_, crate::hosts::HostRow>(&sql)
                .bind(&search)
                .bind(&search)
                .bind(&search);
            for &id in omit_ids {
                q = q.bind(id);
            }
            let rows: Vec<crate::hosts::HostRow> = q.bind(limit).fetch_all(self.pool()).await.map_err(ds_error)?;
            Ok(rows.into_iter().map(host_row_to_host).collect())
        }
    }

    // ---- Queries ----

    async fn query(&self, id: u32) -> ServiceResult<fleet_types::Query> {
        let row = self.query_by_id(id).await.map_err(ServiceError::from)?;
        Ok(query_row_to_query(row))
    }

    async fn list_queries(
        &self,
        opts: fleet_types::ListOptions,
        team_id: Option<u32>,
    ) -> ServiceResult<Vec<fleet_types::Query>> {
        let rows = MysqlDatastore::list_queries(
            self,
            team_id,
            &opts.match_query,
            &opts.order_key,
            opts.per_page,
            opts.page * opts.per_page,
        )
        .await
        .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(query_row_to_query).collect())
    }

    async fn new_query(
        &self,
        query: &fleet_types::Query,
    ) -> ServiceResult<fleet_types::Query> {
        let team_id_char = query
            .team_id
            .map(|id| id.to_string())
            .unwrap_or_default();
        let params = crate::queries::NewQueryParams {
            name: query.name.clone(),
            description: query.description.clone(),
            query: query.query.clone(),
            saved: query.saved,
            author_id: query.author_id,
            observer_can_run: query.observer_can_run,
            team_id: query.team_id,
            team_id_char,
            platform: query.platform.clone(),
            min_osquery_version: query.min_osquery_version.clone(),
            schedule_interval: query.interval,
            automations_enabled: query.automations_enabled,
            logging_type: query.logging.clone(),
            discard_data: query.discard_data,
        };
        let id = MysqlDatastore::new_query(self, params)
            .await
            .map_err(ServiceError::from)?;
        self.query(id).await
    }

    async fn save_query(
        &self,
        query: &fleet_types::Query,
    ) -> ServiceResult<fleet_types::Query> {
        let team_id_char = query
            .team_id
            .map(|id| id.to_string())
            .unwrap_or_default();
        MysqlDatastore::save_query(
            self,
            query.id,
            &query.name,
            &query.description,
            &query.query,
            query.author_id,
            query.saved,
            query.observer_can_run,
            query.team_id,
            &team_id_char,
            &query.platform,
            &query.min_osquery_version,
            query.interval,
            query.automations_enabled,
            &query.logging,
            query.discard_data,
        )
        .await
        .map_err(ServiceError::from)?;
        self.query(query.id).await
    }

    async fn delete_query(&self, name: &str, team_id: Option<u32>) -> ServiceResult<()> {
        MysqlDatastore::delete_query(self, team_id, name)
            .await
            .map_err(ServiceError::from)
    }

    async fn delete_queries(&self, ids: &[u32]) -> ServiceResult<u32> {
        let count = MysqlDatastore::delete_queries(self, ids)
            .await
            .map_err(ServiceError::from)?;
        Ok(count as u32)
    }

    async fn query_by_name(&self, team_id: Option<u32>, name: &str) -> ServiceResult<fleet_types::Query> {
        let row = MysqlDatastore::query_by_name(self, team_id, name)
            .await
            .map_err(ServiceError::from)?;
        Ok(query_row_to_query(row))
    }

    async fn query_result_rows(&self, query_id: u32) -> ServiceResult<Vec<fleet_types::QueryResultRow>> {
        let rows = MysqlDatastore::query_result_rows(self, query_id)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(|r| {
            let columns = r.data
                .and_then(|d| serde_json::from_str(&d).ok())
                .unwrap_or(serde_json::json!({}));
            fleet_types::QueryResultRow {
                host_id: r.host_id,
                hostname: r.hostname,
                last_fetched: r.last_fetched,
                columns,
            }
        }).collect())
    }

    // ---- Packs ----

    async fn pack(&self, id: u32) -> ServiceResult<fleet_types::Pack> {
        let row = self.pack_by_id(id).await.map_err(ServiceError::from)?;
        Ok(pack_row_to_pack(row))
    }

    async fn pack_by_name(&self, name: &str) -> ServiceResult<Option<fleet_types::Pack>> {
        let row = MysqlDatastore::pack_by_name(self, name)
            .await
            .map_err(ServiceError::from)?;
        Ok(row.map(pack_row_to_pack))
    }

    async fn list_packs(
        &self,
        _opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<fleet_types::Pack>> {
        let rows = MysqlDatastore::list_packs(self, false)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(pack_row_to_pack).collect())
    }

    async fn new_pack(&self, pack: &fleet_types::Pack) -> ServiceResult<fleet_types::Pack> {
        let id = MysqlDatastore::new_pack(
            self,
            &pack.name,
            &pack.description,
            &pack.platform,
            pack.disabled,
        )
        .await
        .map_err(ServiceError::from)?;
        self.pack(id).await
    }

    async fn save_pack(&self, pack: &fleet_types::Pack) -> ServiceResult<()> {
        MysqlDatastore::save_pack(
            self,
            pack.id,
            &pack.name,
            &pack.platform,
            pack.disabled,
            &pack.description,
        )
        .await
        .map_err(ServiceError::from)
    }

    async fn delete_pack(&self, name: &str) -> ServiceResult<()> {
        MysqlDatastore::delete_pack(self, name)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Scheduled Queries ----

    async fn scheduled_query(&self, id: u32) -> ServiceResult<fleet_types::ScheduledQuery> {
        let row = MysqlDatastore::scheduled_query_by_id(self, id)
            .await
            .map_err(ServiceError::from)?;
        Ok(sq_row_to_scheduled_query(row))
    }
    async fn list_scheduled_queries_in_pack(&self, pack_id: u32) -> ServiceResult<Vec<fleet_types::ScheduledQuery>> {
        let rows = MysqlDatastore::list_scheduled_queries_in_pack_full(self, pack_id)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(sq_row_to_scheduled_query).collect())
    }
    async fn new_scheduled_query(&self, sq: &fleet_types::ScheduledQuery) -> ServiceResult<fleet_types::ScheduledQuery> {
        let id = MysqlDatastore::insert_scheduled_query(
            self,
            sq.pack_id,
            sq.query_id,
            &sq.query_name,
            &sq.name,
            &sq.description,
            sq.interval,
            sq.snapshot,
            sq.removed,
            &sq.platform,
            &sq.version,
            sq.shard,
        )
        .await
        .map_err(ServiceError::from)?;
        self.scheduled_query(id).await
    }
    async fn save_scheduled_query(&self, sq: &fleet_types::ScheduledQuery) -> ServiceResult<fleet_types::ScheduledQuery> {
        MysqlDatastore::update_scheduled_query(
            self,
            sq.id,
            sq.interval,
            sq.snapshot,
            sq.removed,
            &sq.platform,
            &sq.version,
            sq.shard,
        )
        .await
        .map_err(ServiceError::from)?;
        self.scheduled_query(sq.id).await
    }
    async fn delete_scheduled_query(&self, id: u32) -> ServiceResult<()> {
        MysqlDatastore::remove_scheduled_query(self, id)
            .await
            .map_err(ServiceError::from)
    }
    async fn ensure_global_pack(&self) -> ServiceResult<u32> {
        MysqlDatastore::ensure_pack_by_type(self, "global", "Global")
            .await
            .map_err(ServiceError::from)
    }
    async fn ensure_team_pack(&self, team_id: u32) -> ServiceResult<u32> {
        let pack_type = format!("team-{}", team_id);
        let name = format!("Team {}", team_id);
        MysqlDatastore::ensure_pack_by_type(self, &pack_type, &name)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Labels ----

    async fn label(&self, id: u32) -> ServiceResult<fleet_types::Label> {
        let row = self.label_by_id(id).await.map_err(ServiceError::from)?;
        Ok(label_row_to_label(row))
    }

    async fn label_by_name(&self, name: &str) -> ServiceResult<fleet_types::Label> {
        let row = MysqlDatastore::label_by_name(self, name)
            .await
            .map_err(ServiceError::from)?;
        Ok(label_row_to_label(row))
    }

    async fn list_labels(
        &self,
        _opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<fleet_types::Label>> {
        let rows = MysqlDatastore::list_labels(self, None)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(label_row_to_label).collect())
    }

    async fn new_label(
        &self,
        label: &fleet_types::Label,
    ) -> ServiceResult<fleet_types::Label> {
        let label_type_u32 = match label.label_type {
            fleet_types::LabelType::BuiltIn => 1u32,
            fleet_types::LabelType::Regular => 0u32,
        };
        let membership_type_u32 = match label.label_membership_type {
            fleet_types::label::LabelMembershipType::Dynamic => 0u32,
            fleet_types::label::LabelMembershipType::Manual => 1u32,
            fleet_types::label::LabelMembershipType::HostVitals => 2u32,
        };
        let id = MysqlDatastore::new_label(
            self,
            &label.name,
            &label.description,
            &label.query,
            &label.platform,
            label_type_u32,
            membership_type_u32,
            label.team_id,
        )
        .await
        .map_err(ServiceError::from)?;
        self.label(id).await
    }

    async fn save_label(
        &self,
        label: &fleet_types::Label,
    ) -> ServiceResult<fleet_types::Label> {
        MysqlDatastore::save_label(
            self,
            label.id,
            &label.name,
            &label.description,
            &label.query,
            &label.platform,
        )
        .await
        .map_err(ServiceError::from)?;
        self.label(label.id).await
    }

    async fn delete_label(&self, name: &str) -> ServiceResult<()> {
        MysqlDatastore::delete_label_by_name(self, name)
            .await
            .map_err(ServiceError::from)
    }

    async fn labels_summary(&self) -> ServiceResult<Vec<fleet_types::LabelSummary>> {
        let rows = MysqlDatastore::list_labels(self, None)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows
            .into_iter()
            .map(|row| fleet_types::LabelSummary {
                id: row.id,
                name: row.name,
                description: row.description,
                team_id: row.team_id,
                label_type: match row.label_type {
                    1 => fleet_types::LabelType::BuiltIn,
                    _ => fleet_types::LabelType::Regular,
                },
            })
            .collect())
    }

    async fn record_label_membership(&self, label_id: u32, host_id: u32) -> ServiceResult<()> {
        MysqlDatastore::record_label_membership(self, label_id, host_id)
            .await
            .map_err(ServiceError::from)
    }

    async fn delete_label_membership(&self, label_id: u32, host_id: u32) -> ServiceResult<()> {
        MysqlDatastore::delete_label_membership(self, label_id, host_id)
            .await
            .map_err(ServiceError::from)
    }

    async fn list_labels_for_host(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::Label>> {
        let rows = MysqlDatastore::list_labels_for_host(self, host_id)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(label_row_to_label).collect())
    }

    // ---- Policies ----

    async fn policy(&self, id: u32) -> ServiceResult<fleet_types::Policy> {
        let row = self.policy_by_id(id).await.map_err(ServiceError::from)?;
        Ok(policy_row_to_policy(row))
    }

    async fn list_global_policies(
        &self,
        _opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<fleet_types::Policy>> {
        let rows = MysqlDatastore::list_global_policies(self)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(policy_row_to_policy).collect())
    }

    async fn new_global_policy(
        &self,
        query: &str,
        name: &str,
        description: &str,
    ) -> ServiceResult<fleet_types::Policy> {
        let id = MysqlDatastore::new_global_policy(
            self,
            name,
            query,
            description,
            None,  // resolution
            None,  // author_id
            "",    // platform
            false, // critical
        )
        .await
        .map_err(ServiceError::from)?;
        self.policy(id).await
    }

    async fn save_policy(
        &self,
        policy: &fleet_types::Policy,
    ) -> ServiceResult<fleet_types::Policy> {
        MysqlDatastore::save_policy(
            self,
            policy.policy_data.id,
            &policy.policy_data.name,
            &policy.policy_data.query,
            &policy.policy_data.description,
            policy.policy_data.resolution.as_deref(),
            &policy.policy_data.platform,
            policy.policy_data.critical,
            policy.policy_data.calendar_events_enabled,
        )
        .await
        .map_err(ServiceError::from)?;
        self.policy(policy.policy_data.id).await
    }

    async fn delete_global_policies(&self, ids: &[u32]) -> ServiceResult<Vec<u32>> {
        MysqlDatastore::delete_global_policies(self, ids)
            .await
            .map_err(ServiceError::from)
    }

    async fn list_team_policies(
        &self,
        team_id: u32,
        _opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<fleet_types::Policy>> {
        let rows = MysqlDatastore::list_team_policies(self, team_id)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(policy_row_to_policy).collect())
    }

    async fn new_team_policy(
        &self,
        team_id: u32,
        query: &str,
        name: &str,
        description: &str,
    ) -> ServiceResult<fleet_types::Policy> {
        let id = MysqlDatastore::new_team_policy(
            self,
            team_id,
            name,
            query,
            description,
            None,  // resolution
            None,  // author_id
            "",    // platform
            false, // critical
        )
        .await
        .map_err(ServiceError::from)?;
        self.policy(id).await
    }

    async fn delete_team_policies(
        &self,
        team_id: u32,
        ids: &[u32],
    ) -> ServiceResult<Vec<u32>> {
        MysqlDatastore::delete_team_policies(self, team_id, ids)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Teams ----

    async fn team(&self, id: u32) -> ServiceResult<fleet_types::Team> {
        let row = self.team_by_id(id).await.map_err(ServiceError::from)?;
        Ok(team_row_to_team(row))
    }

    async fn list_teams(
        &self,
        opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<fleet_types::Team>> {
        let match_query = if opts.match_query.is_empty() {
            None
        } else {
            Some(opts.match_query.as_str())
        };
        let rows = MysqlDatastore::list_teams(
            self,
            match_query,
            opts.per_page,
            opts.page * opts.per_page,
        )
        .await
        .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(team_row_to_team).collect())
    }

    async fn new_team(&self, team: &fleet_types::Team) -> ServiceResult<fleet_types::Team> {
        let config_json = serde_json::to_value(&team.config).unwrap_or_default();
        let id = MysqlDatastore::new_team(
            self,
            &team.name,
            team.gitops_filename.as_deref(),
            &team.description,
            &config_json,
        )
        .await
        .map_err(ServiceError::from)?;
        self.team(id).await
    }

    async fn save_team(&self, team: &fleet_types::Team) -> ServiceResult<fleet_types::Team> {
        let config_json = serde_json::to_value(&team.config).unwrap_or_default();
        MysqlDatastore::save_team(self, team.id, &team.name, &team.description, &config_json)
            .await
            .map_err(ServiceError::from)?;
        self.team(team.id).await
    }

    async fn delete_team(&self, id: u32) -> ServiceResult<()> {
        MysqlDatastore::delete_team(self, id)
            .await
            .map_err(ServiceError::from)
    }

    async fn teams_summary(&self) -> ServiceResult<Vec<TeamSummaryInfo>> {
        let rows = MysqlDatastore::list_teams(self, None, 10000, 0)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows
            .into_iter()
            .map(|r| TeamSummaryInfo {
                id: r.id,
                name: r.name,
            })
            .collect())
    }

    async fn list_team_users(&self, team_id: u32) -> ServiceResult<Vec<fleet_types::team::TeamUser>> {
        let rows = self.load_users_for_team(team_id).await.map_err(ServiceError::from)?;
        let now = Utc::now();
        Ok(rows.into_iter().map(|r| {
            fleet_types::team::TeamUser {
                user: fleet_types::User {
                    id: r.id,
                    created_at: now,
                    updated_at: now,
                    password: Vec::new(),
                    salt: String::new(),
                    name: r.name,
                    email: r.email,
                    admin_forced_password_reset: false,
                    gravatar_url: String::new(),
                    position: String::new(),
                    sso_enabled: false,
                    mfa_enabled: false,
                    global_role: None,
                    api_only: false,
                    teams: Vec::new(),
                    settings: None,
                },
                role: r.role,
            }
        }).collect())
    }

    async fn team_enroll_secrets(&self, team_id: u32) -> ServiceResult<Vec<fleet_types::enroll::EnrollSecret>> {
        let rows = self.load_secrets_for_team(team_id).await.map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(|r| fleet_types::enroll::EnrollSecret {
            secret: r.secret,
            team_id: r.team_id,
            created_at: r.created_at,
        }).collect())
    }

    async fn apply_team_enroll_secrets(&self, team_id: u32, secrets: &[String]) -> ServiceResult<()> {
        MysqlDatastore::apply_enroll_secrets(self, Some(team_id), secrets)
            .await
            .map_err(ServiceError::from)
    }

    async fn add_users_to_team(&self, team_id: u32, users: &[(u32, String)]) -> ServiceResult<()> {
        for (user_id, role) in users {
            sqlx::query(
                "INSERT INTO user_teams (user_id, team_id, role) VALUES (?, ?, ?) \
                 ON DUPLICATE KEY UPDATE role = VALUES(role)"
            )
            .bind(user_id)
            .bind(team_id)
            .bind(role)
            .execute(self.pool())
            .await
            .map_err(ds_error)?;
        }
        Ok(())
    }

    async fn remove_users_from_team(&self, team_id: u32, user_ids: &[u32]) -> ServiceResult<()> {
        if user_ids.is_empty() {
            return Ok(());
        }
        let placeholders: Vec<&str> = user_ids.iter().map(|_| "?").collect();
        let sql = format!(
            "DELETE FROM user_teams WHERE team_id = ? AND user_id IN ({})",
            placeholders.join(",")
        );
        let mut query = sqlx::query(&sql).bind(team_id);
        for &id in user_ids {
            query = query.bind(id);
        }
        query.execute(self.pool()).await.map_err(ds_error)?;
        Ok(())
    }

    // ---- AppConfig ----

    async fn app_config(&self) -> ServiceResult<AppConfigData> {
        let val = MysqlDatastore::app_config(self)
            .await
            .map_err(ServiceError::from)?;
        Ok(app_config_json_to_data(val))
    }

    async fn save_app_config(&self, config: &AppConfigData) -> ServiceResult<()> {
        let json = app_config_data_to_json(config);
        MysqlDatastore::save_app_config(self, &json)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Secret Variables ----

    async fn list_secret_variables(&self) -> ServiceResult<Vec<fleet_types::config::SecretVariable>> {
        let rows = MysqlDatastore::list_secret_variables(self)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(sv_row_to_secret_variable).collect())
    }

    async fn create_secret_variable(&self, name: &str, value: &str) -> ServiceResult<fleet_types::config::SecretVariable> {
        let id = MysqlDatastore::create_secret_variable(self, name, value)
            .await
            .map_err(ServiceError::from)?;
        let row = MysqlDatastore::secret_variable_by_id(self, id)
            .await
            .map_err(ServiceError::from)?;
        Ok(sv_row_to_secret_variable(row))
    }

    async fn delete_secret_variable(&self, id: u32) -> ServiceResult<()> {
        MysqlDatastore::delete_secret_variable(self, id)
            .await
            .map_err(ServiceError::from)
    }

    async fn upsert_secret_variables(&self, secrets: &[(String, String)]) -> ServiceResult<()> {
        MysqlDatastore::upsert_secret_variables(self, secrets)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Invites ----

    async fn invite_by_email(&self, email: &str) -> ServiceResult<Option<InviteData>> {
        match MysqlDatastore::invite_by_email(self, email).await {
            Ok(row) => Ok(Some(invite_row_to_invite_data(row))),
            Err(DatastoreError::NotFound { .. }) => Ok(None),
            Err(e) => Err(ServiceError::from(e)),
        }
    }

    async fn invite_by_token(&self, token: &str) -> ServiceResult<InviteData> {
        let row = MysqlDatastore::invite_by_token(self, token)
            .await
            .map_err(ServiceError::from)?;
        Ok(invite_row_to_invite_data(row))
    }

    async fn new_invite(&self, invite: &InviteData) -> ServiceResult<InviteData> {
        let teams: Vec<(u32, String)> = invite
            .teams
            .iter()
            .map(|ut| (ut.team.id, ut.role.clone()))
            .collect();
        let id = MysqlDatastore::new_invite(
            self,
            invite.invited_by,
            &invite.email,
            &invite.name,
            &invite.position,
            &invite.token,
            invite.sso_enabled,
            invite.mfa_enabled,
            invite.global_role.as_deref(),
            &teams,
        )
        .await
        .map_err(ServiceError::from)?;
        let row = self.invite_by_id(id).await.map_err(ServiceError::from)?;
        Ok(invite_row_to_invite_data(row))
    }

    async fn list_invites(
        &self,
        _opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<InviteData>> {
        let rows = MysqlDatastore::list_invites(self, None)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(invite_row_to_invite_data).collect())
    }

    async fn delete_invite(&self, id: u32) -> ServiceResult<()> {
        MysqlDatastore::delete_invite(self, id)
            .await
            .map_err(ServiceError::from)
    }

    async fn update_invite(&self, id: u32, invite: &InviteData) -> ServiceResult<InviteData> {
        let teams: Vec<(u32, String)> = invite
            .teams
            .iter()
            .map(|ut| (ut.team.id, ut.role.clone()))
            .collect();
        MysqlDatastore::update_invite(
            self,
            id,
            invite.invited_by,
            &invite.email,
            &invite.name,
            &invite.position,
            invite.sso_enabled,
            invite.mfa_enabled,
            invite.global_role.as_deref(),
            &teams,
        )
        .await
        .map_err(ServiceError::from)?;
        let row = self.invite_by_id(id).await.map_err(ServiceError::from)?;
        Ok(invite_row_to_invite_data(row))
    }

    // ---- Enroll Secrets ----

    async fn verify_enroll_secret(&self, secret: &str) -> ServiceResult<EnrollSecretInfo> {
        let row = MysqlDatastore::verify_enroll_secret(self, secret)
            .await
            .map_err(ServiceError::from)?;
        Ok(EnrollSecretInfo {
            secret: row.secret,
            team_id: row.team_id,
        })
    }

    async fn get_enroll_secrets(&self, team_id: Option<u32>) -> ServiceResult<Vec<fleet_types::enroll::EnrollSecret>> {
        let rows = MysqlDatastore::get_enroll_secrets(self, team_id)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(|r| fleet_types::enroll::EnrollSecret {
            secret: r.secret,
            team_id: r.team_id,
            created_at: r.created_at,
        }).collect())
    }

    async fn apply_enroll_secrets(&self, team_id: Option<u32>, secrets: &[fleet_types::enroll::EnrollSecret]) -> ServiceResult<()> {
        let secret_strings: Vec<String> = secrets.iter().map(|s| s.secret.clone()).collect();
        MysqlDatastore::apply_enroll_secrets(self, team_id, &secret_strings)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Password Reset ----

    async fn new_password_reset_request(
        &self,
        user_id: u32,
        expires_at: DateTime<Utc>,
        token: &str,
    ) -> ServiceResult<()> {
        MysqlDatastore::new_password_reset_request(self, user_id, expires_at, token)
            .await
            .map_err(ServiceError::from)
    }

    async fn find_password_reset_by_token(
        &self,
        token: &str,
    ) -> ServiceResult<PasswordResetRequest> {
        let row = MysqlDatastore::find_password_reset_by_token(self, token)
            .await
            .map_err(ServiceError::from)?;
        Ok(PasswordResetRequest {
            id: row.id,
            user_id: row.user_id,
            token: row.token,
            expires_at: row.expires_at,
        })
    }

    async fn delete_password_reset_requests_for_user(&self, user_id: u32) -> ServiceResult<()> {
        MysqlDatastore::delete_password_reset_requests_for_user(self, user_id)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Software ----

    async fn list_software(
        &self,
        opts: fleet_types::ListOptions,
        team_id: Option<u32>,
    ) -> ServiceResult<Vec<fleet_types::Software>> {
        let rows = MysqlDatastore::list_software(self, team_id, opts.per_page, opts.page)
            .await
            .map_err(ServiceError::from)?;
        let mut software: Vec<fleet_types::Software> = rows.into_iter().map(software_row_to_software).collect();
        enrich_software_with_cves(self, &mut software).await;
        Ok(software)
    }

    async fn software_by_id(&self, id: u32) -> ServiceResult<fleet_types::Software> {
        let row = MysqlDatastore::software_by_id(self, id)
            .await
            .map_err(ServiceError::from)?;
        let mut sw = software_row_to_software(row);
        enrich_software_with_cves(self, std::slice::from_mut(&mut sw)).await;
        Ok(sw)
    }

    async fn update_software_title_name(&self, id: u32, name: &str) -> ServiceResult<()> {
        MysqlDatastore::update_software_title_name(self, id, name)
            .await
            .map_err(ServiceError::from)
    }

    async fn delete_software_installer(&self, title_id: u32) -> ServiceResult<()> {
        MysqlDatastore::delete_software_installer(self, title_id)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Email Changes ----

    async fn confirm_pending_email_change(&self, user_id: u32, token: &str) -> ServiceResult<String> {
        // Find the pending email change record
        let row: Option<(u32, String)> = sqlx::query_as(
            "SELECT id, new_email FROM email_changes WHERE token = ? AND user_id = ?",
        )
        .bind(token)
        .bind(user_id)
        .fetch_optional(self.pool())
        .await
        .map_err(ds_error)?;

        let (change_id, new_email) = row.ok_or_else(|| {
            ServiceError::not_found("email change with token not found")
        })?;

        // Update the user's email
        sqlx::query("UPDATE users SET email = ? WHERE id = ?")
            .bind(&new_email)
            .bind(user_id)
            .execute(self.pool())
            .await
            .map_err(ds_error)?;

        // Delete the email change record
        sqlx::query("DELETE FROM email_changes WHERE id = ?")
            .bind(change_id)
            .execute(self.pool())
            .await
            .map_err(ds_error)?;

        Ok(new_email)
    }

    // ---- Batch User Operations ----

    async fn save_users(&self, users: &[fleet_types::User]) -> ServiceResult<()> {
        for user in users {
            Datastore::save_user(self, user).await?;
        }
        Ok(())
    }

    async fn team_by_name(&self, name: &str) -> ServiceResult<fleet_types::Team> {
        let row = MysqlDatastore::team_by_name(self, name)
            .await
            .map_err(ServiceError::from)?;
        Ok(team_row_to_team(row))
    }

    // ---- Orbit ----

    async fn load_host_by_orbit_node_key(&self, orbit_node_key: &str) -> ServiceResult<fleet_types::Host> {
        let row = sqlx::query_as::<_, crate::hosts::HostRow>(
            "SELECT h.* FROM hosts h JOIN host_orbit_info hoi ON h.id = hoi.host_id WHERE hoi.orbit_node_key = ?"
        )
        .bind(orbit_node_key)
        .fetch_optional(self.pool())
        .await
        .map_err(ds_error)?
        .ok_or_else(|| ServiceError::not_found("host not found for orbit node key"))?;
        Ok(host_row_to_host(row))
    }

    async fn enroll_orbit(
        &self,
        hardware_uuid: &str,
        hardware_serial: &str,
        orbit_node_key: &str,
        _team_id: Option<u32>,
    ) -> ServiceResult<fleet_types::Host> {
        // Try to find existing host by hardware UUID or serial
        let existing = sqlx::query_as::<_, crate::hosts::HostRow>(
            "SELECT * FROM hosts WHERE uuid = ? OR hardware_serial = ? LIMIT 1"
        )
        .bind(hardware_uuid)
        .bind(hardware_serial)
        .fetch_optional(self.pool())
        .await
        .map_err(ds_error)?;

        match existing {
            Some(row) => {
                let host = host_row_to_host(row);
                // Update orbit node key
                sqlx::query(
                    "INSERT INTO host_orbit_info (host_id, orbit_node_key) VALUES (?, ?) ON DUPLICATE KEY UPDATE orbit_node_key = VALUES(orbit_node_key)"
                )
                .bind(host.id)
                .bind(orbit_node_key)
                .execute(self.pool())
                .await
                .map_err(ds_error)?;
                Ok(host)
            }
            None => {
                Err(ServiceError::not_found("no matching host found for orbit enrollment"))
            }
        }
    }

    async fn set_orbit_node_key(&self, host_id: u32, orbit_node_key: &str) -> ServiceResult<()> {
        sqlx::query(
            "INSERT INTO host_orbit_info (host_id, orbit_node_key) VALUES (?, ?) ON DUPLICATE KEY UPDATE orbit_node_key = VALUES(orbit_node_key)"
        )
        .bind(host_id)
        .bind(orbit_node_key)
        .execute(self.pool())
        .await
        .map_err(ds_error)?;
        Ok(())
    }

    async fn get_host_script_execution(&self, execution_id: &str) -> ServiceResult<fleet_types::script::HostScriptResult> {
        let query = r#"
            SELECT
                hsr.id, hsr.host_id, hsr.execution_id,
                COALESCE(sc.contents, '') as script_contents,
                hsr.script_id,
                COALESCE(hsr.output, '') as output,
                COALESCE(hsr.runtime, 0) as runtime,
                hsr.exit_code,
                hsr.timeout as host_timeout,
                hsr.host_deleted_at,
                hsr.created_at,
                hsr.updated_at
            FROM host_script_results hsr
            LEFT JOIN script_contents sc ON sc.id = hsr.script_content_id
            WHERE hsr.execution_id = ?
        "#;

        let row = sqlx::query_as::<_, HostScriptResultRow>(query)
            .bind(execution_id)
            .fetch_optional(self.pool())
            .await
            .map_err(ds_error)?
            .ok_or_else(|| ServiceError::not_found("script execution not found"))?;

        Ok(host_script_result_row_to_result(row))
    }

    async fn save_host_script_result(&self, result: &fleet_types::script::HostScriptResult) -> ServiceResult<()> {
        // Truncate output to 10000 chars like Go does
        let output = if result.output.len() > 10000 {
            &result.output[..10000]
        } else {
            &result.output
        };

        sqlx::query(
            r#"UPDATE host_script_results SET
                output = ?,
                runtime = ?,
                exit_code = ?,
                timeout = ?
            WHERE host_id = ? AND execution_id = ?"#,
        )
        .bind(output)
        .bind(result.runtime)
        .bind(result.exit_code)
        .bind(result.host_timeout)
        .bind(result.host_id)
        .bind(&result.execution_id)
        .execute(self.pool())
        .await
        .map_err(ds_error)?;

        Ok(())
    }

    async fn set_host_disk_encryption_key(&self, host_id: u32, key: &[u8], client_error: Option<&str>) -> ServiceResult<()> {
        sqlx::query(
            "INSERT INTO host_disk_encryption_keys (host_id, base64_encrypted, client_error) VALUES (?, ?, ?) ON DUPLICATE KEY UPDATE base64_encrypted = VALUES(base64_encrypted), client_error = VALUES(client_error)"
        )
        .bind(host_id)
        .bind(key)
        .bind(client_error.unwrap_or(""))
        .execute(self.pool())
        .await
        .map_err(ds_error)?;
        Ok(())
    }

    // ---- Activities ----

    async fn new_activity(
        &self,
        user_id: Option<u32>,
        activity_type: &str,
        details: &JsonValue,
    ) -> ServiceResult<()> {
        MysqlDatastore::new_activity(self, user_id, None, None, activity_type, details)
            .await
            .map_err(ServiceError::from)?;
        Ok(())
    }

    async fn list_activities(&self, limit: u32, offset: u32) -> ServiceResult<Vec<fleet_types::Activity>> {
        let rows = MysqlDatastore::list_activities(self, limit, offset)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(activity_row_to_activity).collect())
    }

    async fn count_host_upcoming_activities(&self, host_id: u32) -> ServiceResult<u32> {
        MysqlDatastore::count_host_upcoming_activities(self, host_id)
            .await
            .map_err(ServiceError::from)
    }

    async fn list_host_upcoming_activities(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::UpcomingActivity>> {
        let rows = MysqlDatastore::list_host_upcoming_activities(self, host_id)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(|r| fleet_types::UpcomingActivity {
            id: r.id,
            host_id: r.host_id,
            user_id: r.user_id,
            activity_type: r.activity_type,
            execution_id: r.execution_id,
            created_at: r.created_at,
            activated_at: r.activated_at,
            fleet_initiated: r.fleet_initiated,
            priority: r.priority,
        }).collect())
    }

    async fn delete_host_upcoming_activity(&self, host_id: u32, activity_id: u32) -> ServiceResult<()> {
        MysqlDatastore::delete_host_upcoming_activity(self, host_id, activity_id)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Device ----

    async fn load_host_by_device_auth_token(&self, token: &str) -> ServiceResult<fleet_types::Host> {
        let row = sqlx::query_as::<_, crate::hosts::HostRow>(
            "SELECT h.* FROM hosts h JOIN host_device_auth hda ON h.id = hda.host_id WHERE hda.token = ?"
        )
        .bind(token)
        .fetch_optional(self.pool())
        .await
        .map_err(ds_error)?
        .ok_or_else(|| ServiceError::not_found("host not found for device token"))?;
        Ok(host_row_to_host(row))
    }

    async fn set_or_update_device_auth_token(&self, host_id: u32, token: &str) -> ServiceResult<()> {
        sqlx::query(
            "INSERT INTO host_device_auth (host_id, token) VALUES (?, ?) ON DUPLICATE KEY UPDATE token = VALUES(token)"
        )
        .bind(host_id)
        .bind(token)
        .execute(self.pool())
        .await
        .map_err(ds_error)?;
        Ok(())
    }

    async fn list_policies_for_host(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::policy::HostPolicy>> {
        // Get the host's platform to filter policies by platform
        let host = self.host_by_id(host_id).await.map_err(ServiceError::from)?;
        let fleet_platform = fleet_platform_from_host(&host.platform);

        let query = r#"
            SELECT p.id, p.team_id, COALESCE(p.resolution, '') as resolution,
                   p.name, p.query, p.description, p.author_id,
                   COALESCE(p.platforms, '') as platform,
                   p.critical, p.created_at, p.updated_at,
                   p.conditional_access_enabled,
                   p.conditional_access_bypass_enabled,
                   COALESCE(u.name, '<deleted>') AS author_name,
                   COALESCE(u.email, '') AS author_email,
                   CASE
                       WHEN pm.passes = 1 THEN 'pass'
                       WHEN pm.passes = 0 THEN 'fail'
                       ELSE ''
                   END AS response
            FROM policies p
            LEFT JOIN policy_membership pm ON (p.id = pm.policy_id AND pm.host_id = ?)
            LEFT JOIN users u ON p.author_id = u.id
            WHERE (p.team_id IS NULL OR p.team_id = COALESCE((SELECT team_id FROM hosts WHERE id = ?), 0))
            AND (p.platforms IS NULL OR p.platforms = '' OR FIND_IN_SET(?, p.platforms) != 0)
            ORDER BY FIELD(response, 'fail', '', 'pass'), p.name
        "#;

        let rows = sqlx::query_as::<_, HostPolicyRow>(query)
            .bind(host_id)
            .bind(host_id)
            .bind(&fleet_platform)
            .fetch_all(self.pool())
            .await
            .map_err(ds_error)?;

        Ok(rows.into_iter().map(host_policy_row_to_host_policy).collect())
    }

    async fn list_software_for_host(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::Software>> {
        let query = r#"
            SELECT
                s.id, s.name, s.version, s.source,
                COALESCE(s.extension_for, '') as extension_for,
                COALESCE(s.bundle_identifier, '') as bundle_identifier,
                COALESCE(s.`release`, '') as `release`,
                COALESCE(s.vendor, '') as vendor,
                COALESCE(s.arch, '') as arch,
                COALESCE(s.extension_id, '') as extension_id,
                s.upgrade_code,
                hs.last_opened_at
            FROM software s
            JOIN host_software hs ON hs.software_id = s.id
            WHERE hs.host_id = ?
        "#;

        let rows = sqlx::query_as::<_, SoftwareForHostRow>(query)
            .bind(host_id)
            .fetch_all(self.pool())
            .await
            .map_err(ds_error)?;

        let mut software: Vec<fleet_types::Software> = rows.into_iter().map(software_for_host_row_to_software).collect();
        enrich_software_with_cves(self, &mut software).await;
        Ok(software)
    }

    async fn device_mapping_for_host(&self, host_id: u32) -> ServiceResult<serde_json::Value> {
        let query = r#"
            SELECT
                id, host_id, email,
                CASE
                    WHEN source LIKE 'custom_%' THEN 'custom'
                    WHEN source = 'idp' THEN 'mdm_idp_accounts'
                    ELSE source
                END as source
            FROM host_emails
            WHERE host_id = ?
            ORDER BY email, source
        "#;

        let rows = sqlx::query_as::<_, DeviceMappingRow>(query)
            .bind(host_id)
            .fetch_all(self.pool())
            .await
            .map_err(ds_error)?;

        let mappings: Vec<serde_json::Value> = rows
            .into_iter()
            .map(|r| {
                serde_json::json!({
                    "id": r.id,
                    "host_id": r.host_id,
                    "email": r.email,
                    "source": r.source,
                })
            })
            .collect();

        Ok(serde_json::Value::Array(mappings))
    }

    async fn set_custom_host_device_mapping(&self, host_id: u32, email: &str) -> ServiceResult<()> {
        sqlx::query(
            "INSERT INTO host_emails (host_id, email, source) VALUES (?, ?, 'custom') \
             ON DUPLICATE KEY UPDATE email = VALUES(email)"
        )
        .bind(host_id)
        .bind(email)
        .execute(self.pool())
        .await
        .map_err(ds_error)?;
        Ok(())
    }

    async fn mark_host_refetch_requested(&self, host_id: u32) -> ServiceResult<()> {
        sqlx::query("UPDATE hosts SET refetch_requested = 1 WHERE id = ?")
            .bind(host_id)
            .execute(self.pool())
            .await
            .map_err(ds_error)?;
        Ok(())
    }

    // ---- Carves ----

    async fn new_carve(&self, carve: &fleet_types::CarveMetadata) -> ServiceResult<fleet_types::CarveMetadata> {
        let row = crate::carves::CarveRow {
            id: 0,
            host_id: carve.host_id,
            created_at: carve.created_at,
            name: carve.name.clone(),
            block_count: carve.block_count,
            block_size: carve.block_size,
            carve_size: carve.carve_size,
            carve_id: carve.carve_id.clone(),
            request_id: carve.request_id.clone(),
            session_id: carve.session_id.clone(),
            expired: carve.expired,
            max_block: -1,
            error: carve.error.clone(),
        };
        let result = MysqlDatastore::new_carve(self, &row)
            .await
            .map_err(ServiceError::from)?;
        Ok(carve_row_to_carve(result))
    }

    async fn carve_by_id(&self, id: i64) -> ServiceResult<fleet_types::CarveMetadata> {
        let row = MysqlDatastore::carve_by_id(self, id)
            .await
            .map_err(ServiceError::from)?;
        Ok(carve_row_to_carve(row))
    }

    async fn carve_by_session_id(&self, session_id: &str) -> ServiceResult<fleet_types::CarveMetadata> {
        let row = MysqlDatastore::carve_by_session_id(self, session_id)
            .await
            .map_err(ServiceError::from)?;
        Ok(carve_row_to_carve(row))
    }

    async fn list_carves(&self, include_expired: bool) -> ServiceResult<Vec<fleet_types::CarveMetadata>> {
        let rows = MysqlDatastore::list_carves(self, include_expired)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(carve_row_to_carve).collect())
    }

    async fn update_carve(&self, id: i64, max_block: i64, expired: bool, error: Option<&str>) -> ServiceResult<()> {
        MysqlDatastore::update_carve(self, id, max_block, expired, error)
            .await
            .map_err(ServiceError::from)
    }

    async fn new_carve_block(&self, metadata_id: i64, block_id: i64, data: &[u8]) -> ServiceResult<()> {
        MysqlDatastore::new_carve_block(self, metadata_id, block_id, data)
            .await
            .map_err(ServiceError::from)
    }

    async fn get_carve_block(&self, metadata_id: i64, block_id: i64) -> ServiceResult<Vec<u8>> {
        MysqlDatastore::get_carve_block(self, metadata_id, block_id)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Scripts ----

    async fn new_script(&self, team_id: Option<u32>, name: &str, contents: &str) -> ServiceResult<fleet_types::script::Script> {
        let row = MysqlDatastore::new_script(self, team_id, name, contents)
            .await
            .map_err(ServiceError::from)?;
        Ok(script_row_to_script(row))
    }

    async fn script_by_id(&self, id: u32) -> ServiceResult<fleet_types::script::Script> {
        let row = MysqlDatastore::script_by_id(self, id)
            .await
            .map_err(ServiceError::from)?;
        Ok(script_row_to_script(row))
    }

    async fn list_scripts(&self, team_id: Option<u32>) -> ServiceResult<Vec<fleet_types::script::Script>> {
        let rows = MysqlDatastore::list_scripts(self, team_id)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(script_row_to_script).collect())
    }

    async fn delete_script(&self, id: u32) -> ServiceResult<()> {
        MysqlDatastore::delete_script(self, id)
            .await
            .map_err(ServiceError::from)
    }

    async fn get_script_contents(&self, script_id: u32) -> ServiceResult<String> {
        MysqlDatastore::get_script_contents(self, script_id)
            .await
            .map_err(ServiceError::from)
    }

    // ---- Utilities ----

    async fn list_packs_for_host(&self, host_id: u32) -> ServiceResult<Vec<fleet_types::Pack>> {
        let rows = MysqlDatastore::list_packs_for_host(self, host_id)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(pack_row_to_pack).collect())
    }

    async fn list_software_titles(&self, team_id: Option<u32>, limit: u32, offset: u32) -> ServiceResult<Vec<fleet_types::Software>> {
        let rows = MysqlDatastore::list_software_titles(self, team_id, limit, offset)
            .await
            .map_err(ServiceError::from)?;
        Ok(rows.into_iter().map(software_title_row_to_software).collect())
    }
}
