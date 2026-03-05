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

    /// Gets a script execution result by execution ID.
    pub async fn get_script_result(
        &self,
        viewer: &Viewer,
        execution_id: &str,
    ) -> ServiceResult<fleet_types::script::HostScriptResult> {
        authz::authorize(viewer, Subject::Script, Action::Read)?;
        self.ds.get_host_script_execution(execution_id).await
    }

    /// Saves a host script execution result.
    pub async fn save_host_script_result(
        &self,
        viewer: &Viewer,
        result: &fleet_types::script::HostScriptResult,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Script, Action::Write)?;
        self.ds.save_host_script_result(result).await
    }

    /// Creates a new host script execution request.
    pub async fn new_host_script_execution_request(
        &self,
        viewer: &Viewer,
        host_id: u32,
        script_id: Option<u32>,
        script_contents: &str,
        execution_id: &str,
        sync_request: bool,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Script, Action::Write)?;
        self.ds
            .new_host_script_execution_request(host_id, script_id, script_contents, execution_id, sync_request)
            .await
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

    /// Runs a script on a batch of hosts.
    pub async fn batch_run_script(
        &self,
        viewer: &Viewer,
        host_ids: &[u32],
        script_id: u32,
    ) -> ServiceResult<String> {
        authz::authorize(viewer, Subject::Script, Action::Write)?;

        let batch_execution_id = uuid::Uuid::new_v4().to_string();
        let script = self.ds.script_by_id(script_id).await?;
        let contents = self.ds.get_script_contents(script_id).await?;

        for &host_id in host_ids {
            let execution_id = format!("{}:{}", batch_execution_id, uuid::Uuid::new_v4());
            self.ds
                .new_host_script_execution_request(host_id, Some(script_id), &contents, &execution_id, false)
                .await?;
        }

        info!(
            batch_execution_id = %batch_execution_id,
            script_id = script.id,
            host_count = host_ids.len(),
            "batch script execution started"
        );
        Ok(batch_execution_id)
    }

    /// Gets batch script execution summary.
    pub async fn get_batch_script_execution_summary(
        &self,
        viewer: &Viewer,
        batch_execution_id: &str,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Script, Action::Read)?;
        self.ds.get_batch_script_execution_summary(batch_execution_id).await
    }

    /// Gets batch script execution host results.
    pub async fn list_batch_script_execution_hosts(
        &self,
        viewer: &Viewer,
        batch_execution_id: &str,
        limit: u32,
        offset: u32,
    ) -> ServiceResult<Vec<fleet_types::script::HostScriptResult>> {
        authz::authorize(viewer, Subject::Script, Action::Read)?;
        self.ds.list_batch_script_execution_hosts(batch_execution_id, limit, offset).await
    }

    /// Cancels a batch script execution.
    pub async fn cancel_batch_script_execution(
        &self,
        viewer: &Viewer,
        batch_execution_id: &str,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Script, Action::Write)?;
        self.ds.cancel_batch_script_execution(batch_execution_id).await?;
        info!(batch_execution_id = %batch_execution_id, "batch script execution cancelled");
        Ok(())
    }
}
