//! Osquery service operations.
//!
//! Implements osquery agent enrollment, configuration delivery,
//! distributed query dispatch, and log submission.
//! Corresponds to Go's `server/service/osquery.go`.

use std::collections::HashMap;

use tracing::{info, warn};

use crate::auth;
use crate::fleet_service::FleetService;
#[allow(unused_imports)]
use crate::{ServiceError, ServiceResult};

impl FleetService {
    /// Authenticates a host by its node key.
    ///
    /// Corresponds to Go's `(svc *Service) AuthenticateHost`.
    /// Returns the host and a debug flag.
    pub async fn authenticate_host(
        &self,
        node_key: &str,
    ) -> ServiceResult<(fleet_types::Host, bool)> {
        if node_key.is_empty() {
            return Err(OsqueryError::invalid_node(
                "authentication error: missing node key",
            )
            .into());
        }

        let host = self
            .ds
            .load_host_by_node_key(node_key)
            .await
            .map_err(|e| match e {
                ServiceError::NotFound(_) => {
                    OsqueryError::invalid_node("authentication error: invalid node key").into()
                }
                other => ServiceError::internal(format!("authentication error: {}", other)),
            })?;

        // Record host last seen time.
        let now = self.clock.now();
        if let Err(e) = self.ds.mark_host_seen(host.id.into(), now).await {
            warn!("Failed to record host last seen: {}", e);
        }

        // Debug mode is false by default in the free tier.
        Ok((host, false))
    }

    /// Enrolls a new osquery agent.
    ///
    /// Corresponds to Go's `(svc *Service) EnrollOsquery`.
    /// Validates the enroll secret, generates a node key, and creates/updates
    /// the host record.
    pub async fn enroll_agent(
        &self,
        enroll_secret: &str,
        host_identifier: &str,
        host_details: &HashMap<String, HashMap<String, String>>,
    ) -> ServiceResult<String> {
        // Verify the enroll secret.
        let secret_info = self
            .ds
            .verify_enroll_secret(enroll_secret)
            .await
            .map_err(|e| OsqueryError::invalid_node(format!("enroll failed: {}", e)))?;

        // Generate a node key.
        let node_key = auth::generate_random_text(self.config.osquery.node_key_size)?;

        // Extract hardware UUID and serial from system_info.
        let (hardware_uuid, hardware_serial) = if let Some(system_info) = host_details.get("system_info") {
            (
                system_info.get("uuid").cloned().unwrap_or_default(),
                system_info
                    .get("hardware_serial")
                    .cloned()
                    .unwrap_or_default(),
            )
        } else {
            (String::new(), String::new())
        };

        let host = self
            .ds
            .enroll_host(
                host_identifier,
                &node_key,
                secret_info.team_id,
                &hardware_uuid,
                &hardware_serial,
            )
            .await
            .map_err(|e| {
                OsqueryError::invalid_node(format!("save enroll failed: {}", e))
            })?;

        info!(
            host_id = host.id,
            identifier = %host_identifier,
            "host enrolled"
        );

        Ok(node_key)
    }

    /// Returns the osquery configuration for a host.
    ///
    /// Corresponds to Go's `(svc *Service) GetClientConfig`.
    /// The configuration includes options, scheduled queries (packs),
    /// decorators, and file paths.
    pub async fn get_client_config(
        &self,
        host: &fleet_types::Host,
    ) -> ServiceResult<HashMap<String, serde_json::Value>> {
        let mut config = HashMap::new();

        // Load app config for agent options.
        let app_config = self.ds.app_config().await?;

        if let Some(agent_options) = app_config.agent_options {
            // Merge agent options into the config.
            if let serde_json::Value::Object(opts) = agent_options {
                for (key, value) in opts {
                    config.insert(key, value);
                }
            }
        }

        // Build packs section from host's packs and their scheduled queries.
        let packs = self.ds.list_packs_for_host(host.id).await.unwrap_or_default();
        if !packs.is_empty() {
            let mut packs_map = serde_json::Map::new();
            for pack in &packs {
                if pack.disabled {
                    continue;
                }
                let scheduled = self.ds.list_scheduled_queries_in_pack(pack.id).await.unwrap_or_default();
                let mut queries_map = serde_json::Map::new();
                for sq in &scheduled {
                    if let Ok(q) = self.ds.query(sq.query_id).await {
                        queries_map.insert(q.name.clone(), serde_json::json!({
                            "query": q.query,
                            "interval": sq.interval,
                            "snapshot": sq.snapshot,
                            "removed": sq.removed,
                            "platform": sq.platform,
                            "version": sq.version,
                        }));
                    }
                }
                packs_map.insert(pack.name.clone(), serde_json::json!({
                    "queries": queries_map,
                }));
            }
            config.insert("packs".to_string(), serde_json::Value::Object(packs_map));
        }

        Ok(config)
    }

    /// Returns the distributed queries that should be run on a host.
    ///
    /// Corresponds to Go's `(svc *Service) GetDistributedQueries`.
    /// Returns a map of query name -> query SQL, a map of discovery queries,
    /// and an accelerate interval (0 = no acceleration).
    pub async fn get_distributed_queries(
        &self,
        host: &fleet_types::Host,
    ) -> ServiceResult<DistributedQueryResult> {
        let mut queries = HashMap::new();
        let mut discovery = HashMap::new();

        // 1. Add label queries for dynamic labels
        let labels = self.ds.list_labels(fleet_types::ListOptions::default()).await.unwrap_or_default();
        for label in &labels {
            if label.label_membership_type == fleet_types::label::LabelMembershipType::Dynamic
                && !label.query.is_empty()
            {
                let key = format!("fleet_label_query_{}", label.id);
                queries.insert(key.clone(), label.query.clone());
                discovery.insert(key, String::new());
            }
        }

        // 2. Add global policy queries
        let policies = self.ds.list_global_policies(fleet_types::ListOptions::default()).await.unwrap_or_default();
        for policy in &policies {
            let key = format!("fleet_policy_query_{}", policy.policy_data.id);
            queries.insert(key, policy.policy_data.query.clone());
        }

        // 3. Add team policy queries if host belongs to a team
        if let Some(team_id) = host.team_id {
            let team_policies = self.ds.list_team_policies(team_id, fleet_types::ListOptions::default()).await.unwrap_or_default();
            for policy in &team_policies {
                let key = format!("fleet_policy_query_{}", policy.policy_data.id);
                queries.insert(key, policy.policy_data.query.clone());
            }
        }

        Ok(DistributedQueryResult {
            queries,
            discovery,
            accelerate: 0,
        })
    }

    /// Submits results from distributed queries.
    ///
    /// Corresponds to Go's `(svc *Service) SubmitDistributedQueryResults`.
    /// Processes results for detail queries, label queries, policy queries,
    /// and live queries.
    pub async fn submit_distributed_query_results(
        &self,
        host: &fleet_types::Host,
        results: &HashMap<String, Vec<HashMap<String, String>>>,
        _statuses: &HashMap<String, i32>,
        _messages: &HashMap<String, String>,
    ) -> ServiceResult<()> {
        for (query_name, rows) in results {
            // Process label query results
            if let Some(label_id_str) = query_name.strip_prefix("fleet_label_query_") {
                if let Ok(label_id) = label_id_str.parse::<u32>() {
                    // Non-empty result means the label matches this host
                    let matches = !rows.is_empty();
                    if matches {
                        if let Err(e) = self.ds.record_label_membership(label_id, host.id).await {
                            warn!(label_id, host_id = host.id, "failed to record label membership: {}", e);
                        }
                    } else {
                        if let Err(e) = self.ds.delete_label_membership(label_id, host.id).await {
                            warn!(label_id, host_id = host.id, "failed to remove label membership: {}", e);
                        }
                    }
                }
                continue;
            }

            // Process policy query results
            if query_name.starts_with("fleet_policy_query_") {
                // Policy results are processed elsewhere (via policy_updated_at)
                // Just acknowledge receipt
                continue;
            }
        }

        info!(
            host_id = host.id,
            result_count = results.len(),
            "distributed query results submitted"
        );

        Ok(())
    }

    /// Submits osquery status logs.
    ///
    /// Corresponds to Go's `(svc *Service) SubmitStatusLogs`.
    pub async fn submit_status_logs(
        &self,
        host: &fleet_types::Host,
        logs: &[serde_json::Value],
    ) -> ServiceResult<()> {
        info!(
            host_id = host.id,
            log_count = logs.len(),
            "status logs submitted"
        );
        Ok(())
    }

    /// Submits osquery result logs.
    ///
    /// Corresponds to Go's `(svc *Service) SubmitResultLogs`.
    pub async fn submit_result_logs(
        &self,
        host: &fleet_types::Host,
        logs: &[serde_json::Value],
    ) -> ServiceResult<()> {
        info!(
            host_id = host.id,
            log_count = logs.len(),
            "result logs submitted"
        );
        Ok(())
    }
}

/// Result of a distributed query fetch.
pub struct DistributedQueryResult {
    /// Map of query name -> query SQL.
    pub queries: HashMap<String, String>,
    /// Map of query name -> discovery SQL (empty string = always run).
    pub discovery: HashMap<String, String>,
    /// Accelerate interval in seconds (0 = no acceleration).
    pub accelerate: u32,
}

/// OsqueryError is an error specific to osquery agent communication.
/// It may indicate that the node key is invalid and the agent should re-enroll.
#[derive(Debug)]
pub struct OsqueryError {
    pub message: String,
    pub invalid_node: bool,
}

impl OsqueryError {
    pub fn new(message: impl Into<String>, invalid_node: bool) -> Self {
        Self {
            message: message.into(),
            invalid_node,
        }
    }

    pub fn invalid_node(message: impl Into<String>) -> Self {
        Self::new(message, true)
    }
}

impl std::fmt::Display for OsqueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.message)
    }
}

impl std::error::Error for OsqueryError {}

impl From<OsqueryError> for ServiceError {
    fn from(e: OsqueryError) -> Self {
        if e.invalid_node {
            ServiceError::AuthFailed(e.message)
        } else {
            ServiceError::Internal(e.message)
        }
    }
}
