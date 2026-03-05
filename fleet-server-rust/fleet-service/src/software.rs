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
}
