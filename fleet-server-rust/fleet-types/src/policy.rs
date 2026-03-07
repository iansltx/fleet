//! Policy types matching Go's `server/fleet/policies.go`.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::label::LabelIdent;

/// PolicyPayload holds data for policy creation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyPayload {
    pub query_id: Option<u32>,
    pub name: String,
    pub query: String,
    pub critical: bool,
    pub description: String,
    pub resolution: String,
    pub platform: String,
    pub calendar_events_enabled: bool,
    pub software_installer_id: Option<u32>,
    pub vpp_apps_teams_id: Option<u32>,
    pub script_id: Option<u32>,
    #[serde(default)]
    pub labels_include_any: Vec<String>,
    #[serde(default)]
    pub labels_exclude_any: Vec<String>,
    pub conditional_access_enabled: bool,
    pub conditional_access_bypass_enabled: Option<bool>,
}

/// ModifyPolicyPayload holds data for policy modification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ModifyPolicyPayload {
    pub name: Option<String>,
    pub query: Option<String>,
    pub description: Option<String>,
    pub resolution: Option<String>,
    pub platform: Option<String>,
    pub critical: Option<bool>,
    pub calendar_events_enabled: Option<bool>,
    pub software_title_id: Option<u32>,
    pub script_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_exclude_any: Vec<String>,
    pub conditional_access_enabled: Option<bool>,
    pub conditional_access_bypass_enabled: Option<bool>,
}

/// PolicySoftwareTitle holds information about software to install when a policy fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySoftwareTitle {
    pub software_title_id: u32,
    pub name: String,
    pub display_name: String,
}

/// PolicyScript holds information about a script to run when a policy fails.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyScript {
    pub id: u32,
    pub name: String,
}

/// PolicyData holds data of a fleet policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyData {
    pub id: u32,
    pub name: String,
    pub query: String,
    pub critical: bool,
    pub description: String,
    pub author_id: Option<u32>,
    pub author_name: String,
    pub author_email: String,
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    pub platform: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<LabelIdent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_exclude_any: Vec<LabelIdent>,
    pub calendar_events_enabled: bool,
    #[serde(skip)]
    pub software_installer_id: Option<u32>,
    #[serde(skip)]
    pub vpp_apps_teams_id: Option<u32>,
    #[serde(skip)]
    pub script_id: Option<u32>,
    pub conditional_access_enabled: bool,
    pub conditional_access_bypass_enabled: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Policy is a fleet's policy query.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Policy {
    #[serde(flatten)]
    pub policy_data: PolicyData,

    pub passing_host_count: u32,
    pub failing_host_count: u32,
    pub host_count_updated_at: Option<DateTime<Utc>>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub install_software: Option<PolicySoftwareTitle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_script: Option<PolicyScript>,
}

/// HostPolicy represents a policy result for a specific host.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostPolicy {
    #[serde(flatten)]
    pub policy_data: PolicyData,

    /// Response can be "pass", "fail", or "" (not yet run).
    pub response: String,
}

/// PolicySpec is used to hold policy data to apply policy specs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySpec {
    pub name: String,
    pub query: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub critical: bool,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub resolution: String,
    #[serde(default, skip_serializing_if = "String::is_empty", rename = "team")]
    pub team: String,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub platform: String,
    #[serde(default)]
    pub calendar_events_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub software_title_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script_id: Option<u32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_include_any: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub labels_exclude_any: Vec<String>,
    #[serde(default)]
    pub conditional_access_enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conditional_access_bypass_enabled: Option<bool>,
}

/// NewTeamPolicyPayload holds data for team policy creation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NewTeamPolicyPayload {
    pub query_id: Option<u32>,
    pub name: String,
    pub query: String,
    pub critical: bool,
    pub description: String,
    pub resolution: String,
    pub platform: String,
    pub calendar_events_enabled: bool,
    pub software_title_id: Option<u32>,
    pub script_id: Option<u32>,
    #[serde(default)]
    pub labels_include_any: Vec<String>,
    #[serde(default)]
    pub labels_exclude_any: Vec<String>,
    pub conditional_access_enabled: bool,
    pub conditional_access_bypass_enabled: Option<bool>,
}

/// PolicyCalendarData contains calendar-related policy data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyCalendarData {
    pub id: u32,
    pub name: String,
}

/// PolicySoftwareInstallerData holds policy and software installer ID pairing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicySoftwareInstallerData {
    pub id: u32,
    pub software_installer_id: u32,
}

/// PolicyVPPData holds policy and VPP app data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyVPPData {
    pub id: u32,
    pub adam_id: String,
    pub platform: String,
}

/// PolicyScriptData holds policy and script ID pairing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyScriptData {
    pub id: u32,
    pub script_id: u32,
}

/// PolicyLite is a stripped down version of the policy.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PolicyLite {
    pub id: u32,
    pub name: String,
    pub description: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
}

/// PolicySetHost is a host entry for a policy set.
#[derive(Debug, Clone)]
pub struct PolicySetHost {
    pub id: u32,
    pub hostname: String,
    pub display_name: String,
}

/// PolicyMembershipResult holds the result of a policy membership check.
#[derive(Debug, Clone)]
pub struct PolicyMembershipResult {
    pub host_id: u32,
    pub policy_id: u32,
    pub passes: Option<bool>,
}

/// PolicyNoTeamID is the team ID for "No team" policies.
pub const POLICY_NO_TEAM_ID: u32 = 0;
/// Max retries for policy automation.
pub const MAX_POLICY_AUTOMATION_RETRIES: u32 = 3;
/// Policy kind constant.
pub const POLICY_KIND: &str = "policy";

// ───────────────────────────────────────────────────────────────────────────
// Validation helpers
// ───────────────────────────────────────────────────────────────────────────

/// Errors that can occur when verifying policy-related payloads.
#[derive(Debug, Clone, thiserror::Error)]
pub enum PolicyValidationError {
    #[error("policy name cannot be empty")]
    EmptyName,
    #[error("policy query cannot be empty")]
    EmptyQuery,
    #[error("both fields \"queryID\" and \"query\" cannot be set")]
    IdAndQuerySet,
    #[error("invalid policy platform")]
    InvalidPlatform,
    #[error("policy cannot include both labels_include_any and labels_exclude_any")]
    ConflictingLabels,
}

fn is_empty_string(s: &str) -> bool {
    s.trim().is_empty()
}

/// Verify that a policy name is non-empty.
pub fn verify_policy_name(name: &str) -> Result<(), PolicyValidationError> {
    if is_empty_string(name) {
        return Err(PolicyValidationError::EmptyName);
    }
    Ok(())
}

/// Verify that a policy query is non-empty.
pub fn verify_policy_query(query: &str) -> Result<(), PolicyValidationError> {
    if is_empty_string(query) {
        return Err(PolicyValidationError::EmptyQuery);
    }
    Ok(())
}

/// Verify that a comma-separated platform string contains only valid platforms.
pub fn verify_policy_platforms(platforms: &str) -> Result<(), PolicyValidationError> {
    if platforms.is_empty() {
        return Ok(());
    }
    for s in platforms.split(',') {
        match s.trim() {
            "windows" | "linux" | "darwin" | "chrome" => {}
            _ => return Err(PolicyValidationError::InvalidPlatform),
        }
    }
    Ok(())
}

impl PolicyPayload {
    /// Verify verifies the policy payload is valid.
    pub fn verify(&self) -> Result<(), PolicyValidationError> {
        if self.query_id.is_some() {
            if !self.query.is_empty() {
                return Err(PolicyValidationError::IdAndQuerySet);
            }
        } else {
            verify_policy_name(&self.name)?;
            verify_policy_query(&self.query)?;
        }
        verify_policy_platforms(&self.platform)?;
        if !self.labels_include_any.is_empty() && !self.labels_exclude_any.is_empty() {
            return Err(PolicyValidationError::ConflictingLabels);
        }
        Ok(())
    }
}

impl ModifyPolicyPayload {
    /// Verify verifies the modify-policy payload is valid.
    pub fn verify(&self) -> Result<(), PolicyValidationError> {
        if let Some(ref name) = self.name {
            verify_policy_name(name)?;
        }
        if let Some(ref query) = self.query {
            verify_policy_query(query)?;
        }
        if let Some(ref platform) = self.platform {
            verify_policy_platforms(platform)?;
        }
        Ok(())
    }
}

impl PolicySpec {
    /// Verify verifies the policy spec is valid.
    pub fn verify(&self) -> Result<(), PolicyValidationError> {
        verify_policy_name(&self.name)?;
        verify_policy_query(&self.query)?;
        verify_policy_platforms(&self.platform)?;
        Ok(())
    }
}

/// Returns the first duplicate policy spec name (within the same team), or `None`
/// if there are no duplicates.
pub fn first_duplicate_policy_spec_name(specs: &[PolicySpec]) -> Option<&str> {
    let mut teams: std::collections::HashMap<&str, std::collections::HashSet<&str>> =
        std::collections::HashMap::new();
    for spec in specs {
        let team_set = teams.entry(&spec.team).or_default();
        if !team_set.insert(&spec.name) {
            return Some(&spec.name);
        }
    }
    None
}

/// FailingPolicySet holds sets of hosts that failed policy executions.
pub trait FailingPolicySet: Send + Sync {
    /// Lists all the policy sets.
    fn list_sets(&self) -> Result<Vec<u32>, Box<dyn std::error::Error>>;
    /// Adds the given host to the policy set.
    fn add_host(
        &self,
        policy_id: u32,
        host: PolicySetHost,
    ) -> Result<(), Box<dyn std::error::Error>>;
    /// Returns the list of hosts present in the policy set.
    fn list_hosts(
        &self,
        policy_id: u32,
    ) -> Result<Vec<PolicySetHost>, Box<dyn std::error::Error>>;
    /// Removes the hosts from the policy set.
    fn remove_hosts(
        &self,
        policy_id: u32,
        hosts: &[PolicySetHost],
    ) -> Result<(), Box<dyn std::error::Error>>;
    /// Removes a policy set.
    fn remove_set(&self, policy_id: u32) -> Result<(), Box<dyn std::error::Error>>;
}
