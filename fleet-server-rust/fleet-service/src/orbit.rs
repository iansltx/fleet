//! Orbit service operations.
//!
//! Implements Orbit agent operations: enrollment, device token management,
//! config delivery, script execution, software install reporting, and
//! disk encryption key escrow.
//! Corresponds to Go's `server/service/orbit.go`.

use tracing::{info, warn};

use crate::auth;
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult};

impl FleetService {
    /// Authenticates an Orbit agent by its orbit_node_key.
    ///
    /// Returns the host associated with the key.
    pub async fn authenticate_orbit(
        &self,
        orbit_node_key: &str,
    ) -> ServiceResult<fleet_types::Host> {
        if orbit_node_key.is_empty() {
            return Err(ServiceError::auth_failed("missing orbit node key"));
        }
        let host = self.ds.load_host_by_orbit_node_key(orbit_node_key).await?;

        // Mark host as seen.
        let now = self.clock.now();
        if let Err(e) = self.ds.mark_host_seen(host.id, now).await {
            warn!("Failed to mark orbit host seen: {}", e);
        }

        Ok(host)
    }

    /// Enrolls an Orbit agent.
    ///
    /// Validates the enroll secret, finds or creates a host, and
    /// returns an orbit_node_key.
    pub async fn enroll_orbit(
        &self,
        enroll_secret: &str,
        hardware_uuid: Option<&str>,
        hardware_serial: Option<&str>,
    ) -> ServiceResult<String> {
        // Verify the enroll secret.
        let secret_info = self.ds.verify_enroll_secret(enroll_secret).await
            .map_err(|e| ServiceError::auth_failed(format!("orbit enroll failed: {}", e)))?;

        // Generate an orbit node key.
        let orbit_node_key = auth::generate_random_text(self.config.osquery.node_key_size)?;

        let _host = self.ds.enroll_orbit(
            hardware_uuid.unwrap_or(""),
            hardware_serial.unwrap_or(""),
            &orbit_node_key,
            secret_info.team_id,
        ).await?;

        info!("orbit agent enrolled");

        Ok(orbit_node_key)
    }

    /// Sets or updates the device auth token for an Orbit host.
    pub async fn set_or_update_device_token(
        &self,
        orbit_node_key: &str,
        device_auth_token: &str,
    ) -> ServiceResult<()> {
        let host = self.authenticate_orbit(orbit_node_key).await?;
        self.ds.set_or_update_device_auth_token(host.id, device_auth_token).await
    }

    /// Returns the Orbit configuration for a host.
    ///
    /// This includes notifications, nudge config, and other Orbit-specific settings.
    pub async fn get_orbit_config(
        &self,
        orbit_node_key: &str,
    ) -> ServiceResult<serde_json::Value> {
        let _host = self.authenticate_orbit(orbit_node_key).await?;

        // Return basic orbit config. Full implementation would include
        // notifications, nudge config, script execution pending, etc.
        Ok(serde_json::json!({
            "notifications": {},
            "orbit_node_key": orbit_node_key
        }))
    }

    /// Returns the script content for a pending script execution.
    pub async fn get_orbit_script(
        &self,
        orbit_node_key: &str,
        execution_id: &str,
    ) -> ServiceResult<fleet_types::script::HostScriptResult> {
        let _host = self.authenticate_orbit(orbit_node_key).await?;
        self.ds.get_host_script_execution(execution_id).await
    }

    /// Records the result of a script execution from Orbit.
    pub async fn post_orbit_script_result(
        &self,
        orbit_node_key: &str,
        execution_id: &str,
        exit_code: i32,
        output: &str,
        runtime: Option<u64>,
    ) -> ServiceResult<()> {
        let host = self.authenticate_orbit(orbit_node_key).await?;

        let result = fleet_types::script::HostScriptResult {
            id: 0,
            host_id: host.id,
            execution_id: execution_id.to_string(),
            script_id: None,
            script_contents: String::new(),
            output: output.to_string(),
            runtime: runtime.unwrap_or(0) as i32,
            exit_code: Some(exit_code as i64),
            message: None,
            host_timeout: false,
            host_deleted_at: None,
            created_at: self.clock.now(),
            updated_at: self.clock.now(),
        };

        self.ds.save_host_script_result(&result).await
    }

    /// Updates device mapping (email) from Orbit.
    pub async fn put_orbit_device_mapping(
        &self,
        orbit_node_key: &str,
        _email: Option<&str>,
    ) -> ServiceResult<()> {
        let _host = self.authenticate_orbit(orbit_node_key).await?;
        // Full implementation would update host_emails table
        Ok(())
    }

    /// Records a disk encryption key escrow from Orbit.
    pub async fn post_orbit_disk_encryption_key(
        &self,
        orbit_node_key: &str,
        encryption_key: &[u8],
        client_error: Option<&str>,
    ) -> ServiceResult<()> {
        let host = self.authenticate_orbit(orbit_node_key).await?;
        self.ds.set_host_disk_encryption_key(host.id, encryption_key, client_error).await
    }

    /// Records a LUKS passphrase escrow from Orbit (Linux).
    pub async fn post_orbit_luks_data(
        &self,
        orbit_node_key: &str,
        passphrase: &str,
        _slot_key: Option<&str>,
        client_error: Option<&str>,
    ) -> ServiceResult<()> {
        let host = self.authenticate_orbit(orbit_node_key).await?;
        self.ds.set_host_disk_encryption_key(host.id, passphrase.as_bytes(), client_error).await
    }
}
