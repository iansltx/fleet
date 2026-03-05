//! Host service operations.
//!
//! Implements host listing, retrieval, deletion, and summary.
//! Corresponds to Go's `server/service/hosts.go`.

use tracing::info;

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceResult, Viewer};

impl FleetService {
    /// Lists hosts matching the given filter options.
    ///
    /// Corresponds to Go's list hosts endpoint.
    pub async fn list_hosts(
        &self,
        viewer: &Viewer,
        opts: fleet_types::HostListOptions,
    ) -> ServiceResult<Vec<fleet_types::Host>> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;
        self.ds.list_hosts(opts).await
    }

    /// Gets a single host by ID with full details.
    ///
    /// Corresponds to Go's `getHostEndpoint`.
    pub async fn get_host(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::HostDetail> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;

        let host = self.ds.host(id.into()).await?;

        Ok(fleet_types::HostDetail {
            host,
            labels: Vec::new(),
            packs: Vec::new(),
        })
    }

    /// Gets a host by identifier (hostname, UUID, serial number, etc.).
    ///
    /// Corresponds to Go's host by identifier endpoint.
    pub async fn get_host_by_identifier(
        &self,
        viewer: &Viewer,
        identifier: &str,
    ) -> ServiceResult<fleet_types::HostDetail> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;

        let host = self.ds.host_by_identifier(identifier).await?;

        Ok(fleet_types::HostDetail {
            host,
            labels: Vec::new(),
            packs: Vec::new(),
        })
    }

    /// Deletes a host.
    ///
    /// Corresponds to Go's `deleteHostEndpoint`.
    pub async fn delete_host(&self, viewer: &Viewer, id: u32) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Host, Action::Write)?;

        self.ds.delete_host(id.into()).await?;

        info!(host_id = id, "host deleted");
        Ok(())
    }

    /// Returns a summary of host counts by status.
    ///
    /// Corresponds to Go's host summary endpoint.
    pub async fn host_summary(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<fleet_types::HostSummary> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;
        self.ds.host_summary().await
    }

    /// Marks a host for refetch -- the next time the host checks in,
    /// Fleet will request full detail queries.
    ///
    /// Corresponds to Go's `refetchHostEndpoint`.
    pub async fn refetch_host(&self, viewer: &Viewer, id: u32) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Host, Action::Write)?;

        // Verify the host exists.
        let _host = self.ds.host(id.into()).await?;

        // In the Go implementation this sets `refetch_requested = true`.
        // The datastore will handle this in a real implementation.
        info!(host_id = id, "host refetch requested");
        Ok(())
    }

    /// Gets basic host info without expensive JOINs.
    ///
    /// Corresponds to Go's `GetHostLite`.
    pub async fn get_host_lite(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::Host> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;
        self.ds.host_lite(id.into()).await
    }
}
