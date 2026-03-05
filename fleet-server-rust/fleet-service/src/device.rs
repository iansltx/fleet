//! Device service operations.
//!
//! Implements device-authenticated operations for Fleet Desktop.
//! Corresponds to Go's `server/service/device_client.go` and related.

use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult};

impl FleetService {
    /// Authenticates a device by its device auth token.
    ///
    /// Returns the host associated with the token.
    pub async fn authenticate_device(
        &self,
        token: &str,
    ) -> ServiceResult<fleet_types::Host> {
        if token.is_empty() {
            return Err(ServiceError::auth_failed("missing device auth token"));
        }
        let host = self.ds.load_host_by_device_auth_token(token).await?;

        // Mark host as seen.
        let now = self.clock.now();
        if let Err(e) = self.ds.mark_host_seen(host.id, now).await {
            tracing::warn!("Failed to mark device host seen: {}", e);
        }

        Ok(host)
    }

    /// Returns the full host info for a device-authenticated request.
    pub async fn get_device_host(
        &self,
        token: &str,
    ) -> ServiceResult<fleet_types::HostDetail> {
        let host = self.authenticate_device(token).await?;

        let host_id = host.id;
        let policies = self.ds.list_policies_for_host(host_id).await.unwrap_or_default();
        let software = self.ds.list_software_for_host(host_id).await.unwrap_or_default();
        Ok(fleet_types::HostDetail {
            host,
            labels: Vec::new(),
            packs: Vec::new(),
            policies,
            software,
        })
    }

    /// Returns the Fleet Desktop summary info for the host.
    pub async fn get_fleet_desktop(
        &self,
        token: &str,
    ) -> ServiceResult<serde_json::Value> {
        let host = self.authenticate_device(token).await?;

        // Count failing policies.
        let policies = self.ds.list_policies_for_host(host.id).await?;
        let failing_policies = policies.iter().filter(|p| p.response == "fail").count();

        Ok(serde_json::json!({
            "failing_policies_count": failing_policies,
            "notifications": {},
            "config": {}
        }))
    }

    /// Lists policies for the authenticated device's host.
    pub async fn list_device_policies(
        &self,
        token: &str,
    ) -> ServiceResult<Vec<fleet_types::policy::HostPolicy>> {
        let host = self.authenticate_device(token).await?;
        self.ds.list_policies_for_host(host.id).await
    }

    /// Returns the transparency URL from the app config.
    pub async fn get_transparency_url(&self) -> ServiceResult<String> {
        let config = self.ds.app_config().await?;
        Ok(config.transparency_url)
    }

    /// Returns device mapping (email addresses) for the host.
    pub async fn get_device_mapping(
        &self,
        token: &str,
    ) -> ServiceResult<serde_json::Value> {
        let host = self.authenticate_device(token).await?;
        self.ds.device_mapping_for_host(host.id).await
    }

    /// Marks the device's host for refetch.
    pub async fn refetch_device_host(
        &self,
        token: &str,
    ) -> ServiceResult<()> {
        let host = self.authenticate_device(token).await?;
        self.ds.mark_host_refetch_requested(host.id).await
    }

    /// Lists software for the authenticated device's host.
    pub async fn list_device_software(
        &self,
        token: &str,
    ) -> ServiceResult<Vec<fleet_types::Software>> {
        let host = self.authenticate_device(token).await?;
        self.ds.list_software_for_host(host.id).await
    }

    /// Returns macadmins data (Munki/MDM info) for the device's host.
    pub async fn get_device_macadmins_data(
        &self,
        token: &str,
    ) -> ServiceResult<serde_json::Value> {
        let _host = self.authenticate_device(token).await?;
        // Macadmins data includes Munki info, MDM status, and Munki issues.
        // Without Munki/MDM enrollment, we return the structured empty response.
        Ok(serde_json::json!({
            "munki": null,
            "mobile_device_management": null,
            "munki_issues": []
        }))
    }

    /// Queues a self-service software install for the device's host.
    pub async fn submit_self_service_software_install(
        &self,
        token: &str,
        software_title_id: u64,
    ) -> ServiceResult<String> {
        let host = self.authenticate_device(token).await?;
        let execution_id = uuid::Uuid::new_v4().to_string();
        let result = fleet_types::script::HostScriptResult {
            id: 0,
            host_id: host.id,
            execution_id: execution_id.clone(),
            script_id: None,
            script_contents: format!("install_software_title:{}", software_title_id),
            output: String::new(),
            runtime: 0,
            exit_code: None,
            message: Some(format!("Self-service install of software title {}", software_title_id)),
            host_timeout: false,
            host_deleted_at: None,
            created_at: self.clock.now(),
            updated_at: self.clock.now(),
        };
        self.ds.save_host_script_result(&result).await?;
        Ok(execution_id)
    }

    /// Queues a self-service software uninstall for the device's host.
    pub async fn submit_device_software_uninstall(
        &self,
        token: &str,
        software_title_id: u64,
    ) -> ServiceResult<String> {
        let host = self.authenticate_device(token).await?;
        let execution_id = uuid::Uuid::new_v4().to_string();
        let result = fleet_types::script::HostScriptResult {
            id: 0,
            host_id: host.id,
            execution_id: execution_id.clone(),
            script_id: None,
            script_contents: format!("uninstall_software_title:{}", software_title_id),
            output: String::new(),
            runtime: 0,
            exit_code: None,
            message: Some(format!("Self-service uninstall of software title {}", software_title_id)),
            host_timeout: false,
            host_deleted_at: None,
            created_at: self.clock.now(),
            updated_at: self.clock.now(),
        };
        self.ds.save_host_script_result(&result).await?;
        Ok(execution_id)
    }

    /// Returns software install results for a device.
    pub async fn get_device_software_install_results(
        &self,
        token: &str,
        install_uuid: &str,
    ) -> ServiceResult<fleet_types::software::SoftwareInstallResult> {
        let _host = self.authenticate_device(token).await?;
        self.ds.get_software_install_result(install_uuid).await
    }

    /// Returns software uninstall results for a device.
    pub async fn get_device_software_uninstall_results(
        &self,
        token: &str,
        execution_id: &str,
    ) -> ServiceResult<fleet_types::script::HostScriptResult> {
        let _host = self.authenticate_device(token).await?;
        self.ds.get_host_script_execution(execution_id).await
    }

    /// Returns setup experience status for the device.
    pub async fn get_device_setup_experience_status(
        &self,
        token: &str,
    ) -> ServiceResult<serde_json::Value> {
        let host = self.authenticate_device(token).await?;
        // Check if setup experience software is configured for the host's team
        let team_id = host.team_id;
        let software_title_ids = self.ds.list_setup_experience_software_title_ids(team_id).await
            .unwrap_or_default();

        // Return the current status - software entries are pending until installed
        let software: Vec<serde_json::Value> = software_title_ids.iter().map(|id| {
            serde_json::json!({
                "software_title_id": id,
                "status": "pending",
                "name": "",
            })
        }).collect();

        Ok(serde_json::json!({
            "software": software,
            "profiles": [],
        }))
    }
}
