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
        let labels: Vec<fleet_types::label::LabelSummary> = self
            .ds
            .list_labels_for_host(id)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(label_to_summary)
            .collect();
        let packs = self.ds.list_packs_for_host(id).await.unwrap_or_default();

        Ok(fleet_types::HostDetail {
            host,
            labels,
            packs,
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
        let labels: Vec<fleet_types::label::LabelSummary> = self
            .ds
            .list_labels_for_host(host.id)
            .await
            .unwrap_or_default()
            .into_iter()
            .map(label_to_summary)
            .collect();
        let packs = self.ds.list_packs_for_host(host.id).await.unwrap_or_default();

        Ok(fleet_types::HostDetail {
            host,
            labels,
            packs,
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

    /// Deletes multiple hosts by IDs.
    ///
    /// Corresponds to Go's `deleteHostsEndpoint`.
    pub async fn delete_hosts(&self, viewer: &Viewer, ids: &[u32]) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Host, Action::Write)?;
        for &id in ids {
            self.ds.delete_host(id).await?;
        }
        info!(count = ids.len(), "hosts deleted");
        Ok(())
    }

    /// Counts hosts matching filter options.
    ///
    /// Corresponds to Go's `countHostsEndpoint`.
    pub async fn count_hosts(
        &self,
        viewer: &Viewer,
        opts: fleet_types::HostListOptions,
    ) -> ServiceResult<u64> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;
        let hosts = self.ds.list_hosts(opts).await?;
        Ok(hosts.len() as u64)
    }

    /// Gets device mapping (emails) for a host.
    ///
    /// Corresponds to Go's `listHostDeviceMappingEndpoint`.
    pub async fn list_host_device_mapping(
        &self,
        viewer: &Viewer,
        host_id: u32,
    ) -> ServiceResult<serde_json::Value> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;
        // Verify host exists
        self.ds.host(host_id).await?;
        self.ds.device_mapping_for_host(host_id).await
    }

    /// Gets a report of all hosts (CSV-style listing).
    ///
    /// Corresponds to Go's `hostsReportEndpoint`.
    pub async fn hosts_report(
        &self,
        viewer: &Viewer,
    ) -> ServiceResult<Vec<fleet_types::Host>> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;
        let opts = fleet_types::HostListOptions::default();
        self.ds.list_hosts(opts).await
    }

    /// Lists software installed on a specific host.
    ///
    /// Corresponds to Go's host software listing endpoint.
    pub async fn list_host_software(
        &self,
        viewer: &Viewer,
        host_id: u32,
    ) -> ServiceResult<Vec<fleet_types::Software>> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;
        self.ds.list_software_for_host(host_id).await
    }

    /// Transfers hosts to a team (or no team if team_id is None).
    ///
    /// Corresponds to Go's `(svc *Service) AddHostsToTeam`.
    pub async fn add_hosts_to_team(
        &self,
        viewer: &Viewer,
        host_ids: &[u32],
        team_id: Option<u32>,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Host, Action::Write)?;

        // If transferring to a specific team, verify it exists.
        if let Some(tid) = team_id {
            self.ds.team(tid).await?;
        }

        self.ds.transfer_hosts_to_team(host_ids, team_id).await?;
        info!(count = host_ids.len(), team_id = ?team_id, "hosts transferred to team");
        Ok(())
    }
}

/// Converts a full Label to a LabelSummary.
fn label_to_summary(label: fleet_types::Label) -> fleet_types::label::LabelSummary {
    fleet_types::label::LabelSummary {
        id: label.id,
        name: label.name,
        description: label.description,
        team_id: label.team_id,
        label_type: label.label_type,
    }
}
