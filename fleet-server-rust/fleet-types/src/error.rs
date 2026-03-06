//! Error types matching Go's `server/fleet/errors.go` and `server/platform/http/errors.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;

// ===========================================================================
// FleetError - Main error enum
// ===========================================================================

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

    #[error("authentication failed")]
    AuthFailed {
        internal: String,
    },

    #[error("authentication required")]
    AuthRequired {
        internal: String,
    },

    #[error("authorization header required")]
    AuthHeaderRequired {
        internal: String,
    },

    #[error("forbidden: {message}")]
    Forbidden { message: String },

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

    #[error("not configured")]
    NotConfigured,

    #[error("gateway error: {message}")]
    GatewayError { message: String, code: u16 },

    #[error("permission error: {message}")]
    PermissionError { message: String },

    #[error("OTA forbidden")]
    OTAForbidden {
        internal: Option<String>,
    },

    #[error("conflict: {message}")]
    Conflict { message: String },

    #[error("bad request: {message}")]
    BadRequest {
        message: String,
        internal: Option<String>,
    },

    #[error("{message}")]
    UserMessage {
        message: String,
        status_code: u16,
    },

    #[error("orbit error: {message}")]
    OrbitError { message: String, code: u16 },

    #[error("fleet error: code={code} {message}")]
    FleetCodedError { code: i32, message: String },

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
            FleetError::AuthFailed { .. } => 401,
            FleetError::AuthRequired { .. } => 401,
            FleetError::AuthHeaderRequired { .. } => 401,
            FleetError::Forbidden { .. } => 403,
            FleetError::PasswordResetRequired => 403,
            FleetError::MissingLicense => 402,
            FleetError::MDMNotConfigured => 400,
            FleetError::WindowsMDMNotConfigured => 400,
            FleetError::AndroidMDMNotConfigured => 400,
            FleetError::NotConfigured => 422,
            FleetError::GatewayError { code, .. } => *code,
            FleetError::PermissionError { .. } => 403,
            FleetError::OTAForbidden { .. } => 403,
            FleetError::Conflict { .. } => 409,
            FleetError::BadRequest { .. } => 400,
            FleetError::UserMessage { status_code, .. } => *status_code,
            FleetError::OrbitError { code, .. } => *code,
            FleetError::FleetCodedError { .. } => 500,
            FleetError::Internal(_) | FleetError::Database(_) => 500,
            FleetError::NoContext => 500,
        }
    }

    /// Returns true if this is a client error (4xx).
    pub fn is_client_error(&self) -> bool {
        let code = self.status_code();
        (400..500).contains(&code)
    }
}

// ===========================================================================
// InvalidArgument types
// ===========================================================================

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

    /// Returns the invalid arguments as a list of name/reason maps,
    /// matching Go's `Invalid()` method.
    pub fn invalid(&self) -> Vec<HashMap<String, String>> {
        self.errors
            .iter()
            .map(|i| {
                let mut m = HashMap::new();
                m.insert("name".to_string(), i.name.clone());
                m.insert("reason".to_string(), i.reason.clone());
                m
            })
            .collect()
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

// ===========================================================================
// FleetdError - errors reported by fleetd components
// ===========================================================================

/// FleetdError is an error reported by any of the fleetd components.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FleetdError {
    pub error_source: String,
    pub error_source_version: String,
    pub error_timestamp: DateTime<Utc>,
    pub error_message: String,
    #[serde(default)]
    pub error_additional_info: HashMap<String, serde_json::Value>,
    /// Vital errors are always reported to Fleet server.
    #[serde(default)]
    pub vital: bool,
}

impl fmt::Display for FleetdError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.error_message)
    }
}

impl std::error::Error for FleetdError {}

// ===========================================================================
// VPPIconAvailable
// ===========================================================================

/// VPPIconAvailable is an error-like type that indicates a VPP icon is available.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VPPIconAvailable {
    pub icon_url: String,
}

impl fmt::Display for VPPIconAvailable {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VPP icon available at: {}", self.icon_url)
    }
}

impl std::error::Error for VPPIconAvailable {}

// ===========================================================================
// Error number constants (from errors.go)
// ===========================================================================

/// Error number for valid role needed.
pub const ERR_NO_ROLE_NEEDED: i32 = 1;
/// Error number when all admins are about to be removed.
pub const ERR_NO_ONE_ADMIN_NEEDED: i32 = 2;
/// Returned when an item type in the translate payload is unknown.
pub const ERR_NO_UNKNOWN_TRANSLATE: i32 = 3;
/// Returned when a selected role for a user is for API only users.
pub const ERR_API_ONLY_ROLE: i32 = 4;

// ===========================================================================
// Well-known error messages (matching Go constants)
// ===========================================================================

pub const MDM_NOT_CONFIGURED_MESSAGE: &str =
    "MDM features aren't turned on in Fleet. For more information about setting up MDM, please visit https://fleetdm.com/docs/using-fleet";
pub const WINDOWS_MDM_NOT_CONFIGURED_MESSAGE: &str =
    "Windows MDM isn't turned on. For more information about setting up MDM, please visit https://fleetdm.com/learn-more-about/windows-mdm";
pub const ANDROID_MDM_NOT_CONFIGURED_MESSAGE: &str =
    "Android MDM isn't turned on. For more information about setting up MDM, please visit https://fleetdm.com/learn-more-about/how-to-connect-android-enterprise";
pub const APPLE_MDM_NOT_CONFIGURED_MESSAGE: &str =
    "macOS MDM isn't turned on. Visit https://fleetdm.com/docs/using-fleet to learn how to turn on MDM.";
pub const APPLE_ABM_DEFAULT_TEAM_DEPRECATED_MESSAGE: &str =
    "mdm.apple_bm_default_team has been deprecated. Please use the new mdm.apple_business_manager key documented here: https://fleetdm.com/learn-more-about/apple-business-manager-gitops";
pub const CANT_TURN_OFF_MDM_FOR_WINDOWS_HOSTS_MESSAGE: &str =
    "Can't turn off MDM for Windows hosts.";
pub const CANT_TURN_OFF_MDM_FOR_PERSONAL_HOSTS_MESSAGE: &str =
    "Couldn't turn off MDM. This command isn't available for personal hosts.";
pub const CANT_WIPE_PERSONAL_HOSTS_MESSAGE: &str =
    "Couldn't wipe. This command isn't available for personal hosts.";
pub const CANT_LOCK_PERSONAL_HOSTS_MESSAGE: &str =
    "Couldn't lock. This command isn't available for personal hosts.";
pub const CANT_LOCK_MANUAL_IOS_IPADOS_HOSTS_MESSAGE: &str =
    "Couldn't lock. This command isn't available for manually enrolled iOS/iPadOS hosts.";
pub const CANT_DISABLE_DISK_ENCRYPTION_IF_PIN_REQUIRED_ERR_MSG: &str =
    "Couldn't disable disk encryption, you need to disable the BitLocker PIN requirement first.";
pub const CANT_ENABLE_PIN_REQUIRED_IF_DISK_ENCRYPTION_ENABLED: &str =
    "Couldn't enable BitLocker PIN requirement, you must enable disk encryption first.";
pub const CANT_RESEND_APPLE_DECLARATION_PROFILES_MESSAGE: &str =
    "Can't resend declaration (DDM) profiles. Unlike configuration profiles (.mobileconfig), the host automatically checks in to get the latest DDM profiles.";

// Host/general error messages
pub const HOST_NOT_FOUND_ERR_MSG: &str =
    "Host doesn't exist. Make sure you provide a valid hostname, UUID, or serial number. Learn more about host identifiers: https://fleetdm.com/learn-more-about/host-identifiers";
pub const NO_HOSTS_TARGETED_ERR_MSG: &str =
    "No hosts targeted. Make sure you provide a valid hostname, UUID, or serial number. Learn more about host identifiers: https://fleetdm.com/learn-more-about/host-identifiers";
pub const TARGETED_HOSTS_DONT_EXIST_ERR_MSG: &str =
    "One or more targeted hosts don't exist. Make sure you provide a valid hostname, UUID, or serial number. Learn more about host identifiers: https://fleetdm.com/learn-more-about/host-identifiers";

// Script error messages
pub const RUN_SCRIPT_INVALID_TYPE_ERR_MSG: &str =
    "File type not supported. Only .sh (Bash) and .ps1 (PowerShell) file types are allowed.";
pub const RUN_SCRIPT_HOST_OFFLINE_ERR_MSG: &str = "Script can't run on offline host.";
pub const RUN_SCRIPT_FORBIDDEN_ERR_MSG: &str =
    "You don't have the right permissions in Fleet to run the script.";
pub const RUN_SCRIPT_ALREADY_RUNNING_ERR_MSG: &str =
    "A script is already running on this host. Please wait about 5 minutes to let it finish.";
pub const RUN_SCRIPT_HOST_TIMEOUT_ERR_MSG: &str =
    "Fleet didn't hear back from the host in under 5 minutes (timeout for live scripts). Fleet doesn't know if the script ran because it didn't receive the result. Go to Fleet and check Host details > Activities to see script results.";
pub const RUN_SCRIPT_SCRIPTS_DISABLED_GLOBALLY_ERR_MSG: &str =
    "Running scripts is disabled in organization settings.";
pub const RUN_SCRIPT_DISABLED_ERR_MSG: &str =
    "Scripts are disabled for this host. To run scripts, deploy the fleetd agent with scripts enabled.";
pub const RUN_SCRIPTS_ORBIT_DISABLED_ERR_MSG: &str =
    "Couldn't run script. To run a script, deploy the fleetd agent with --enable-scripts.";
pub const RUN_SCRIPT_ASYNC_SCRIPT_ENQUEUED_MSG: &str =
    "Script is running or will run when the host comes online.";
pub const RUN_SCRIPT_SAVED_MAX_LEN_ERR_MSG: &str =
    "Script is too large. It's limited to 500,000 characters (approximately 10,000 lines).";
pub const RUN_SCRIPT_UNSAVED_MAX_LEN_ERR_MSG: &str =
    "Script is too large. It's limited to 10,000 characters (approximately 125 lines).";
pub const RUN_SCRIPT_GATEWAY_TIMEOUT_ERR_MSG: &str =
    "Gateway timeout. Fleet didn't hear back from the host and doesn't know if the script ran. Please make sure your load balancer timeout isn't shorter than the Fleet server timeout.";

// Software error messages
pub const INSTALL_SOFTWARE_PERSONAL_APPLE_DEVICE_ERR_MSG: &str =
    "Couldn't install. Currently, software install isn't supported on personal (BYOD) iOS and iPadOS hosts.";

// End user authentication error messages
pub const END_USER_AUTH_DEP_WEB_URL_CONFIGURED_ERR_MSG: &str =
    "End user authentication can't be configured when the configured automatic enrollment (DEP) profile specifies a configuration_web_url.";

// Labels error messages
pub const INVALID_LABEL_SPECIFIED_ERR_MSG: &str = "Invalid label name(s):";

// Config error messages
pub const INVALID_SERVER_URL_MSG: &str = "Fleet server URL must use \"https\" or \"http\".";

// macOS setup experience error messages
pub const BOOTSTRAP_PKG_NOT_DISTRIBUTION_ERR_MSG: &str =
    "Couldn't add. Bootstrap package must be a distribution package. Learn more at: https://fleetdm.com/learn-more-about/macos-distribution-packages";

// NDES/SCEP validation error messages
pub const MULTIPLE_SCEP_PAYLOADS_ERR_MSG: &str = "Add only one SCEP payload.";
pub const SCEP_VARIABLES_NOT_IN_SCEP_PAYLOAD_ERR_MSG: &str =
    "Variables prefixed with \"$FLEET_VAR_SCEP_\", \"$FLEET_VAR_CUSTOM_SCEP_\", \"$FLEET_VAR_NDES_SCEP\" and \"$FLEET_VAR_SMALLSTEP_\" must only be in the SCEP payload.";

// List options error messages
pub const FILTER_TITLES_BY_PLATFORM_NEEDS_TEAM_ID_ERR_MSG: &str =
    "The 'platform' and 'team_id' parameters must be used together to filter the software available for install.";

// Windows MDM premium command error message
pub const WINDOWS_MDM_REQUIRES_PREMIUM_CMD_MESSAGE: &str =
    "Missing or invalid license. Wipe command is available in Fleet Premium only.";
