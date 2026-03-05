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

        Ok(fleet_types::HostDetail {
            host,
            labels: Vec::new(),
            packs: Vec::new(),
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
}
