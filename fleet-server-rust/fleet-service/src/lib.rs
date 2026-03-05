//! Fleet Service - Business logic layer for the Fleet server.
//!
//! This crate implements the service layer, ported from Go's `server/service/`.
//! It contains all business logic for user management, host management,
//! osquery enrollment, query scheduling, policy enforcement, and more.

pub mod activities;
pub mod app_config;
pub mod auth;
pub mod authz;
pub mod carves;
pub mod certificates;
pub mod device;
pub mod fleet_service;
pub mod hosts;
pub mod invites;
pub mod labels;
pub mod live_query;
pub mod orbit;
pub mod osquery;
pub mod packs;
pub mod policies;
pub mod queries;
pub mod scripts;
pub mod sessions;
pub mod setup_experience;
pub mod software;
pub mod teams;
pub mod users;

pub use fleet_service::FleetService;

use thiserror::Error;

/// ServiceError covers all errors that can arise from the service layer.
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("not found: {0}")]
    NotFound(String),

    #[error("unauthorized: {0}")]
    Unauthorized(String),

    #[error("forbidden: {0}")]
    Forbidden(String),

    #[error("bad request: {0}")]
    BadRequest(String),

    #[error("conflict: {0}")]
    Conflict(String),

    #[error("missing license")]
    MissingLicense,

    #[error("auth failed: {0}")]
    AuthFailed(String),

    #[error("auth required: {0}")]
    AuthRequired(String),

    #[error("password reset required")]
    PasswordResetRequired,

    #[error("validation error: {field}: {message}")]
    InvalidArgument { field: String, message: String },

    #[error("internal error: {0}")]
    Internal(String),

    #[error(transparent)]
    Anyhow(#[from] anyhow::Error),
}

impl ServiceError {
    pub fn not_found(msg: impl Into<String>) -> Self {
        Self::NotFound(msg.into())
    }

    pub fn bad_request(msg: impl Into<String>) -> Self {
        Self::BadRequest(msg.into())
    }

    pub fn invalid_argument(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self::InvalidArgument {
            field: field.into(),
            message: message.into(),
        }
    }

    pub fn auth_failed(msg: impl Into<String>) -> Self {
        Self::AuthFailed(msg.into())
    }

    pub fn auth_required(msg: impl Into<String>) -> Self {
        Self::AuthRequired(msg.into())
    }

    pub fn internal(msg: impl Into<String>) -> Self {
        Self::Internal(msg.into())
    }
}

pub type ServiceResult<T> = Result<T, ServiceError>;

/// Viewer represents the authenticated context for a request, containing the
/// user and their active session.
#[derive(Debug, Clone)]
pub struct Viewer {
    pub user: fleet_types::User,
    pub session: fleet_types::Session,
}

impl Viewer {
    pub fn user_id(&self) -> u32 {
        self.user.id
    }

    pub fn session_id(&self) -> u32 {
        self.session.id
    }

    /// Returns true if the user has admin global role.
    pub fn is_global_admin(&self) -> bool {
        self.user.is_global_admin()
    }

    /// Returns true if the user can perform write operations based on their role.
    pub fn can_perform_write_actions(&self) -> bool {
        if let Some(ref role) = self.user.global_role {
            matches!(role.as_str(), "admin" | "maintainer" | "gitops")
        } else {
            self.user
                .teams
                .iter()
                .any(|t| matches!(t.role.as_str(), "admin" | "maintainer" | "gitops"))
        }
    }

    /// Returns true if the user can perform actions on the given team.
    pub fn can_access_team(&self, team_id: u32) -> bool {
        if self.user.global_role.is_some() {
            return true;
        }
        self.user.teams.iter().any(|t| t.team.id == team_id)
    }
}

/// Configuration for the Fleet service, matching the Go FleetConfig.
#[derive(Debug, Clone)]
pub struct FleetServiceConfig {
    pub server: ServerConfig,
    pub session: SessionConfig,
    pub osquery: OsqueryConfig,
    pub auth: AuthConfig,
    pub app: AppServiceConfig,
}

impl Default for FleetServiceConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            session: SessionConfig::default(),
            osquery: OsqueryConfig::default(),
            auth: AuthConfig::default(),
            app: AppServiceConfig::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ServerConfig {
    pub url_prefix: String,
    pub server_url: String,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url_prefix: String::new(),
            server_url: "https://localhost:8080".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct SessionConfig {
    pub key_size: usize,
    pub duration: std::time::Duration,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            key_size: 64,
            duration: std::time::Duration::from_secs(4 * 60 * 60),
        }
    }
}

#[derive(Debug, Clone)]
pub struct OsqueryConfig {
    pub node_key_size: usize,
    pub host_identifier: String,
    pub enroll_cooldown: std::time::Duration,
    pub label_update_interval: std::time::Duration,
    pub policy_update_interval: std::time::Duration,
    pub detail_update_interval: std::time::Duration,
}

impl Default for OsqueryConfig {
    fn default() -> Self {
        Self {
            node_key_size: 24,
            host_identifier: "provided".to_string(),
            enroll_cooldown: std::time::Duration::from_secs(0),
            label_update_interval: std::time::Duration::from_secs(3600),
            policy_update_interval: std::time::Duration::from_secs(3600),
            detail_update_interval: std::time::Duration::from_secs(3600),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AuthConfig {
    pub jwt_key: String,
    pub bcrypt_cost: u32,
    pub salt_key_size: usize,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            jwt_key: String::new(),
            bcrypt_cost: bcrypt::DEFAULT_COST,
            salt_key_size: 24,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppServiceConfig {
    pub token_key_size: usize,
}

impl Default for AppServiceConfig {
    fn default() -> Self {
        Self {
            token_key_size: 24,
        }
    }
}
