//! Carve service operations.
//!
//! Implements file carve management business logic.
//! Corresponds to Go's `server/service/carves.go`.

use tracing::info;
use uuid::Uuid;

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult, Viewer};

/// Maximum carve size: 8GB.
const MAX_CARVE_SIZE: i64 = 8 * 1024 * 1024 * 1024;
/// Maximum block size: 256MB.
const MAX_BLOCK_SIZE: i64 = 256 * 1024 * 1024;

impl FleetService {
    /// Lists all carves.
    pub async fn list_carves(
        &self,
        viewer: &Viewer,
        include_expired: bool,
    ) -> ServiceResult<Vec<fleet_types::CarveMetadata>> {
        authz::authorize(viewer, Subject::Carve, Action::Read)?;
        self.ds.list_carves(include_expired).await
    }

    /// Gets a carve by ID.
    pub async fn get_carve(
        &self,
        viewer: &Viewer,
        id: i64,
    ) -> ServiceResult<fleet_types::CarveMetadata> {
        authz::authorize(viewer, Subject::Carve, Action::Read)?;
        self.ds.carve_by_id(id).await
    }

    /// Gets a carve block.
    pub async fn get_carve_block(
        &self,
        viewer: &Viewer,
        carve_id: i64,
        block_id: i64,
    ) -> ServiceResult<Vec<u8>> {
        authz::authorize(viewer, Subject::Carve, Action::Read)?;

        let carve = self.ds.carve_by_id(carve_id).await?;
        if carve.expired {
            return Err(ServiceError::bad_request("carve has expired"));
        }
        if block_id > carve.max_block {
            return Err(ServiceError::not_found("block not yet received"));
        }

        self.ds.get_carve_block(carve_id, block_id).await
    }

    /// Begins a new carve (called by osquery agent).
    pub async fn carve_begin(
        &self,
        host: &fleet_types::Host,
        payload: fleet_types::CarveBeginPayload,
    ) -> ServiceResult<fleet_types::CarveMetadata> {
        // Validate carve parameters
        if payload.carve_size < 0 || payload.carve_size > MAX_CARVE_SIZE {
            return Err(ServiceError::bad_request(format!(
                "carve_size must be between 0 and {} bytes",
                MAX_CARVE_SIZE
            )));
        }
        if payload.block_size < 0 || payload.block_size > MAX_BLOCK_SIZE {
            return Err(ServiceError::bad_request(format!(
                "block_size must be between 0 and {} bytes",
                MAX_BLOCK_SIZE
            )));
        }
        if payload.block_count < 0 {
            return Err(ServiceError::bad_request("block_count must be non-negative"));
        }

        let session_id = Uuid::new_v4().to_string();
        let now = self.clock.now();
        let name = format!(
            "{}-{}-{}",
            host.hostname,
            now.format("%Y-%m-%dT%H:%M:%S"),
            payload.request_id
        );

        let carve = fleet_types::CarveMetadata {
            id: 0,
            created_at: now,
            host_id: host.id,
            name,
            block_count: payload.block_count,
            block_size: payload.block_size,
            carve_size: payload.carve_size,
            carve_id: payload.carve_id,
            request_id: payload.request_id,
            session_id,
            expired: false,
            error: None,
            max_block: -1,
        };

        let created = self.ds.new_carve(&carve).await?;
        info!(carve_id = created.id, host_id = host.id, "carve begun");
        Ok(created)
    }

    /// Receives a carve block (called by osquery agent).
    pub async fn carve_block(
        &self,
        payload: fleet_types::CarveBlockPayload,
    ) -> ServiceResult<()> {
        let carve = self.ds.carve_by_session_id(&payload.session_id).await?;

        if carve.request_id != payload.request_id {
            return Err(ServiceError::bad_request("request_id mismatch"));
        }
        if carve.expired {
            return Err(ServiceError::bad_request("carve has expired"));
        }
        if payload.block_id >= carve.block_count {
            return Err(ServiceError::bad_request("block_id exceeds block_count"));
        }

        self.ds
            .new_carve_block(carve.id, payload.block_id, &payload.data)
            .await?;

        Ok(())
    }
}
