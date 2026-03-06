//! MDM service operations.
//!
//! Implements MDM configuration profile listing, command listing,
//! summary endpoints, and host profile status.

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceResult, Viewer};

impl FleetService {
    /// Lists MDM configuration profiles for a team.
    pub async fn list_mdm_config_profiles(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
        page: u32,
        per_page: u32,
    ) -> ServiceResult<Vec<fleet_types::mdm::MDMConfigProfilePayload>> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.ds.list_mdm_config_profiles(team_id, page, per_page).await
    }

    /// Gets a single MDM config profile by UUID.
    pub async fn get_mdm_config_profile(
        &self,
        viewer: &Viewer,
        profile_uuid: &str,
    ) -> ServiceResult<fleet_types::mdm::MDMConfigProfilePayload> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.ds.get_mdm_config_profile(profile_uuid).await
    }

    /// Deletes an MDM config profile by UUID.
    pub async fn delete_mdm_config_profile(
        &self,
        viewer: &Viewer,
        profile_uuid: &str,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.ds.delete_mdm_config_profile(profile_uuid).await
    }

    /// Gets MDM profiles summary counts.
    pub async fn get_mdm_profiles_summary(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
    ) -> ServiceResult<fleet_types::mdm::MDMProfilesSummary> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.ds.get_mdm_profiles_summary(team_id).await
    }

    /// Gets Apple-specific profiles summary (same data as generic summary).
    pub async fn get_mdm_apple_profiles_summary(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
    ) -> ServiceResult<fleet_types::mdm::MDMProfilesSummary> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.ds.get_mdm_profiles_summary(team_id).await
    }

    /// Lists MDM commands with pagination.
    pub async fn list_mdm_commands(
        &self,
        viewer: &Viewer,
        page: u32,
        per_page: u32,
    ) -> ServiceResult<Vec<fleet_types::mdm::MDMCommand>> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.ds.list_mdm_commands(page, per_page).await
    }

    /// Gets MDM command results by command UUID.
    pub async fn get_mdm_command_results(
        &self,
        viewer: &Viewer,
        command_uuid: &str,
    ) -> ServiceResult<Vec<fleet_types::mdm::MDMCommandResult>> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.ds.get_mdm_command_results(command_uuid).await
    }

    /// Gets MDM profiles for a specific host.
    pub async fn get_host_mdm_profiles(
        &self,
        viewer: &Viewer,
        host_uuid: &str,
    ) -> ServiceResult<Vec<fleet_types::mdm::HostMDMProfile>> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.ds.get_host_mdm_profiles(host_uuid).await
    }

    /// Gets disk encryption summary counts.
    pub async fn get_mdm_disk_encryption_summary(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
    ) -> ServiceResult<fleet_types::mdm::MDMDiskEncryptionSummary> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.ds.get_mdm_disk_encryption_summary(team_id).await
    }

    /// Gets Apple FileVault summary counts.
    pub async fn get_mdm_apple_filevault_summary(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
    ) -> ServiceResult<fleet_types::mdm::MDMAppleFileVaultSummary> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.ds.get_mdm_apple_filevault_summary(team_id).await
    }

    /// Gets status counts for a specific MDM config profile.
    pub async fn get_mdm_config_profile_status(
        &self,
        viewer: &Viewer,
        profile_uuid: &str,
        page: u32,
        per_page: u32,
    ) -> ServiceResult<fleet_types::mdm::MDMConfigProfileStatus> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.ds.get_mdm_config_profile_status(profile_uuid, page, per_page).await
    }

    // ---------------------------------------------------------------
    // Premium-gated MDM methods
    // ---------------------------------------------------------------

    /// Updates MDM Apple setup configuration for a team.
    pub async fn update_mdm_apple_setup(
        &self,
        viewer: &Viewer,
        _team_id: Option<u32>,
        _enable_end_user_authentication: Option<bool>,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Enqueues an MDM Apple command for the given device IDs.
    pub async fn enqueue_mdm_apple_command(
        &self,
        viewer: &Viewer,
        _command: &serde_json::Value,
        _device_ids: &[String],
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Creates an MDM Apple setup assistant.
    pub async fn create_mdm_apple_setup_assistant(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Gets an MDM Apple setup assistant for a team.
    pub async fn get_mdm_apple_setup_assistant(
        &self,
        viewer: &Viewer,
        _team_id: Option<u32>,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Deletes an MDM Apple setup assistant for a team.
    pub async fn delete_mdm_apple_setup_assistant(
        &self,
        viewer: &Viewer,
        _team_id: Option<u32>,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Uploads a bootstrap package.
    pub async fn upload_bootstrap_package(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Gets bootstrap package metadata for a team.
    pub async fn get_bootstrap_package_metadata(
        &self,
        viewer: &Viewer,
        _team_id: u32,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Deletes a bootstrap package for a team.
    pub async fn delete_bootstrap_package(
        &self,
        viewer: &Viewer,
        _team_id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Gets bootstrap package summary counts.
    pub async fn get_bootstrap_package_summary(
        &self,
        viewer: &Viewer,
        _team_id: Option<u32>,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Locks a device by host ID.
    pub async fn device_lock(
        &self,
        viewer: &Viewer,
        _host_id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Wipes a device by host ID.
    pub async fn device_wipe(
        &self,
        viewer: &Viewer,
        _host_id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Runs an MDM command on the given host UUIDs.
    pub async fn run_mdm_command(
        &self,
        viewer: &Viewer,
        _command: &str,
        _host_uuids: &[String],
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Unenrolls a host from MDM.
    pub async fn mdm_unenroll(
        &self,
        viewer: &Viewer,
        _host_id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Gets the encryption key for a host.
    pub async fn get_host_encryption_key(
        &self,
        viewer: &Viewer,
        _host_id: u32,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Updates MDM Apple settings (e.g., disk encryption).
    pub async fn update_mdm_apple_settings(
        &self,
        viewer: &Viewer,
        _enable_disk_encryption: Option<bool>,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Updates disk encryption setting for a team.
    pub async fn update_disk_encryption(
        &self,
        viewer: &Viewer,
        _team_id: Option<u32>,
        _enable_disk_encryption: Option<bool>,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Creates a new MDM config profile.
    pub async fn new_mdm_config_profile(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Batch-sets MDM profiles for a team.
    pub async fn batch_set_mdm_profiles(
        &self,
        viewer: &Viewer,
        _profiles: &serde_json::Value,
        _team_id: Option<u32>,
        _dry_run: bool,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Resends an MDM profile to a specific host.
    pub async fn resend_host_mdm_profile(
        &self,
        viewer: &Viewer,
        _host_id: u32,
        _profile_uuid: &str,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Batch-resends an MDM profile to multiple hosts.
    pub async fn batch_resend_mdm_profile_to_hosts(
        &self,
        viewer: &Viewer,
        _profile_uuid: &str,
        _host_ids: &[u32],
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Creates an MDM EULA.
    pub async fn create_mdm_eula(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Gets MDM EULA metadata.
    pub async fn get_mdm_eula_metadata(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Deletes an MDM EULA by token.
    pub async fn delete_mdm_eula(
        &self,
        viewer: &Viewer,
        _token: &str,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Preassigns an MDM Apple profile.
    pub async fn preassign_mdm_apple_profile(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Matches MDM Apple preassignment by external host identifier.
    pub async fn match_mdm_apple_preassignment(
        &self,
        viewer: &Viewer,
        _external_host_identifier: &str,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Gets the Apple MDM configuration.
    pub async fn get_apple_mdm(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Gets the manual enrollment profile.
    pub async fn get_manual_enrollment_profile(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<Vec<u8>> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Uploads an ABM token.
    pub async fn upload_abm_token(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Deletes an ABM token by ID.
    pub async fn delete_abm_token(
        &self,
        viewer: &Viewer,
        _id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Lists all ABM tokens.
    pub async fn list_abm_tokens(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Counts ABM tokens.
    pub async fn count_abm_tokens(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<u32> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Updates ABM token team assignments.
    pub async fn update_abm_token_teams(
        &self,
        viewer: &Viewer,
        _id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Renews an ABM token.
    pub async fn renew_abm_token(
        &self,
        viewer: &Viewer,
        _id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Gets all VPP tokens.
    pub async fn get_vpp_tokens(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Mdm, Action::Read)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Uploads a VPP token.
    pub async fn upload_vpp_token(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Updates VPP token team assignments.
    pub async fn patch_vpp_tokens_teams(
        &self,
        viewer: &Viewer,
        _id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Renews a VPP token.
    pub async fn patch_vpp_token_renew(
        &self,
        viewer: &Viewer,
        _id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Deletes a VPP token by ID.
    pub async fn delete_vpp_token(
        &self,
        viewer: &Viewer,
        _id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Mdm, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }
}
