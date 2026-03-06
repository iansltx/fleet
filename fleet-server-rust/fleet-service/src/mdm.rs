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
}
