//! Invite service operations.
//!
//! Implements invite CRUD: creating, listing, updating, deleting, and verifying invitations.
//! Corresponds to Go's `server/service/invites.go`.

use tracing::info;

use crate::auth;
use crate::authz::{self, Action, Subject};
use crate::fleet_service::{FleetService, InviteData};
use crate::{ServiceError, ServiceResult, Viewer};

impl FleetService {
    /// Creates a new user invitation.
    ///
    /// Corresponds to Go's `(svc *Service) InviteNewUser`.
    pub async fn invite_new_user(
        &self,
        viewer: &Viewer,
        payload: InvitePayload,
    ) -> ServiceResult<InviteData> {
        authz::authorize(viewer, Subject::Invite, Action::Write)?;

        let email = payload
            .email
            .as_deref()
            .ok_or_else(|| ServiceError::invalid_argument("email", "missing required argument"))?
            .to_lowercase();

        // Verify that a user with this email doesn't already exist.
        match self.ds.user_by_email(&email).await {
            Ok(_) => {
                return Err(ServiceError::invalid_argument(
                    "email",
                    "a user with this account already exists",
                ));
            }
            Err(ServiceError::NotFound(_)) => { /* Expected -- no existing user. */ }
            Err(e) => return Err(e),
        }

        // Generate invite token.
        let token = auth::generate_random_text(self.config.app.token_key_size)?;

        let invite = InviteData {
            id: 0,
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
            invited_by: viewer.user_id(),
            email: email.clone(),
            name: payload.name.unwrap_or_default(),
            position: payload.position.unwrap_or_default(),
            token,
            sso_enabled: payload.sso_enabled.unwrap_or(false),
            mfa_enabled: payload.mfa_enabled.unwrap_or(false),
            global_role: payload.global_role,
            teams: payload.teams.unwrap_or_default(),
        };

        // Validate MFA/SSO constraints.
        if invite.mfa_enabled && invite.sso_enabled {
            return Err(ServiceError::Conflict(
                "Fleet MFA is not applicable to SSO users".to_string(),
            ));
        }

        let created = self.ds.new_invite(&invite).await?;

        info!(invite_id = created.id, email = %created.email, "invite created");
        Ok(created)
    }

    /// Lists all invites.
    ///
    /// Corresponds to Go's `(svc *Service) ListInvites`.
    pub async fn list_invites(
        &self,
        viewer: &Viewer,
        opts: fleet_types::ListOptions,
    ) -> ServiceResult<Vec<InviteData>> {
        authz::authorize(viewer, Subject::Invite, Action::Read)?;
        self.ds.list_invites(opts).await
    }

    /// Updates an existing invite.
    ///
    /// Corresponds to Go's `(svc *Service) UpdateInvite`.
    pub async fn update_invite(
        &self,
        viewer: &Viewer,
        id: u32,
        payload: InvitePayload,
    ) -> ServiceResult<InviteData> {
        authz::authorize(viewer, Subject::Invite, Action::Write)?;

        // Fetch invite by listing and finding the one with the matching ID.
        // In practice, the datastore would have a get_invite_by_id method.
        let current = self
            .ds
            .list_invites(fleet_types::ListOptions::default())
            .await?
            .into_iter()
            .find(|inv| inv.id == id)
            .ok_or_else(|| ServiceError::not_found(format!("invite {}", id)))?;

        let mut updated = current;

        if let Some(email) = payload.email {
            updated.email = email.to_lowercase();
        }
        if let Some(name) = payload.name {
            updated.name = name;
        }
        if let Some(position) = payload.position {
            updated.position = position;
        }
        if let Some(sso_enabled) = payload.sso_enabled {
            updated.sso_enabled = sso_enabled;
        }
        if let Some(mfa_enabled) = payload.mfa_enabled {
            updated.mfa_enabled = mfa_enabled;
        }
        if let Some(global_role) = payload.global_role {
            updated.global_role = Some(global_role);
        }
        if let Some(teams) = payload.teams {
            updated.teams = teams;
        }

        let result = self.ds.update_invite(id.into(), &updated).await?;
        info!(invite_id = id, "invite updated");
        Ok(result)
    }

    /// Deletes an invite.
    ///
    /// Corresponds to Go's `(svc *Service) DeleteInvite`.
    pub async fn delete_invite(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Invite, Action::Write)?;
        self.ds.delete_invite(id.into()).await?;

        info!(invite_id = id, "invite deleted");
        Ok(())
    }

    /// Verifies an invite token and returns the invite.
    ///
    /// Corresponds to Go's `(svc *Service) VerifyInvite`.
    pub async fn verify_invite(&self, token: &str) -> ServiceResult<InviteData> {
        let invite = self.ds.invite_by_token(token).await.map_err(|_| {
            ServiceError::auth_failed("invalid invite token")
        })?;
        Ok(invite)
    }
}

/// Payload for creating or updating an invite.
#[derive(Debug, Clone, Default)]
pub struct InvitePayload {
    pub email: Option<String>,
    pub name: Option<String>,
    pub position: Option<String>,
    pub sso_enabled: Option<bool>,
    pub mfa_enabled: Option<bool>,
    pub global_role: Option<String>,
    pub teams: Option<Vec<fleet_types::team::UserTeam>>,
}
