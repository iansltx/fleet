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

fn app_config_json_to_data(val: serde_json::Value) -> AppConfigData {
    serde_json::from_value(val).unwrap_or_default()
}

fn app_config_data_to_json(data: &AppConfigData) -> serde_json::Value {
    serde_json::to_value(data).unwrap_or_default()
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
            online_count: Some(0),
            offline_count: Some(0),
            mia_count: Some(0),
            missing_30_days_count: None,
            new_count: Some(0),
            platforms: vec![
                fleet_types::host::HostSummaryPlatform { platform: "linux".to_string(), hosts_count: row.linux_count as u32 },
                fleet_types::host::HostSummaryPlatform { platform: "darwin".to_string(), hosts_count: row.macos_count as u32 },
                fleet_types::host::HostSummaryPlatform { platform: "windows".to_string(), hosts_count: row.windows_count as u32 },
                fleet_types::host::HostSummaryPlatform { platform: "chrome".to_string(), hosts_count: row.chrome_count as u32 },
            ],
        })
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
        // The datastore has list_software_titles but no generic list_software yet.
        // Return empty list for now.
        Ok(Vec::new())
    }

    async fn software_by_id(&self, id: u32) -> ServiceResult<fleet_types::Software> {
        let row = MysqlDatastore::software_by_id(self, id)
            .await
            .map_err(ServiceError::from)?;
        Ok(software_row_to_software(row))
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
}
