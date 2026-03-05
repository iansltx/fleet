//! Script service operations.
//!
//! Implements script management business logic.
//! Corresponds to Go's `server/service/scripts.go`.

use tracing::info;

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult, Viewer};

impl FleetService {
    /// Creates a new script.
    pub async fn create_script(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
        name: &str,
        contents: &str,
    ) -> ServiceResult<fleet_types::script::Script> {
        authz::authorize(viewer, Subject::Script, Action::Write)?;

        if name.is_empty() {
            return Err(ServiceError::invalid_argument("name", "name is required"));
        }
        if contents.is_empty() {
            return Err(ServiceError::invalid_argument(
                "script_contents",
                "script contents are required",
            ));
        }

        let script = self.ds.new_script(team_id, name, contents).await?;
        info!(script_id = script.id, name = %name, "script created");
        Ok(script)
    }

    /// Lists scripts, optionally filtered by team.
    pub async fn list_scripts(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
    ) -> ServiceResult<Vec<fleet_types::script::Script>> {
        authz::authorize(viewer, Subject::Script, Action::Read)?;
        self.ds.list_scripts(team_id).await
    }

    /// Gets a script by ID.
    pub async fn get_script(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::script::Script> {
        authz::authorize(viewer, Subject::Script, Action::Read)?;
        self.ds.script_by_id(id).await
    }

    /// Gets script contents by script ID.
    pub async fn get_script_contents(
        &self,
        viewer: &Viewer,
        script_id: u32,
    ) -> ServiceResult<String> {
        authz::authorize(viewer, Subject::Script, Action::Read)?;
        self.ds.get_script_contents(script_id).await
    }

    /// Deletes a script by ID.
    pub async fn delete_script(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Script, Action::Write)?;
        self.ds.delete_script(id).await?;
        info!(script_id = id, "script deleted");
        Ok(())
    }
}
