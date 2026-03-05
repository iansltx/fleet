//! Setup experience service operations.
//!
//! Implements setup experience script and software management.

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceResult, Viewer};

impl FleetService {
    /// Gets the setup experience script for a team (or global).
    pub async fn get_setup_experience_script(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
    ) -> ServiceResult<fleet_types::certificate::SetupExperienceScript> {
        authz::authorize(viewer, Subject::AppConfig, Action::Read)?;
        self.ds.get_setup_experience_script(team_id).await
    }

    /// Deletes the setup experience script for a team (or global).
    pub async fn delete_setup_experience_script(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::AppConfig, Action::Write)?;
        self.ds.delete_setup_experience_script(team_id).await
    }

    /// Lists software title IDs configured for setup experience.
    pub async fn list_setup_experience_software(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
    ) -> ServiceResult<Vec<u32>> {
        authz::authorize(viewer, Subject::AppConfig, Action::Read)?;
        self.ds.list_setup_experience_software_title_ids(team_id).await
    }

    /// Sets which software titles should be installed during setup experience.
    pub async fn set_setup_experience_software(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
        title_ids: &[u32],
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::AppConfig, Action::Write)?;
        self.ds.set_setup_experience_software(team_id, title_ids).await
    }
}
