//! Team service operations.
//!
//! Implements team CRUD and related operations.
//! Corresponds to Go's `server/service/teams.go`.
//!
//! Note: In the Go implementation, the free (non-premium) service returns
//! `ErrMissingLicense` for most team operations. The enterprise (premium)
//! service overrides these methods. This Rust implementation mirrors
//! that structure.

use tracing::info;

use crate::authz::{self, Action, Subject};
use crate::fleet_service::{FleetService, TeamSummaryInfo};
use crate::{ServiceError, ServiceResult, Viewer};

impl FleetService {
    /// Lists all teams.
    ///
    /// Corresponds to Go's `(svc *Service) ListTeams`.
    /// Free tier returns MissingLicense; premium overrides.
    pub async fn list_teams(
        &self,
        viewer: &Viewer,
        opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<fleet_types::Team>> {
        authz::authorize(viewer, Subject::Team, Action::Read)?;
        self.ds.list_teams(opts).await
    }

    /// Gets a team by ID.
    ///
    /// Corresponds to Go's `(svc *Service) GetTeam`.
    pub async fn get_team(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::Team> {
        authz::authorize(viewer, Subject::Team, Action::Read)?;
        self.ds.team(id).await
    }

    /// Creates a new team.
    ///
    /// Corresponds to Go's `(svc *Service) NewTeam`.
    pub async fn new_team(
        &self,
        viewer: &Viewer,
        payload: fleet_types::team::TeamPayload,
    ) -> ServiceResult<fleet_types::Team> {
        authz::authorize(viewer, Subject::Team, Action::Write)?;

        let name = payload.name.as_deref().unwrap_or("").to_string();
        if name.is_empty() {
            return Err(ServiceError::invalid_argument("name", "missing required argument"));
        }

        if fleet_types::team::is_reserved_team_name(&name) {
            return Err(ServiceError::invalid_argument(
                "name",
                format!("'{}' is a reserved team name", name),
            ));
        }

        let team = fleet_types::Team {
            id: 0,
            gitops_filename: None,
            name: name.clone(),
            description: payload.description.unwrap_or_default(),
            config: Default::default(),
            user_count: 0,
            users: Vec::new(),
            host_count: 0,
            hosts: Vec::new(),
            secrets: payload.secrets,
            created_at: chrono::Utc::now(),
        };

        let created = self.ds.new_team(&team).await?;

        info!(team_id = created.id, name = %created.name, "team created");
        Ok(created)
    }

    /// Modifies an existing team.
    ///
    /// Corresponds to Go's `(svc *Service) ModifyTeam`.
    pub async fn modify_team(
        &self,
        viewer: &Viewer,
        id: u32,
        payload: fleet_types::team::TeamPayload,
    ) -> ServiceResult<fleet_types::Team> {
        authz::authorize(viewer, Subject::Team, Action::Write)?;

        let mut team = self.ds.team(id).await?;

        if let Some(name) = payload.name {
            if fleet_types::team::is_reserved_team_name(&name) {
                return Err(ServiceError::invalid_argument(
                    "name",
                    format!("'{}' is a reserved team name", name),
                ));
            }
            team.name = name;
        }
        if let Some(description) = payload.description {
            team.description = description;
        }
        if let Some(secrets) = payload.secrets {
            team.secrets = Some(secrets);
        }
        if let Some(webhook_settings) = payload.webhook_settings {
            team.config.webhook_settings = webhook_settings;
        }
        if let Some(integrations) = payload.integrations {
            team.config.integrations = integrations;
        }
        if let Some(host_expiry_settings) = payload.host_expiry_settings {
            team.config.host_expiry_settings = host_expiry_settings;
        }

        let saved = self.ds.save_team(&team).await?;
        info!(team_id = saved.id, name = %saved.name, "team modified");
        Ok(saved)
    }

    /// Deletes a team.
    ///
    /// Corresponds to Go's `(svc *Service) DeleteTeam`.
    pub async fn delete_team(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Team, Action::Write)?;

        self.ds.delete_team(id).await?;

        info!(team_id = id, "team deleted");
        Ok(())
    }

    /// Returns summary info for all teams. Used internally for validation
    /// (e.g., verifying team IDs when creating users).
    pub async fn teams_summary(&self) -> ServiceResult<Vec<TeamSummaryInfo>> {
        self.ds.teams_summary().await
    }

    /// Lists users belonging to a team.
    ///
    /// Corresponds to Go's `(svc *Service) ListTeamUsers`.
    pub async fn list_team_users(
        &self,
        viewer: &Viewer,
        team_id: u32,
    ) -> ServiceResult<Vec<fleet_types::team::TeamUser>> {
        authz::authorize(viewer, Subject::Team, Action::Read)?;
        // Verify team exists
        self.ds.team(team_id).await?;
        self.ds.list_team_users(team_id).await
    }

    /// Gets enroll secrets for a team.
    ///
    /// Corresponds to Go's `(svc *Service) TeamEnrollSecrets`.
    pub async fn team_enroll_secrets(
        &self,
        viewer: &Viewer,
        team_id: u32,
    ) -> ServiceResult<Vec<fleet_types::enroll::EnrollSecret>> {
        authz::authorize(viewer, Subject::EnrollSecret, Action::Read)?;
        self.ds.team_enroll_secrets(team_id).await
    }

    /// Modifies enroll secrets for a team.
    ///
    /// Corresponds to Go's `(svc *Service) ModifyTeamEnrollSecrets`.
    pub async fn modify_team_enroll_secrets(
        &self,
        viewer: &Viewer,
        team_id: u32,
        secrets: Vec<String>,
    ) -> ServiceResult<Vec<fleet_types::enroll::EnrollSecret>> {
        authz::authorize(viewer, Subject::EnrollSecret, Action::Write)?;
        // Verify team exists
        self.ds.team(team_id).await?;
        self.ds.apply_team_enroll_secrets(team_id, &secrets).await?;
        self.ds.team_enroll_secrets(team_id).await
    }

    /// Modifies agent options for a team.
    ///
    /// Corresponds to Go's `(svc *Service) ModifyTeamAgentOptions`.
    pub async fn modify_team_agent_options(
        &self,
        viewer: &Viewer,
        team_id: u32,
        agent_options: serde_json::Value,
    ) -> ServiceResult<fleet_types::Team> {
        authz::authorize(viewer, Subject::Team, Action::Write)?;

        let mut team = self.ds.team(team_id).await?;
        team.config.agent_options = Some(agent_options);
        let saved = self.ds.save_team(&team).await?;

        info!(team_id = saved.id, name = %saved.name, "team agent options modified");
        Ok(saved)
    }
}
