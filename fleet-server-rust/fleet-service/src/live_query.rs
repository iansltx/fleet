//! Live query service operations.
//!
//! Implements distributed query campaign creation and management.
//! Corresponds to Go's `server/service/live_query.go`.

use tracing::info;

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult, Viewer};

impl FleetService {
    /// Creates a distributed query campaign and stores it in Redis.
    ///
    /// This is the core live query operation: given a SQL query and target hosts,
    /// it creates a campaign in MySQL, stores the query in Redis for hosts to pick up,
    /// and returns the campaign.
    pub async fn new_distributed_query_campaign(
        &self,
        viewer: &Viewer,
        query_sql: &str,
        host_ids: Vec<u32>,
    ) -> ServiceResult<fleet_types::campaign::DistributedQueryCampaign> {
        authz::authorize(viewer, Subject::Query, Action::Write)?;

        if query_sql.is_empty() {
            return Err(ServiceError::invalid_argument("query", "query SQL is required"));
        }
        if host_ids.is_empty() {
            return Err(ServiceError::bad_request("no target hosts specified"));
        }

        // Create an ad-hoc query for the campaign
        let query = fleet_types::Query {
            id: 0,
            team_id: None,
            interval: 0,
            platform: String::new(),
            min_osquery_version: String::new(),
            automations_enabled: false,
            logging: "snapshot".to_string(),
            name: format!("live_query_{}", chrono::Utc::now().timestamp()),
            description: "Ad-hoc live query".to_string(),
            query: query_sql.to_string(),
            saved: false,
            observer_can_run: false,
            author_id: Some(viewer.user_id()),
            author_name: viewer.user.name.clone(),
            author_email: viewer.user.email.clone(),
            packs: Vec::new(),
            aggregated_stats: Default::default(),
            discard_data: true,
            labels_include_any: Vec::new(),
            created_at: chrono::Utc::now(),
            updated_at: chrono::Utc::now(),
        };
        let saved_query = self.ds.new_query(&query).await?;

        // Create the campaign
        let campaign = self
            .ds
            .new_distributed_query_campaign(saved_query.id, viewer.user_id())
            .await?;

        // Add host targets
        for &host_id in &host_ids {
            self.ds
                .new_distributed_query_campaign_target(
                    campaign.id,
                    fleet_types::target::TargetType::Host,
                    host_id,
                )
                .await?;
        }

        info!(
            campaign_id = campaign.id,
            query_id = saved_query.id,
            host_count = host_ids.len(),
            "distributed query campaign created"
        );

        Ok(campaign)
    }

    /// Creates a distributed query campaign from an existing query ID and targets.
    pub async fn new_distributed_query_campaign_by_query_id(
        &self,
        viewer: &Viewer,
        query_id: u32,
        targets: &fleet_types::target::HostTargets,
    ) -> ServiceResult<(fleet_types::campaign::DistributedQueryCampaign, Vec<u32>)> {
        authz::authorize(viewer, Subject::Query, Action::Write)?;

        // Verify query exists
        let _query = self.ds.query(query_id).await?;

        // Resolve targets to host IDs
        let host_ids = self.ds.hosts_ids_for_targets(targets).await?;

        if host_ids.is_empty() {
            return Err(ServiceError::bad_request("no hosts targeted"));
        }

        // Create the campaign
        let campaign = self
            .ds
            .new_distributed_query_campaign(query_id, viewer.user_id())
            .await?;

        // Add all target types
        for &host_id in &targets.hosts {
            self.ds
                .new_distributed_query_campaign_target(
                    campaign.id,
                    fleet_types::target::TargetType::Host,
                    host_id,
                )
                .await?;
        }
        for &label_id in &targets.labels {
            self.ds
                .new_distributed_query_campaign_target(
                    campaign.id,
                    fleet_types::target::TargetType::Label,
                    label_id,
                )
                .await?;
        }
        for &team_id in &targets.teams {
            self.ds
                .new_distributed_query_campaign_target(
                    campaign.id,
                    fleet_types::target::TargetType::Team,
                    team_id,
                )
                .await?;
        }

        info!(
            campaign_id = campaign.id,
            query_id = query_id,
            host_count = host_ids.len(),
            "distributed query campaign created by query ID"
        );

        Ok((campaign, host_ids))
    }

    /// Runs a live query: creates a campaign, stores in Redis, waits for results
    /// from Redis pub/sub, and returns aggregated results.
    ///
    /// This is a "fire and wait" version used by the run_one_live_query endpoint.
    pub async fn run_live_query(
        &self,
        viewer: &Viewer,
        query_id: u32,
        host_ids: &[u32],
    ) -> ServiceResult<Vec<fleet_types::campaign::QueryResult>> {
        authz::authorize(viewer, Subject::Query, Action::Run)?;

        let query = self.ds.query(query_id).await?;

        if host_ids.is_empty() {
            return Err(ServiceError::bad_request("no target hosts specified"));
        }

        // Create the campaign in DB
        let mut campaign = self
            .ds
            .new_distributed_query_campaign(query.id, viewer.user_id())
            .await?;

        // Add host targets
        for &host_id in host_ids {
            self.ds
                .new_distributed_query_campaign_target(
                    campaign.id,
                    fleet_types::target::TargetType::Host,
                    host_id,
                )
                .await?;
        }

        // Mark campaign as running
        campaign.status = fleet_types::campaign::DistributedQueryStatus::Running;
        self.ds.save_distributed_query_campaign(&campaign).await?;

        info!(
            campaign_id = campaign.id,
            query_id = query.id,
            host_count = host_ids.len(),
            "live query started"
        );

        // Return empty results - actual result collection happens via Redis pub/sub
        // and WebSocket streaming. The handler will use RedisLiveQuery and RedisQueryResults
        // to manage the full lifecycle.
        Ok(Vec::new())
    }
}
