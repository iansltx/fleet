//! Software service operations.
//!
//! Implements software listing and retrieval.

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceResult, Viewer};

impl FleetService {
    /// Lists software with optional team filter.
    pub async fn list_software(
        &self,
        viewer: &Viewer,
        opts: fleet_types::ListOptions,
        team_id: Option<u32>,
    ) -> ServiceResult<Vec<fleet_types::Software>> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.ds.list_software(opts, team_id).await
    }

    /// Gets software by ID.
    pub async fn get_software(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::Software> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.ds.software_by_id(id).await
    }

    /// Lists software titles with optional team filter and pagination.
    pub async fn list_software_titles(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
        limit: u32,
        offset: u32,
    ) -> ServiceResult<Vec<fleet_types::Software>> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.ds.list_software_titles(team_id, limit, offset).await
    }

    /// Gets a software title by ID (delegates to software_by_id).
    pub async fn get_software_title(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::Software> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.ds.software_by_id(id).await
    }

    /// Updates a software title name.
    pub async fn update_software_title_name(
        &self,
        viewer: &Viewer,
        id: u32,
        name: &str,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.ds.update_software_title_name(id, name).await
    }

    /// Deletes a software installer for a title.
    pub async fn delete_software_installer(
        &self,
        viewer: &Viewer,
        title_id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.ds.delete_software_installer(title_id).await
    }

    /// Deletes a software title icon.
    pub async fn delete_software_title_icon(
        &self,
        viewer: &Viewer,
        title_id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.ds.delete_software_title_icon(title_id).await
    }

    /// Lists vulnerabilities with host counts.
    pub async fn list_vulnerabilities(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
        query: Option<&str>,
        exploit: Option<bool>,
        limit: u32,
        offset: u32,
    ) -> ServiceResult<Vec<fleet_types::vulnerability::VulnerabilityWithMetadata>> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.ds.list_vulnerabilities(team_id, query, exploit, limit, offset).await
    }

    /// Gets a single vulnerability by CVE.
    pub async fn get_vulnerability(
        &self,
        viewer: &Viewer,
        cve: &str,
        team_id: Option<u32>,
    ) -> ServiceResult<fleet_types::vulnerability::VulnerabilityWithMetadata> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.ds.get_vulnerability(cve, team_id).await
    }

    /// Gets a software install result by execution ID.
    pub async fn get_software_install_result(
        &self,
        viewer: &Viewer,
        execution_id: &str,
    ) -> ServiceResult<fleet_types::software::SoftwareInstallResult> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.ds.get_software_install_result(execution_id).await
    }

    /// Lists fleet maintained apps.
    pub async fn list_fleet_maintained_apps(
        &self,
        viewer: &Viewer,
        query: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> ServiceResult<Vec<fleet_types::software::FleetMaintainedApp>> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.ds.list_fleet_maintained_apps(query, limit, offset).await
    }

    /// Gets a fleet maintained app by ID.
    pub async fn get_fleet_maintained_app(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::software::FleetMaintainedApp> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.ds.get_fleet_maintained_app(id).await
    }

    // ---- Premium-gated software methods ----

    /// Downloads a software installer binary (premium-only).
    pub async fn get_software_installer(
        &self,
        viewer: &Viewer,
        title_id: u32,
    ) -> ServiceResult<Vec<u8>> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.require_premium()?;
        let _ = title_id;
        // TODO: implement with datastore/blob store operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Generates a download token for a software installer (premium-only).
    pub async fn get_software_installer_token(
        &self,
        viewer: &Viewer,
        title_id: u32,
    ) -> ServiceResult<String> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.require_premium()?;
        let _ = title_id;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Uploads a new software installer (premium-only).
    pub async fn upload_software_installer(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.require_premium()?;
        // TODO: implement with datastore/blob store operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Updates an existing software installer (premium-only).
    pub async fn update_software_installer(
        &self,
        viewer: &Viewer,
        title_id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.require_premium()?;
        let _ = title_id;
        // TODO: implement with datastore/blob store operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Batch sets software installers (premium-only).
    pub async fn batch_set_software_installers(
        &self,
        viewer: &Viewer,
        software: &[serde_json::Value],
        team_id: Option<u32>,
        dry_run: bool,
    ) -> ServiceResult<String> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.require_premium()?;
        let _ = (software, team_id, dry_run);
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Gets the result of a batch software installer set operation (premium-only).
    pub async fn batch_set_software_installers_result(
        &self,
        viewer: &Viewer,
        request_uuid: &str,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.require_premium()?;
        let _ = request_uuid;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Gets a software title icon (premium-only).
    pub async fn get_software_title_icon(
        &self,
        viewer: &Viewer,
        title_id: u32,
    ) -> ServiceResult<Vec<u8>> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.require_premium()?;
        let _ = title_id;
        // TODO: implement with datastore/blob store operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Uploads a software title icon (premium-only).
    pub async fn put_software_title_icon(
        &self,
        viewer: &Viewer,
        title_id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.require_premium()?;
        let _ = title_id;
        // TODO: implement with datastore/blob store operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Lists app store apps (premium-only).
    pub async fn get_app_store_apps(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
        platform: Option<&str>,
    ) -> ServiceResult<Vec<serde_json::Value>> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.require_premium()?;
        let _ = (team_id, platform);
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Adds an app store app (premium-only).
    pub async fn add_app_store_app(
        &self,
        viewer: &Viewer,
        app_store_id: &str,
        team_id: Option<u32>,
        platform: Option<&str>,
        self_service: bool,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.require_premium()?;
        let _ = (app_store_id, team_id, platform, self_service);
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Updates an app store app (premium-only).
    pub async fn update_app_store_app(
        &self,
        viewer: &Viewer,
        title_id: u32,
        team_id: Option<u32>,
        self_service: Option<bool>,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.require_premium()?;
        let _ = (title_id, team_id, self_service);
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Adds a Fleet-maintained app (premium-only).
    pub async fn add_fleet_maintained_app_installer(
        &self,
        viewer: &Viewer,
        fleet_maintained_app_id: Option<u64>,
        team_id: Option<u32>,
        self_service: bool,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.require_premium()?;
        let _ = (fleet_maintained_app_id, team_id, self_service);
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Batch associates app store apps (premium-only).
    pub async fn batch_associate_app_store_apps(
        &self,
        viewer: &Viewer,
        app_store_apps: &[serde_json::Value],
        team_id: Option<u32>,
        dry_run: bool,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.require_premium()?;
        let _ = (app_store_apps, team_id, dry_run);
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Creates an Android web app (premium-only).
    pub async fn create_android_web_app(
        &self,
        viewer: &Viewer,
        data: &serde_json::Value,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Software, Action::Write)?;
        self.require_premium()?;
        let _ = data;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Downloads a software installer via token (premium-only).
    pub async fn download_software_installer(
        &self,
        viewer: &Viewer,
        title_id: u32,
        token: &str,
    ) -> ServiceResult<Vec<u8>> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.require_premium()?;
        let _ = (title_id, token);
        // TODO: implement with datastore/blob store operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Downloads an in-house app package (premium-only).
    pub async fn get_in_house_app_package(
        &self,
        viewer: &Viewer,
        title_id: u32,
    ) -> ServiceResult<Vec<u8>> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.require_premium()?;
        let _ = title_id;
        // TODO: implement with datastore/blob store operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }

    /// Returns an in-house app manifest (premium-only).
    pub async fn get_in_house_app_manifest(
        &self,
        viewer: &Viewer,
        title_id: u32,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Software, Action::Read)?;
        self.require_premium()?;
        let _ = title_id;
        // TODO: implement with datastore operation
        Err(crate::ServiceError::Internal("not yet implemented".to_string()))
    }
}
