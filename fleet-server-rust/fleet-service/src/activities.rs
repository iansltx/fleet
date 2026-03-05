//! Activity logging service operations.
//!
//! Implements activity listing and management.
//! Corresponds to Go's activity-related service operations.

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceResult, Viewer};

impl FleetService {
    /// Lists activities with pagination.
    pub async fn list_activities(
        &self,
        viewer: &Viewer,
        page: u32,
        per_page: u32,
    ) -> ServiceResult<Vec<fleet_types::Activity>> {
        authz::authorize(viewer, Subject::Activity, Action::Read)?;
        let offset = page.saturating_mul(per_page);
        let limit = if per_page == 0 { 20 } else { per_page };
        self.ds.list_activities(limit, offset).await
    }

    /// Counts upcoming activities for a host.
    pub async fn count_host_upcoming_activities(
        &self,
        viewer: &Viewer,
        host_id: u32,
    ) -> ServiceResult<u32> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;
        self.ds.count_host_upcoming_activities(host_id).await
    }

    /// Lists upcoming activities for a host.
    pub async fn list_host_upcoming_activities(
        &self,
        viewer: &Viewer,
        host_id: u32,
    ) -> ServiceResult<Vec<fleet_types::UpcomingActivity>> {
        authz::authorize(viewer, Subject::Host, Action::Read)?;
        self.ds.list_host_upcoming_activities(host_id).await
    }

    /// Cancels (deletes) a specific upcoming activity for a host.
    pub async fn cancel_host_upcoming_activity(
        &self,
        viewer: &Viewer,
        host_id: u32,
        activity_id: u32,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Host, Action::Write)?;
        self.ds.delete_host_upcoming_activity(host_id, activity_id).await
    }
}
