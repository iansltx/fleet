//! Error types matching Go's `server/fleet/errors.go`.

use serde::{Deserialize, Serialize};
use std::fmt;

/// FleetError is the main error type for the Fleet server.
#[derive(Debug, thiserror::Error)]
pub enum FleetError {
    #[error("not found: {resource} {identifier}")]
    NotFound {
        resource: String,
        identifier: String,
    },

    #[error("already exists: {resource} {identifier}")]
    AlreadyExists {
        resource: String,
        identifier: String,
    },

    #[error("validation failed: {0}")]
    InvalidArgument(InvalidArgumentError),

    #[error("unauthorized")]
    Unauthorized,

    #[error("forbidden")]
    Forbidden,

    #[error("no context")]
    NoContext,

    #[error("password reset required")]
    PasswordResetRequired,

    #[error("missing license")]
    MissingLicense,

    #[error("MDM not configured")]
    MDMNotConfigured,

    #[error("Windows MDM not configured")]
    WindowsMDMNotConfigured,

    #[error("Android MDM not configured")]
    AndroidMDMNotConfigured,

    #[error("gateway error: {message}")]
    GatewayError { message: String, code: u16 },

    #[error("permission error: {message}")]
    PermissionError { message: String },

    #[error("not configured: {message}")]
    NotConfigured { message: String },

    #[error("conflict: {message}")]
    Conflict { message: String },

    #[error("bad request: {message}")]
    BadRequest { message: String },

    #[error("internal error: {0}")]
    Internal(String),

    #[error("database error: {0}")]
    Database(String),
}

impl FleetError {
    /// Returns the HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            FleetError::NotFound { .. } => 404,
            FleetError::AlreadyExists { .. } => 409,
            FleetError::InvalidArgument(_) => 422,
            FleetError::Unauthorized => 401,
            FleetError::Forbidden => 403,
            FleetError::PasswordResetRequired => 403,
            FleetError::MissingLicense => 402,
            FleetError::MDMNotConfigured
            | FleetError::WindowsMDMNotConfigured
            | FleetError::AndroidMDMNotConfigured
            | FleetError::NotConfigured { .. } => 422,
            FleetError::GatewayError { code, .. } => *code,
            FleetError::PermissionError { .. } => 403,
            FleetError::Conflict { .. } => 409,
            FleetError::BadRequest { .. } => 400,
            FleetError::Internal(_) | FleetError::Database(_) => 500,
            FleetError::NoContext => 500,
        }
    }

    /// Returns true if this is a client error.
    pub fn is_client_error(&self) -> bool {
        let code = self.status_code();
        (400..500).contains(&code)
    }
}

/// InvalidArgument describes a single invalid argument.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvalidArgument {
    pub name: String,
    pub reason: String,
}

/// InvalidArgumentError holds one or more invalid argument details.
#[derive(Debug, Clone, Default)]
pub struct InvalidArgumentError {
    pub errors: Vec<InvalidArgument>,
}

impl InvalidArgumentError {
    pub fn new(name: &str, reason: &str) -> Self {
        let mut e = Self::default();
        e.append(name, reason);
        e
    }

    pub fn append(&mut self, name: &str, reason: &str) {
        self.errors.push(InvalidArgument {
            name: name.to_string(),
            reason: reason.to_string(),
        });
    }

    pub fn has_errors(&self) -> bool {
        !self.errors.is_empty()
    }
}

impl fmt::Display for InvalidArgumentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.errors.len() {
            0 => write!(f, "validation failed"),
            1 => write!(
                f,
                "validation failed: {} {}",
                self.errors[0].name, self.errors[0].reason
            ),
            n => write!(
                f,
                "validation failed: {} {} and {} other errors",
                self.errors[0].name,
                self.errors[0].reason,
                n - 1
            ),
        }
    }
}

impl std::error::Error for InvalidArgumentError {}

// Well-known error messages (matching Go constants)
pub const MDM_NOT_CONFIGURED_MESSAGE: &str =
    "MDM features aren't turned on in Fleet. For more information about setting up MDM, please visit https://fleetdm.com/docs/using-fleet";
pub const WINDOWS_MDM_NOT_CONFIGURED_MESSAGE: &str =
    "Windows MDM isn't turned on. For more information about setting up MDM, please visit https://fleetdm.com/learn-more-about/windows-mdm";
pub const HOST_IDENTIFIER_NOT_FOUND: &str =
    "Host doesn't exist. Make sure you provide a valid hostname, UUID, or serial number. Learn more about host identifiers: https://fleetdm.com/learn-more-about/host-identifiers";
pub const ANDROID_MDM_NOT_CONFIGURED_MESSAGE: &str =
    "Android MDM isn't turned on.";
