//! Session management service.
//!
//! Handles login, logout, session creation/validation, and MFA flow.
//! Corresponds to Go's `server/service/sessions.go`.

use std::time::Duration;

use tracing::{info, warn};

use crate::auth;
use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult, Viewer};

/// SSO settings returned to unauthenticated users for login page display.
#[derive(Debug, Clone, serde::Serialize)]
pub struct SsoSettings {
    pub sso_enabled: bool,
    pub idp_name: String,
    pub idp_image_url: String,
}

impl FleetService {
    /// Authenticates a user by email/password and creates a new session.
    ///
    /// Corresponds to Go's `(svc *Service) Login`.
    /// Returns the user and session on success.
    /// On failure, enforces a minimum 1-second delay to frustrate timing attacks.
    pub async fn login(
        &self,
        email: &str,
        password: &str,
    ) -> ServiceResult<(fleet_types::User, fleet_types::Session)> {
        let start = std::time::Instant::now();

        let result = self.login_user(email, password).await;

        // On failure, wait until at least 1 second has elapsed.
        if result.is_err() {
            let elapsed = start.elapsed();
            if elapsed < Duration::from_secs(1) {
                tokio::time::sleep(Duration::from_secs(1) - elapsed).await;
            }
        }

        result
    }

    /// Internal login logic, separated from timing delay.
    pub(crate) async fn login_user(
        &self,
        email: &str,
        password: &str,
    ) -> ServiceResult<(fleet_types::User, fleet_types::Session)> {
        let user = self.ds.user_by_email(email).await.map_err(|e| {
            match e {
                ServiceError::NotFound(_) => ServiceError::auth_failed("user not found"),
                _ => ServiceError::auth_failed(e.to_string()),
            }
        })?;

        // Validate password.
        if !auth::verify_password(password, &user.password)? {
            return Err(ServiceError::auth_failed("invalid password"));
        }

        // SSO users cannot use password login.
        if user.sso_enabled {
            return Err(ServiceError::auth_failed(
                "password login disabled for SSO users",
            ));
        }

        // MFA users need a separate flow (email verification).
        if user.mfa_enabled {
            return Err(ServiceError::bad_request(
                "MFA is enabled; use the MFA login flow",
            ));
        }

        // Create session.
        let session = self.make_session(user.id).await?;

        info!(user_id = user.id, email = %user.email, "user logged in");
        Ok((user, session))
    }

    /// Creates a new session for the given user.
    ///
    /// Corresponds to Go's `(svc *Service) makeSession`.
    async fn make_session(&self, user_id: u32) -> ServiceResult<fleet_types::Session> {
        self.ds
            .new_session(user_id.into(), self.config.session.key_size)
            .await
    }

    /// Logs out the current user by destroying their session.
    ///
    /// Corresponds to Go's `(svc *Service) Logout` and `DestroySession`.
    pub async fn logout(&self, viewer: &Viewer) -> ServiceResult<()> {
        let session = self.ds.session_by_id(viewer.session_id()).await?;
        self.ds.destroy_session(&session).await?;

        info!(user_id = viewer.user_id(), "user logged out");
        Ok(())
    }

    /// Gets information about a specific session by ID.
    ///
    /// Corresponds to Go's `(svc *Service) GetInfoAboutSession`.
    pub async fn get_info_about_session(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::Session> {
        let session = self.ds.session_by_id(id).await?;

        // Users can only view their own sessions, or admins can view any.
        authz::authorize(viewer, Subject::Session, Action::Read)?;

        self.validate_session(&session).await?;
        Ok(session)
    }

    /// Deletes a specific session by ID.
    ///
    /// Corresponds to Go's `(svc *Service) DeleteSession`.
    pub async fn delete_session(&self, viewer: &Viewer, id: u32) -> ServiceResult<()> {
        let session = self.ds.session_by_id(id).await?;

        authz::authorize(viewer, Subject::Session, Action::Write)?;

        self.ds.destroy_session(&session).await
    }

    /// Retrieves a session by its key (token).
    ///
    /// Corresponds to Go's `(svc *Service) GetSessionByKey`.
    pub async fn get_session_by_key(&self, key: &str) -> ServiceResult<fleet_types::Session> {
        let session = self.ds.session_by_key(key).await?;
        self.validate_session(&session).await?;
        Ok(session)
    }

    /// Returns all sessions for a user.
    ///
    /// Corresponds to Go's `(svc *Service) GetInfoAboutSessionsForUser`.
    pub async fn get_info_about_sessions_for_user(
        &self,
        viewer: &Viewer,
        user_id: u32,
    ) -> ServiceResult<Vec<fleet_types::Session>> {
        if viewer.user_id() != user_id {
            authz::authorize(viewer, authz::Subject::Session, authz::Action::Read)?;
        }
        self.ds.list_sessions_for_user(user_id).await
    }

    /// Returns SSO settings for display on the login page.
    /// Unauthenticated users need to see this to initiate SSO.
    pub async fn sso_settings(&self) -> ServiceResult<SsoSettings> {
        let config = self.ds.app_config().await?;
        Ok(SsoSettings {
            sso_enabled: config.enable_sso,
            idp_name: config.sso_idp_name,
            idp_image_url: config.sso_idp_image_url,
        })
    }

    /// Initiates SSO login flow.
    ///
    /// Corresponds to Go's `(svc *Service) InitiateSSO`.
    pub async fn initiate_sso(&self, relay_url: &str) -> ServiceResult<String> {
        let config = self.ds.app_config().await?;
        if !config.enable_sso {
            return Err(crate::ServiceError::bad_request("SSO is not enabled"));
        }
        // In a full implementation, this would generate a SAML AuthnRequest
        // and return the IdP redirect URL.
        info!("SSO initiation requested, relay_url={}", relay_url);
        Ok(String::new())
    }

    /// Handles SSO callback with SAML response.
    ///
    /// Corresponds to Go's `(svc *Service) CallbackSSO`.
    pub async fn callback_sso(
        &self,
        saml_response: &str,
    ) -> ServiceResult<(fleet_types::User, fleet_types::Session)> {
        let config = self.ds.app_config().await?;
        if !config.enable_sso {
            return Err(crate::ServiceError::bad_request("SSO is not enabled"));
        }
        // In a full implementation, this would validate the SAML response,
        // extract the user identity, and create/login the user.
        let _ = saml_response;
        Err(crate::ServiceError::bad_request("SSO callback not yet implemented"))
    }

    /// Validates that a session is still active and not expired.
    ///
    /// Corresponds to Go's `(svc *Service) validateSession`.
    /// - API-only sessions (unlimited duration) never expire.
    /// - Regular sessions expire after the configured duration.
    async fn validate_session(&self, session: &fleet_types::Session) -> ServiceResult<()> {
        let session_duration = self.config.session.duration;

        // API-only tokens have unlimited duration.
        if session.api_only.unwrap_or(false) {
            // Duration 0 = unlimited; also skip for API-only.
            self.ds.mark_session_accessed(session).await?;
            return Ok(());
        }

        // Duration 0 = unlimited.
        if !session_duration.is_zero() {
            let now = self.clock.now();
            let accessed_at = session.accessed_at;
            let elapsed = now.signed_duration_since(accessed_at);

            if elapsed.num_seconds() >= session_duration.as_secs() as i64 {
                // Session has expired -- destroy it.
                if let Err(e) = self.ds.destroy_session(session).await {
                    warn!("Failed to destroy expired session: {}", e);
                }
                return Err(ServiceError::auth_required("expired session"));
            }
        }

        self.ds.mark_session_accessed(session).await?;
        Ok(())
    }
}
