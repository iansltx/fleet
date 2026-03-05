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
}
