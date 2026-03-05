//! AppConfig service operations.
//!
//! Implements reading and modifying the global application configuration.
//! Corresponds to Go's `server/service/appconfig.go`.

use tracing::info;

use crate::authz::{self, Action, Subject};
use crate::fleet_service::{AppConfigData, FleetService};
use crate::{ServiceResult, Viewer};

/// Obfuscated value used to mask sensitive fields in API responses.
pub const OBFUSCATED_VALUE: &str = "********";

impl FleetService {
    /// Returns the application configuration.
    ///
    /// Corresponds to Go's `(svc *Service) AppConfig`.
    pub async fn app_config(&self, viewer: &Viewer) -> ServiceResult<AppConfigData> {
        authz::authorize(viewer, Subject::AppConfig, Action::Read)?;
        self.ds.app_config().await
    }

    /// Returns the application configuration with sensitive fields obfuscated.
    ///
    /// Corresponds to Go's `(svc *Service) AppConfigObfuscated`.
    pub async fn app_config_obfuscated(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<AppConfigData> {
        let mut config = self.app_config(viewer).await?;
        obfuscate_config(&mut config);
        Ok(config)
    }

    /// Modifies the application configuration.
    ///
    /// Corresponds to Go's `(svc *Service) ModifyAppConfig`.
    pub async fn modify_app_config(
        &self,
        viewer: &Viewer,
        payload: ModifyAppConfigPayload,
    ) -> ServiceResult<AppConfigData> {
        authz::authorize(viewer, Subject::AppConfig, Action::Write)?;

        let mut config = self.ds.app_config().await?;

        // Apply modifications.
        if let Some(org_name) = payload.org_name {
            config.org_name = org_name;
        }
        if let Some(org_logo_url) = payload.org_logo_url {
            config.org_logo_url = org_logo_url;
        }
        if let Some(server_url) = payload.server_url {
            config.server_url = server_url;
        }
        if let Some(live_query_disabled) = payload.live_query_disabled {
            config.live_query_disabled = live_query_disabled;
        }
        if let Some(enable_sso) = payload.enable_sso {
            config.enable_sso = enable_sso;
        }
        if let Some(sso_entity_id) = payload.sso_entity_id {
            config.sso_entity_id = sso_entity_id;
        }
        if let Some(sso_idp_name) = payload.sso_idp_name {
            config.sso_idp_name = sso_idp_name;
        }
        if let Some(sso_metadata) = payload.sso_metadata {
            config.sso_metadata = sso_metadata;
        }
        if let Some(sso_metadata_url) = payload.sso_metadata_url {
            config.sso_metadata_url = sso_metadata_url;
        }
        if let Some(smtp_configured) = payload.smtp_configured {
            config.smtp_configured = smtp_configured;
        }
        if let Some(smtp_sender_address) = payload.smtp_sender_address {
            config.smtp_sender_address = smtp_sender_address;
        }
        if let Some(smtp_server) = payload.smtp_server {
            config.smtp_server = smtp_server;
        }
        if let Some(smtp_port) = payload.smtp_port {
            config.smtp_port = smtp_port;
        }
        if let Some(smtp_enable_ssl_tls) = payload.smtp_enable_ssl_tls {
            config.smtp_enable_ssl_tls = smtp_enable_ssl_tls;
        }
        if let Some(smtp_user_name) = payload.smtp_user_name {
            config.smtp_user_name = smtp_user_name;
        }
        if let Some(smtp_password) = payload.smtp_password {
            // Only update if it is not the obfuscated value.
            if smtp_password != OBFUSCATED_VALUE {
                config.smtp_password = smtp_password;
            }
        }
        if let Some(host_expiry_enabled) = payload.host_expiry_enabled {
            config.host_expiry_enabled = host_expiry_enabled;
        }
        if let Some(host_expiry_window) = payload.host_expiry_window {
            config.host_expiry_window = host_expiry_window;
        }
        if let Some(agent_options) = payload.agent_options {
            config.agent_options = Some(agent_options);
        }
        if let Some(transparency_url) = payload.transparency_url {
            config.transparency_url = transparency_url;
        }
        if let Some(enable_host_users) = payload.enable_host_users {
            config.enable_host_users = enable_host_users;
        }
        if let Some(enable_software_inventory) = payload.enable_software_inventory {
            config.enable_software_inventory = enable_software_inventory;
        }

        self.ds.save_app_config(&config).await?;

        info!("app config modified");
        Ok(config)
    }
}

/// Obfuscates sensitive fields in the app configuration for API responses.
fn obfuscate_config(config: &mut AppConfigData) {
    if !config.smtp_password.is_empty() {
        config.smtp_password = OBFUSCATED_VALUE.to_string();
    }
}

/// Payload for modifying application configuration.
#[derive(Debug, Clone, Default)]
pub struct ModifyAppConfigPayload {
    pub org_name: Option<String>,
    pub org_logo_url: Option<String>,
    pub server_url: Option<String>,
    pub live_query_disabled: Option<bool>,
    pub enable_sso: Option<bool>,
    pub sso_entity_id: Option<String>,
    pub sso_idp_name: Option<String>,
    pub sso_metadata: Option<String>,
    pub sso_metadata_url: Option<String>,
    pub smtp_configured: Option<bool>,
    pub smtp_sender_address: Option<String>,
    pub smtp_server: Option<String>,
    pub smtp_port: Option<u16>,
    pub smtp_enable_ssl_tls: Option<bool>,
    pub smtp_user_name: Option<String>,
    pub smtp_password: Option<String>,
    pub host_expiry_enabled: Option<bool>,
    pub host_expiry_window: Option<i64>,
    pub agent_options: Option<serde_json::Value>,
    pub transparency_url: Option<String>,
    pub enable_host_users: Option<bool>,
    pub enable_software_inventory: Option<bool>,
}
