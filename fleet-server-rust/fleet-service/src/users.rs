//! User service operations.
//!
//! Implements user CRUD, password management, and related business logic.
//! Corresponds to Go's `server/service/users.go`.

use tracing::{info, warn};

use crate::auth;
use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult, Viewer};

impl FleetService {
    /// Creates a new user (admin-initiated).
    ///
    /// Corresponds to Go's `(svc *Service) CreateUser`.
    pub async fn create_user(
        &self,
        viewer: &Viewer,
        payload: CreateUserPayload,
    ) -> ServiceResult<(fleet_types::User, Option<String>)> {
        authz::authorize(viewer, Subject::User, Action::Write)?;

        if payload.email.is_empty() {
            return Err(ServiceError::invalid_argument("email", "missing required argument"));
        }
        if payload.name.is_empty() {
            return Err(ServiceError::invalid_argument("name", "missing required argument"));
        }

        // Validate teams exist if provided.
        if let Some(ref team_list) = payload.teams {
            let available = self.ds.teams_summary().await?;
            let valid_ids: std::collections::HashSet<u32> =
                available.iter().map(|t| t.id).collect();
            for ut in team_list {
                if !valid_ids.contains(&(ut.team.id )) {
                    return Err(ServiceError::invalid_argument(
                        "teams.id",
                        format!("team with id {} does not exist", ut.team.id),
                    ));
                }
            }
        }

        // Check if invite already exists for email.
        if let Ok(Some(_)) = self.ds.invite_by_email(&payload.email).await {
            return Err(ServiceError::Conflict(format!(
                "{} already invited",
                payload.email
            )));
        }

        // Hash the password.
        let password_hash = if let Some(ref pwd) = payload.password {
            auth::hash_password(pwd, self.config.auth.bcrypt_cost)?
        } else if !payload.sso_enabled {
            return Err(ServiceError::invalid_argument(
                "password",
                "password required for non-SSO users",
            ));
        } else {
            Vec::new()
        };

        let now = chrono::Utc::now();
        let user = fleet_types::User {
            id: 0,
            created_at: now,
            updated_at: now,
            name: payload.name.clone(),
            email: payload.email.clone(),
            password: password_hash,
            salt: String::new(),
            admin_forced_password_reset: payload.admin_forced_password_reset.unwrap_or(true),
            gravatar_url: String::new(),
            sso_enabled: payload.sso_enabled,
            mfa_enabled: payload.mfa_enabled.unwrap_or(false),
            api_only: payload.api_only,
            global_role: payload.global_role.clone(),
            position: payload.position.clone().unwrap_or_default(),
            teams: payload.teams.clone().unwrap_or_default(),
            settings: None,
            deleted: false,
            invite_id: None,
        };

        let created_user = self.ds.new_user(&user).await?;

        // For API-only non-SSO users, create a session and return the key.
        let session_key = if created_user.api_only && !created_user.sso_enabled {
            if let Some(ref pwd) = payload.password {
                match self.login_user(&created_user.email, pwd).await {
                    Ok((_, session)) => Some(session.key),
                    Err(e) => {
                        warn!("Failed to create session for API-only user: {}", e);
                        None
                    }
                }
            } else {
                None
            }
        } else {
            None
        };

        info!(user_id = created_user.id, email = %created_user.email, "user created");
        Ok((created_user, session_key))
    }

    /// Creates a user from an invitation.
    ///
    /// Corresponds to Go's `(svc *Service) CreateUserFromInvite`.
    pub async fn create_user_from_invite(
        &self,
        payload: CreateUserFromInvitePayload,
    ) -> ServiceResult<fleet_types::User> {
        let invite = self.verify_invite(&payload.invite_token).await?;

        if invite.email != payload.email {
            return Err(ServiceError::invalid_argument(
                "invite_token",
                "Invite Token does not match Email Address.",
            ));
        }

        let password_hash = auth::hash_password(&payload.password, self.config.auth.bcrypt_cost)?;

        let now = chrono::Utc::now();
        let user = fleet_types::User {
            id: 0,
            created_at: now,
            updated_at: now,
            name: payload.name.clone(),
            email: payload.email.clone(),
            password: password_hash,
            salt: String::new(),
            admin_forced_password_reset: false,
            gravatar_url: String::new(),
            global_role: invite.global_role.clone(),
            teams: invite.teams.clone(),
            mfa_enabled: invite.mfa_enabled,
            sso_enabled: false,
            api_only: false,
            position: String::new(),
            settings: None,
            deleted: false,
            invite_id: None,
        };

        let created_user = self.ds.new_user(&user).await?;
        self.ds.delete_invite(invite.id).await?;

        info!(user_id = created_user.id, email = %created_user.email, "user created from invite");
        Ok(created_user)
    }

    /// Gets a user by ID.
    pub async fn get_user(&self, viewer: &Viewer, id: u32) -> ServiceResult<fleet_types::User> {
        authz::authorize(viewer, Subject::User, Action::Read)?;
        self.ds.user_by_id(id).await
    }

    /// Gets a user by ID without authorization checks.
    pub async fn get_user_unauthorized(&self, id: u32) -> ServiceResult<fleet_types::User> {
        self.ds.user_by_id(id).await
    }

    /// Returns the currently authenticated user from the viewer context.
    pub async fn authenticated_user(&self, viewer: &Viewer) -> ServiceResult<fleet_types::User> {
        self.ds.user_by_id(viewer.user_id()).await
    }

    /// Modifies a user's properties.
    pub async fn modify_user(
        &self,
        viewer: &Viewer,
        user_id: u32,
        payload: ModifyUserPayload,
    ) -> ServiceResult<fleet_types::User> {
        authz::authorize(viewer, Subject::User, Action::Write)?;

        let mut user = self.ds.user_by_id(user_id).await?;

        if let Some(name) = payload.name {
            user.name = name;
        }
        if let Some(email) = payload.email {
            user.email = email;
        }
        if let Some(position) = payload.position {
            user.position = position;
        }
        if let Some(global_role) = payload.global_role {
            user.global_role = global_role;
        }
        if let Some(teams) = payload.teams {
            user.teams = teams;
        }
        if let Some(sso_enabled) = payload.sso_enabled {
            user.sso_enabled = sso_enabled;
        }
        if let Some(mfa_enabled) = payload.mfa_enabled {
            user.mfa_enabled = mfa_enabled;
        }

        self.ds.save_user(&user).await
    }

    /// Deletes a user.
    pub async fn delete_user(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::User> {
        authz::authorize(viewer, Subject::User, Action::Write)?;

        let user = self.ds.user_by_id(id).await?;
        self.ds.delete_user(id).await?;

        info!(user_id = id, email = %user.email, "user deleted");
        Ok(user)
    }

    /// Lists all users matching the given options.
    pub async fn list_users(
        &self,
        viewer: &Viewer,
        opts: fleet_types::user::UserListOptions,
    ) -> ServiceResult<Vec<fleet_types::User>> {
        authz::authorize(viewer, Subject::User, Action::Read)?;
        self.ds.list_users(opts).await
    }

    /// Requires a user to reset their password on next login.
    pub async fn require_password_reset(
        &self,
        viewer: &Viewer,
        uid: u32,
        require: bool,
    ) -> ServiceResult<fleet_types::User> {
        authz::authorize(viewer, Subject::User, Action::Write)?;

        let mut user = self.ds.user_by_id(uid).await?;
        user.admin_forced_password_reset = require;

        if require {
            self.ds.destroy_all_sessions_for_user(uid).await?;
        }

        self.ds.save_user(&user).await
    }

    /// Changes the current user's password.
    pub async fn change_password(
        &self,
        viewer: &Viewer,
        old_password: &str,
        new_password: &str,
    ) -> ServiceResult<()> {
        let user = self.ds.user_by_id(viewer.user_id()).await?;

        if !auth::verify_password(old_password, &user.password)? {
            return Err(ServiceError::auth_failed("invalid old password"));
        }

        let new_hash = auth::hash_password(new_password, self.config.auth.bcrypt_cost)?;
        let mut user = user;
        user.password = new_hash;
        user.admin_forced_password_reset = false;
        self.ds.save_user(&user).await?;

        Ok(())
    }

    /// Initiates a password reset request.
    pub async fn request_password_reset(&self, email: &str) -> ServiceResult<()> {
        let user = self.ds.user_by_email(email).await?;

        if user.sso_enabled {
            return Err(ServiceError::bad_request(
                "Password reset is not available for SSO users.",
            ));
        }

        let token = auth::generate_random_text(self.config.app.token_key_size)?;
        let expires_at = self.clock.now() + chrono::Duration::hours(24);

        self.ds
            .new_password_reset_request(user.id, expires_at, &token)
            .await?;

        info!(user_id = user.id, "password reset requested");
        Ok(())
    }

    /// Resets a password using a valid reset token.
    pub async fn reset_password(&self, token: &str, new_password: &str) -> ServiceResult<()> {
        let reset_req = self.ds.find_password_reset_by_token(token).await?;

        if self.clock.now() > reset_req.expires_at {
            return Err(ServiceError::auth_failed("password reset token has expired"));
        }

        let mut user = self.ds.user_by_id(reset_req.user_id).await?;
        user.password = auth::hash_password(new_password, self.config.auth.bcrypt_cost)?;
        user.admin_forced_password_reset = false;
        self.ds.save_user(&user).await?;

        self.ds
            .delete_password_reset_requests_for_user(user.id )
            .await?;

        info!(user_id = user.id, "password reset completed");
        Ok(())
    }

    /// Creates the initial (first) user. Skips authorization.
    pub async fn create_initial_user(
        &self,
        payload: CreateUserPayload,
    ) -> ServiceResult<fleet_types::User> {
        let existing = self
            .ds
            .list_users(fleet_types::user::UserListOptions::default())
            .await?;
        if !existing.is_empty() {
            return Err(ServiceError::bad_request(
                "a user already exists; cannot create initial user",
            ));
        }

        let password_hash = if let Some(ref pwd) = payload.password {
            auth::hash_password(pwd, self.config.auth.bcrypt_cost)?
        } else {
            return Err(ServiceError::invalid_argument(
                "password",
                "password is required for initial user",
            ));
        };

        let now = chrono::Utc::now();
        let user = fleet_types::User {
            id: 0,
            created_at: now,
            updated_at: now,
            name: payload.name,
            email: payload.email,
            password: password_hash,
            salt: String::new(),
            admin_forced_password_reset: false,
            gravatar_url: String::new(),
            global_role: Some("admin".to_string()),
            sso_enabled: false,
            mfa_enabled: false,
            api_only: false,
            position: String::new(),
            teams: Vec::new(),
            settings: None,
            deleted: false,
            invite_id: None,
        };

        let created = self.ds.new_user(&user).await?;
        info!(user_id = created.id, "initial user created");
        Ok(created)
    }

    /// Gets user settings.
    pub async fn get_user_settings(
        &self,
        viewer: &Viewer,
        user_id: u32,
    ) -> ServiceResult<Option<serde_json::Value>> {
        authz::authorize(viewer, Subject::User, Action::Read)?;
        self.ds.user_settings(user_id).await
    }

    /// Saves user settings.
    pub async fn save_user_settings(
        &self,
        viewer: &Viewer,
        user_id: u32,
        settings: &serde_json::Value,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::User, Action::Write)?;
        self.ds.save_user_settings(user_id, settings).await
    }

    /// Confirms a pending email change using a token.
    ///
    /// Corresponds to Go's `(svc *Service) ChangeUserEmail`.
    pub async fn change_email(
        &self,
        viewer: &Viewer,
        token: &str,
    ) -> ServiceResult<String> {
        authz::authorize(viewer, Subject::User, Action::Write)?;
        self.ds.confirm_pending_email_change(viewer.user_id(), token).await
    }

    /// Applies user role specifications (batch update roles).
    ///
    /// Corresponds to Go's `(svc *Service) ApplyUserRolesSpecs`.
    pub async fn apply_user_role_specs(
        &self,
        viewer: &Viewer,
        specs: UserRoleSpecs,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::User, Action::Write)?;

        let mut users = Vec::new();
        for (email, spec) in &specs.roles {
            let mut user = self.ds.user_by_email(email).await?;

            // If an admin is being downgraded, ensure at least one other admin remains
            if user.global_role.as_deref() == Some("admin")
                && spec.global_role.as_deref() != Some("admin")
            {
                let all_users = self
                    .ds
                    .list_users(fleet_types::user::UserListOptions::default())
                    .await?;
                let admins_except_current = all_users
                    .iter()
                    .filter(|u| u.email != *email)
                    .filter(|u| u.global_role.as_deref() == Some("admin"))
                    .count();
                if admins_except_current == 0 {
                    return Err(ServiceError::bad_request(
                        "You need at least one admin",
                    ));
                }
            }

            user.global_role = spec.global_role.clone();

            let mut teams = Vec::new();
            for team_spec in &spec.teams {
                let team = self.ds.team_by_name(&team_spec.name).await.map_err(|e| {
                    match e {
                        ServiceError::NotFound(_) => ServiceError::bad_request(e.to_string()),
                        other => other,
                    }
                })?;
                teams.push(fleet_types::team::UserTeam {
                    team,
                    role: team_spec.role.clone(),
                });
            }
            user.teams = teams;
            users.push(user);
        }

        self.ds.save_users(&users).await
    }
}

/// Specification for a user's role.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserRoleSpec {
    pub global_role: Option<String>,
    #[serde(default)]
    pub teams: Vec<UserRoleTeamSpec>,
}

/// Specification for a user's role within a team.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserRoleTeamSpec {
    pub name: String,
    pub role: String,
}

/// Batch user role specifications.
#[derive(Debug, Clone, serde::Deserialize)]
pub struct UserRoleSpecs {
    pub roles: std::collections::HashMap<String, UserRoleSpec>,
}

/// Payload for creating a user (admin-initiated).
#[derive(Debug, Clone, Default)]
pub struct CreateUserPayload {
    pub name: String,
    pub email: String,
    pub password: Option<String>,
    pub sso_enabled: bool,
    pub api_only: bool,
    pub global_role: Option<String>,
    pub admin_forced_password_reset: Option<bool>,
    pub teams: Option<Vec<fleet_types::team::UserTeam>>,
    pub position: Option<String>,
    pub mfa_enabled: Option<bool>,
}

/// Payload for creating a user from an invitation.
#[derive(Debug, Clone)]
pub struct CreateUserFromInvitePayload {
    pub name: String,
    pub email: String,
    pub password: String,
    pub invite_token: String,
}

/// Payload for modifying an existing user.
#[derive(Debug, Clone, Default)]
pub struct ModifyUserPayload {
    pub name: Option<String>,
    pub email: Option<String>,
    pub position: Option<String>,
    pub global_role: Option<Option<String>>,
    pub teams: Option<Vec<fleet_types::team::UserTeam>>,
    pub sso_enabled: Option<bool>,
    pub mfa_enabled: Option<bool>,
}
