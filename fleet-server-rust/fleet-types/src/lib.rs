//! Fleet Types - Core data types for the Fleet server.
//!
//! This crate contains all the core data models used throughout the Fleet server,
//! ported from the Go implementation. All types are serialization-compatible with
//! the Go types for JSON API responses.

pub mod activity;
pub mod blobstore;
pub mod campaign;
pub mod carve;
pub mod certificate;
pub mod config;
pub mod enroll;
pub mod error;
pub mod host;
pub mod invite;
pub mod label;
pub mod mdm;
pub mod optjson;
pub use optjson::{OptBool, OptInt, OptJson, OptSlice, OptString};
pub mod osquery;
pub mod pack;
pub mod policy;
pub mod query;
pub mod script;
pub mod service;
pub mod session;
pub mod software;
pub mod target;
pub mod team;
pub mod user;
pub mod vulnerability;

// Re-export commonly used types at the crate root.
pub use activity::{Activity, UpcomingActivity};
pub use campaign::*;
pub use carve::*;

// Config types (selectively re-exported to avoid conflicts).
pub use config::{
    ActivitiesWebhookSettings, ActivityExpirySettings, AndroidSettings, AppConfig,
    AppleOSUpdateSettings, ApplyClientSpecOptions, ApplySpecOptions, ApplyTeamSpecOptions,
    CertificateTemplateSpec, ConditionalAccessSettings, DeviceFeatures, DeviceGlobalConfig,
    DeviceGlobalMDMConfig, DiskEncryptionConfig, EmailConfig, EnrichedAppConfig,
    FailingPoliciesWebhookSettings, Features, FirehoseConfig, FleetDesktopSettings,
    GoogleCalendarApiKey, GoogleCalendarIntegration, HostExpirySettings,
    HostStatusWebhookSettings, Integrations, JiraIntegration, KafkaRESTConfig, KinesisConfig,
    LambdaConfig, LicenseInfo, ListOptions, ListQueryOptions, Logging, LoggingPlugin,
    MDMAppleABMAssignmentInfo, MDMAppleVolumePurchasingProgramInfo, MDMConfig,
    MDMEndUserAuthentication, MacOSMigration, MacOSMigrationMode, MacOSSetup, MacOSSetupSoftware,
    MacOSSettings, NatsConfig, OrderDirection, OrgInfo, Partnerships, SESConfig, SMTPSettings,
    SSOProviderSettings, SSOSettings, SecretVariable, ServerSettings, TeamSpecsDryRunAssumptions,
    UIGitOpsModeConfig, UpdateIntervalConfig, VulnerabilitiesConfig,
    VulnerabilitiesWebhookSettings, VulnerabilitySettings, WebhookSettings, WindowsSettings,
    WindowsUpdates, YaraRule, YaraRuleSpec, ZendeskIntegration,
};

pub use enroll::{EnrollSecret, EnrollSecretSpec};
pub use error::*;
pub use host::{
    AggregatedMDMData, AggregatedMDMSolutions, AggregatedMDMStatus, AggregatedMacadminsData,
    AggregatedMunkiIssue, AggregatedMunkiVersion, Host, HostBattery, HostDetail,
    HostDetailOptions, HostDeviceMapping, HostDiskEncryptionKey, HostEndUser, HostListOptions,
    HostLite, HostMDM, HostMDMCheckinInfo, HostMaintenanceWindow, HostMunkiInfo, HostMunkiIssue,
    HostStatus, HostSummary, HostUser, HostVulnerabilitySummary, MDMSolution, MacadminsData,
    MunkiIssue, NetworkInterface, OSVersion, OSVersionStats, OSVersions,
};
pub use invite::{Invite, InvitePayload};
pub use label::{Label, LabelMembershipType, LabelSpec, LabelSummary, LabelType};
pub use mdm::{
    ABMToken, ABMTokenTeam, AppleBM, AppleCSR, AppleDevice, AppleDevicesToRefetch, AppleMDM,
    BatchModifyMDMConfigProfilePayload, CommandEnqueueResult, ConfigurationProfileLabel,
    DEPAssignProfileResponseStatus, EnrolledAPIResult, EnrolledAPIResults, ExpectedMDMProfile,
    HostDEPAssignment, HostLocationData, HostMDMAppleProfile, HostMDMCertificateProfile,
    HostMDMCommand, HostMDMIdentifiers, HostMDMProfile, HostMDMProfileRetryCount,
    HostMDMWindowsProfile, InstallableDevicePlatform, MDMAppleBootstrapPackage,
    MDMAppleBootstrapPackageSummary, MDMAppleBulkUpsertHostProfilePayload, MDMAppleCommand,
    MDMAppleConfigProfile, MDMCustomEnrollmentProfileItem, MDMAppleDDMActivation,
    MDMAppleDDMActivationPayload, MDMAppleDDMDeclarationItem, MDMAppleDDMDeclarationItemsResponse,
    MDMAppleDDMDeclarationResponse, MDMAppleDDMDeclarationsToken, MDMAppleDDMErrors,
    MDMAppleDDMManifest, MDMAppleDDMManifestItems, MDMAppleDDMStatusDeclaration,
    MDMAppleDDMStatusDeclarations, MDMAppleDDMStatusErrorReason, MDMAppleDDMStatusItems,
    MDMAppleDDMStatusManagement, MDMAppleDDMStatusReport, MDMAppleDDMTokensResponse,
    MDMAppleDEPDevice, MDMAppleDEPKeyPair, MDMAppleDeclaration, MDMAppleDeclarationValidity,
    MDMAppleDevice, MDMAppleEnrolledDeviceInfo, MDMAppleEnrollmentProfile,
    MDMAppleEnrollmentProfilePayload, MDMAppleEnrollmentType, MDMAppleFileVaultSummary,
    MDMAppleFleetdConfig, MDMAppleHostDeclaration, MDMAppleInstaller, MDMAppleMachineInfo,
    MDMApplePreassignHostProfiles, MDMApplePreassignProfile, MDMApplePreassignProfilePayload,
    MDMAppleProfilePayload, MDMAppleRawDeclaration, MDMAppleSettingsPayload,
    MDMAppleSetupAssistant, MDMAppleSetupPayload, MDMAppleSoftwareUpdateAsset,
    MDMAppleSoftwareUpdateRequired, MDMAppleSoftwareUpdateRequiredDetails, MDMAssetName,
    MDMCommand, MDMCommandAuthz, MDMCommandFilters, MDMCommandListOptions, MDMCommandResult,
    MDMCommandStatusFilter, MDMCommandType, MDMConfigAsset, MDMConfigProfileAuthz,
    MDMConfigProfilePayload, MDMConfigProfileStatus, MDMDeliveryStatus, MDMDiskEncryptionSummary,
    MDMEULAPayload, MDMIdPAccount, MDMLabelsMode, MDMLinuxDiskEncryptionSummary,
    MDMManagedCertificate, MDMOperationType, MDMPlatform, MDMPlatformsCounts,
    MDMProfileBatchPayload, MDMProfileSpec, MDMProfileUUIDFleetVariables, MDMProfilesUpdates,
    MDMProfilesSummary, MDMWindowsBitLockerSummary, MDMWindowsBulkUpsertHostProfilePayload,
    MDMWindowsCommand, MDMWindowsConfigProfile, MDMWindowsEnrolledDevice,
    MDMWindowsProfileContents, MDMWindowsProfilePayload, MDMWindowsWipeMetadata,
    MDMWindowsWipeType, MDMWipeMetadata, MDMEULA, NanoEnrollment, NanoUser, NullTeamType,
    PayloadScope, ProtoCmdState, SCEPIdentityAssociation, SCEPIdentityCertificate, SyncMLDataType,
    TeamTuple, VPPTokenDB, VPPTokenData, VPPTokenInfo, VPPTokenRaw, WindowsMDMAccessTokenPayload,
    WindowsMDMEnrollmentType,
};
pub use osquery::*;
pub use pack::{Pack, PackStats, ScheduledQuery};
pub use policy::{HostPolicy, Policy, PolicyData, PolicySpec};
pub use query::{Query, QueryResultRow, QuerySpec, QueryStats};
pub use script::{BatchScriptHost, HostLockWipeStatus, Script, ScriptPayload};
pub use session::Session;
pub use software::{
    Software, SoftwareCategory, SoftwareInstallDetails, SoftwareInstallResult,
    SoftwareInstallerStatus, SoftwareListOptions, SoftwareSpec, SoftwareTitle,
    SoftwareTitleListOptions, VPPApp, VPPToken,
};
pub use target::*;
pub use team::{
    DefaultTeam, DefaultTeamConfig, DefaultTeamIntegrations, DefaultTeamWebhookSettings, Team,
    TeamConfig, TeamConfigLite, TeamFilter, TeamGoogleCalendarIntegration, TeamIntegrations,
    TeamJiraIntegration, TeamLite, TeamMDM, TeamPayload, TeamPayloadMDM, TeamRole, TeamSpec,
    TeamSpecAppStoreApp, TeamSpecIntegrations, TeamSpecMDM, TeamSpecSoftwareAsset,
    TeamSpecWebhookSettings, TeamSummary, TeamUser, TeamWebhookSettings,
    TeamZendeskIntegration, UserTeam,
};
pub use user::{User, UserPayload};
pub use vulnerability::{CVE, CVEMeta, SoftwareVulnerability};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

/// API version constant for spec objects.
pub const API_VERSION: &str = "v1";

/// ObjectMetadata holds common metadata for spec objects (used in YAML import/export).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ObjectMetadata {
    #[serde(default)]
    pub api_version: String,
    #[serde(default)]
    pub kind: String,
}

/// Timestamps for entity creation.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct CreateTimestamp {
    pub created_at: DateTime<Utc>,
}

/// Timestamps for entity updates.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateTimestamp {
    pub updated_at: DateTime<Utc>,
}

/// Combined create and update timestamps, matching Go's UpdateCreateTimestamps.
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct UpdateCreateTimestamps {
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}
