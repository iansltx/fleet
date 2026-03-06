//! Policy service operations.
//!
//! Implements policy CRUD for both global and team-scoped policies.
//! Corresponds to Go's `server/service/policies.go`.

use tracing::info;

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult, Viewer};

impl FleetService {
    /// Gets a policy by ID.
    ///
    /// Corresponds to Go's `(svc *Service) GetPolicy` / `GetPolicyByIDQueries`.
    pub async fn get_policy(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::Policy> {
        authz::authorize(viewer, Subject::Policy, Action::Read)?;
        self.ds.policy(id).await
    }

    /// Lists all global policies.
    ///
    /// Corresponds to Go's `(svc *Service) ListGlobalPolicies`.
    pub async fn list_global_policies(
        &self,
        viewer: &Viewer,
        opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<fleet_types::Policy>> {
        authz::authorize(viewer, Subject::Policy, Action::Read)?;
        self.ds.list_global_policies(opts).await
    }

    /// Creates a new global policy.
    ///
    /// Corresponds to Go's `(svc *Service) NewGlobalPolicy`.
    pub async fn new_global_policy(
        &self,
        viewer: &Viewer,
        payload: fleet_types::policy::PolicyPayload,
    ) -> ServiceResult<fleet_types::Policy> {
        authz::authorize(viewer, Subject::Policy, Action::Write)?;

        if payload.name.is_empty() {
            return Err(ServiceError::invalid_argument("name", "missing required argument"));
        }
        if payload.query.is_empty() {
            return Err(ServiceError::invalid_argument("query", "missing required argument"));
        }

        let policy = self
            .ds
            .new_global_policy(&payload.query, &payload.name, &payload.description)
            .await?;

        info!(policy_id = policy.policy_data.id, name = %policy.policy_data.name, "global policy created");
        Ok(policy)
    }

    /// Modifies an existing policy.
    ///
    /// Corresponds to Go's `(svc *Service) ModifyPolicy`.
    pub async fn modify_policy(
        &self,
        viewer: &Viewer,
        id: u32,
        payload: fleet_types::policy::ModifyPolicyPayload,
    ) -> ServiceResult<fleet_types::Policy> {
        authz::authorize(viewer, Subject::Policy, Action::Write)?;

        let mut policy = self.ds.policy(id).await?;

        if let Some(name) = payload.name {
            policy.policy_data.name = name;
        }
        if let Some(query) = payload.query {
            policy.policy_data.query = query;
        }
        if let Some(description) = payload.description {
            policy.policy_data.description = description;
        }
        if let Some(resolution) = payload.resolution {
            policy.policy_data.resolution = Some(resolution);
        }
        if let Some(platform) = payload.platform {
            policy.policy_data.platform = platform;
        }
        if let Some(critical) = payload.critical {
            policy.policy_data.critical = critical;
        }
        if let Some(calendar_events_enabled) = payload.calendar_events_enabled {
            policy.policy_data.calendar_events_enabled = calendar_events_enabled;
        }

        let saved = self.ds.save_policy(&policy).await?;
        info!(policy_id = saved.policy_data.id, name = %saved.policy_data.name, "policy modified");
        Ok(saved)
    }

    /// Deletes global policies by IDs.
    ///
    /// Corresponds to Go's `(svc *Service) DeleteGlobalPolicies`.
    pub async fn delete_global_policies(
        &self,
        viewer: &Viewer,
        ids: &[u32],
    ) -> ServiceResult<Vec<u32>> {
        authz::authorize(viewer, Subject::Policy, Action::Write)?;

        let deleted = self.ds.delete_global_policies(ids).await?;
        info!(count = deleted.len(), "global policies deleted");
        Ok(deleted)
    }

    /// Lists policies for a specific team.
    ///
    /// Corresponds to Go's `(svc *Service) ListTeamPolicies`.
    pub async fn list_team_policies(
        &self,
        viewer: &Viewer,
        team_id: u32,
        opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<fleet_types::Policy>> {
        authz::authorize(viewer, Subject::Policy, Action::Read)?;
        self.ds.list_team_policies(team_id, opts).await
    }

    /// Creates a new team-scoped policy.
    ///
    /// Corresponds to Go's `(svc *Service) NewTeamPolicy`.
    pub async fn new_team_policy(
        &self,
        viewer: &Viewer,
        team_id: u32,
        payload: fleet_types::policy::PolicyPayload,
    ) -> ServiceResult<fleet_types::Policy> {
        authz::authorize(viewer, Subject::Policy, Action::Write)?;

        if payload.name.is_empty() {
            return Err(ServiceError::invalid_argument("name", "missing required argument"));
        }
        if payload.query.is_empty() {
            return Err(ServiceError::invalid_argument("query", "missing required argument"));
        }

        let policy = self
            .ds
            .new_team_policy(team_id, &payload.query, &payload.name, &payload.description)
            .await?;

        info!(
            policy_id = policy.policy_data.id,
            team_id = team_id,
            name = %policy.policy_data.name,
            "team policy created"
        );
        Ok(policy)
    }

    /// Deletes team policies by IDs.
    ///
    /// Corresponds to Go's `(svc *Service) DeleteTeamPolicies`.
    pub async fn delete_team_policies(
        &self,
        viewer: &Viewer,
        team_id: u32,
        ids: &[u32],
    ) -> ServiceResult<Vec<u32>> {
        authz::authorize(viewer, Subject::Policy, Action::Write)?;

        let deleted = self.ds.delete_team_policies(team_id, ids).await?;
        info!(team_id = team_id, count = deleted.len(), "team policies deleted");
        Ok(deleted)
    }

    /// Applies policy specs (upsert global policies by name).
    ///
    /// Corresponds to Go's `(svc *Service) ApplyPolicySpecs`.
    pub async fn apply_policy_specs(
        &self,
        viewer: &Viewer,
        specs: Vec<PolicySpec>,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Policy, Action::Write)?;

        // Load all global policies for name lookup.
        let existing = self
            .ds
            .list_global_policies(fleet_types::ListOptions::default())
            .await?;

        for spec in &specs {
            if spec.name.is_empty() {
                return Err(ServiceError::invalid_argument("name", "missing required argument"));
            }
            if spec.query.is_empty() {
                return Err(ServiceError::invalid_argument("query", "missing required argument"));
            }

            if let Some(mut policy) = existing.iter().find(|p| p.policy_data.name == spec.name).cloned() {
                // Update existing policy.
                policy.policy_data.query = spec.query.clone();
                policy.policy_data.description = spec.description.clone().unwrap_or_default();
                if let Some(ref resolution) = spec.resolution {
                    policy.policy_data.resolution = Some(resolution.clone());
                }
                if let Some(ref platform) = spec.platform {
                    policy.policy_data.platform = platform.clone();
                }
                if let Some(critical) = spec.critical {
                    policy.policy_data.critical = critical;
                }
                self.ds.save_policy(&policy).await?;
                info!(name = %spec.name, "policy spec updated");
            } else {
                // Create new global policy.
                self.ds
                    .new_global_policy(
                        &spec.query,
                        &spec.name,
                        &spec.description.clone().unwrap_or_default(),
                    )
                    .await?;
                info!(name = %spec.name, "policy spec created");
            }
        }
        Ok(())
    }

    // ---- Premium-gated policy methods ----

    /// Autofills policy details using AI (premium-only).
    pub async fn autofill_policies(
        &self,
        viewer: &Viewer,
        query: Option<&str>,
        name: Option<&str>,
        description: Option<&str>,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Policy, Action::Write)?;
        self.require_premium()?;
        let _ = (query, name, description);
        // TODO: implement with AI service integration
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }
}

/// Spec representation of a policy for declarative management.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct PolicySpec {
    pub name: String,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub critical: Option<bool>,
}
