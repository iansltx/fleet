//! Query service operations.
//!
//! Implements query CRUD and report retrieval.
//! Corresponds to Go's `server/service/queries.go`.

use tracing::info;

use crate::authz::{self, Action, Subject};
use crate::fleet_service::FleetService;
use crate::{ServiceError, ServiceResult, Viewer};

impl FleetService {
    /// Gets a single query by ID.
    ///
    /// Corresponds to Go's `(svc *Service) GetQuery`.
    pub async fn get_query(
        &self,
        viewer: &Viewer,
        id: u32,
    ) -> ServiceResult<fleet_types::Query> {
        let query = self.ds.query(id).await?;
        authz::authorize(viewer, Subject::Query, Action::Read)?;
        Ok(query)
    }

    /// Lists queries, optionally filtered by team.
    ///
    /// Corresponds to Go's `(svc *Service) ListQueries`.
    pub async fn list_queries(
        &self,
        viewer: &Viewer,
        opts: fleet_types::ListOptions,
        team_id: Option<u32>,
    ) -> ServiceResult<Vec<fleet_types::Query>> {
        authz::authorize(viewer, Subject::Query, Action::Read)?;
        self.ds.list_queries(opts, team_id).await
    }

    /// Creates a new query.
    ///
    /// Corresponds to Go's `(svc *Service) NewQuery`.
    pub async fn new_query(
        &self,
        viewer: &Viewer,
        payload: QueryPayload,
    ) -> ServiceResult<fleet_types::Query> {
        authz::authorize(viewer, Subject::Query, Action::Write)?;

        if payload.name.is_empty() {
            return Err(ServiceError::invalid_argument("name", "missing required argument"));
        }
        if payload.query.is_empty() {
            return Err(ServiceError::invalid_argument("query", "missing required argument"));
        }

        let mut query = default_query();
        query.name = payload.name;
        query.description = payload.description.unwrap_or_default();
        query.query = payload.query;
        query.saved = true;
        query.author_id = Some(viewer.user_id());
        query.author_name = viewer.user.name.clone();
        query.author_email = viewer.user.email.clone();
        query.observer_can_run = payload.observer_can_run.unwrap_or(false);
        query.team_id = payload.team_id;
        query.interval = payload.interval.unwrap_or(0);
        query.platform = payload.platform.unwrap_or_default();
        query.min_osquery_version = payload.min_osquery_version.unwrap_or_default();
        query.automations_enabled = payload.automations_enabled.unwrap_or(false);
        query.logging = payload.logging.unwrap_or_else(|| "snapshot".to_string());
        query.discard_data = payload.discard_data.unwrap_or(false);

        let created = self.ds.new_query(&query).await?;

        info!(query_id = created.id, name = %created.name, "query created");
        Ok(created)
    }

    /// Modifies an existing query.
    ///
    /// Corresponds to Go's `(svc *Service) ModifyQuery`.
    pub async fn modify_query(
        &self,
        viewer: &Viewer,
        id: u32,
        payload: ModifyQueryPayload,
    ) -> ServiceResult<fleet_types::Query> {
        authz::authorize(viewer, Subject::Query, Action::Write)?;

        let mut query = self.ds.query(id).await?;

        if let Some(name) = payload.name {
            query.name = name;
        }
        if let Some(description) = payload.description {
            query.description = description;
        }
        if let Some(sql) = payload.query {
            query.query = sql;
        }
        if let Some(observer_can_run) = payload.observer_can_run {
            query.observer_can_run = observer_can_run;
        }
        if let Some(interval) = payload.interval {
            query.interval = interval;
        }
        if let Some(platform) = payload.platform {
            query.platform = platform;
        }
        if let Some(min_osquery_version) = payload.min_osquery_version {
            query.min_osquery_version = min_osquery_version;
        }
        if let Some(automations_enabled) = payload.automations_enabled {
            query.automations_enabled = automations_enabled;
        }
        if let Some(logging) = payload.logging {
            query.logging = logging;
        }
        if let Some(discard_data) = payload.discard_data {
            query.discard_data = discard_data;
        }

        let saved = self.ds.save_query(&query).await?;
        info!(query_id = saved.id, name = %saved.name, "query modified");
        Ok(saved)
    }

    /// Deletes a query by name.
    ///
    /// Corresponds to Go's `(svc *Service) DeleteQuery`.
    pub async fn delete_query(
        &self,
        viewer: &Viewer,
        name: &str,
        team_id: Option<u32>,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Query, Action::Write)?;

        self.ds.delete_query(name, team_id).await?;

        info!(name = %name, "query deleted");
        Ok(())
    }

    /// Deletes multiple queries by IDs.
    ///
    /// Corresponds to Go's `(svc *Service) DeleteQueries`.
    pub async fn delete_queries(
        &self,
        viewer: &Viewer,
        ids: &[u32],
    ) -> ServiceResult<u32> {
        authz::authorize(viewer, Subject::Query, Action::Write)?;

        let count = self.ds.delete_queries(ids).await?;
        info!(count = count, "queries deleted");
        Ok(count)
    }

    /// Gets all query specs, optionally filtered by team.
    ///
    /// Corresponds to Go's `(svc *Service) GetQuerySpecs`.
    pub async fn get_query_specs(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
    ) -> ServiceResult<Vec<QuerySpec>> {
        authz::authorize(viewer, Subject::Query, Action::Read)?;
        let opts = fleet_types::ListOptions::default();
        let queries = self.ds.list_queries(opts, team_id).await?;
        Ok(queries.into_iter().map(query_to_spec).collect())
    }

    /// Gets a single query spec by name and optional team.
    ///
    /// Corresponds to Go's `(svc *Service) GetQuerySpec`.
    pub async fn get_query_spec(
        &self,
        viewer: &Viewer,
        team_id: Option<u32>,
        name: &str,
    ) -> ServiceResult<QuerySpec> {
        authz::authorize(viewer, Subject::Query, Action::Read)?;
        let query = self.ds.query_by_name(team_id, name).await?;
        Ok(query_to_spec(query))
    }

    /// Applies query specs (create or update). Matches Go's `ApplyQuerySpecs`.
    pub async fn apply_query_specs(
        &self,
        viewer: &Viewer,
        specs: Vec<QuerySpec>,
    ) -> ServiceResult<()> {
        authz::authorize(viewer, Subject::Query, Action::Write)?;

        let spec_count = specs.len();
        for spec in specs {
            if spec.name.is_empty() {
                return Err(ServiceError::invalid_argument("name", "missing required argument"));
            }
            if spec.query.is_empty() {
                return Err(ServiceError::invalid_argument("query", "missing required argument"));
            }

            // Try to find existing query by name + team
            match self.ds.query_by_name(spec.team_id, &spec.name).await {
                Ok(mut existing) => {
                    // Update existing query
                    existing.description = spec.description.unwrap_or(existing.description);
                    existing.query = spec.query;
                    existing.interval = spec.interval.unwrap_or(existing.interval);
                    existing.platform = spec.platform.unwrap_or(existing.platform);
                    existing.min_osquery_version = spec.min_osquery_version.unwrap_or(existing.min_osquery_version);
                    existing.automations_enabled = spec.automations_enabled.unwrap_or(existing.automations_enabled);
                    existing.logging = spec.logging.unwrap_or(existing.logging);
                    existing.observer_can_run = spec.observer_can_run.unwrap_or(existing.observer_can_run);
                    existing.discard_data = spec.discard_data.unwrap_or(existing.discard_data);
                    self.ds.save_query(&existing).await?;
                }
                Err(ServiceError::NotFound(_)) => {
                    // Create new query
                    let mut query = default_query();
                    query.name = spec.name;
                    query.description = spec.description.unwrap_or_default();
                    query.query = spec.query;
                    query.saved = true;
                    query.author_id = Some(viewer.user_id());
                    query.author_name = viewer.user.name.clone();
                    query.author_email = viewer.user.email.clone();
                    query.team_id = spec.team_id;
                    query.interval = spec.interval.unwrap_or(0);
                    query.platform = spec.platform.unwrap_or_default();
                    query.min_osquery_version = spec.min_osquery_version.unwrap_or_default();
                    query.automations_enabled = spec.automations_enabled.unwrap_or(false);
                    query.logging = spec.logging.unwrap_or_else(|| "snapshot".to_string());
                    query.observer_can_run = spec.observer_can_run.unwrap_or(false);
                    query.discard_data = spec.discard_data.unwrap_or(false);
                    self.ds.new_query(&query).await?;
                }
                Err(e) => return Err(e),
            }
        }

        info!(count = spec_count, "query specs applied");
        Ok(())
    }

    /// Returns the query report (result rows) for a given query.
    ///
    /// Corresponds to Go's `(svc *Service) GetQueryReportResults`.
    pub async fn get_query_report(
        &self,
        viewer: &Viewer,
        query_id: u32,
    ) -> ServiceResult<QueryReport> {
        authz::authorize(viewer, Subject::Query, Action::Read)?;

        let query = self.ds.query(query_id).await?;

        if query.discard_data {
            return Ok(QueryReport {
                query_id: query.id,
                results: Vec::new(),
                report_clipped: false,
            });
        }

        let rows = self.ds.query_result_rows(query_id).await?;
        let report_clipped = rows.len() > 1000;
        let results: Vec<fleet_types::QueryResultRow> = if report_clipped {
            rows.into_iter().take(1000).collect()
        } else {
            rows
        };

        Ok(QueryReport {
            query_id: query.id,
            results,
            report_clipped,
        })
    }
}

/// Query report containing result rows.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryReport {
    pub query_id: u32,
    pub results: Vec<fleet_types::QueryResultRow>,
    pub report_clipped: bool,
}

/// Payload for creating a new query.
#[derive(Debug, Clone, Default)]
pub struct QueryPayload {
    pub name: String,
    pub description: Option<String>,
    pub query: String,
    pub observer_can_run: Option<bool>,
    pub team_id: Option<u32>,
    pub interval: Option<u32>,
    pub platform: Option<String>,
    pub min_osquery_version: Option<String>,
    pub automations_enabled: Option<bool>,
    pub logging: Option<String>,
    pub discard_data: Option<bool>,
}

/// Payload for modifying an existing query.
#[derive(Debug, Clone, Default)]
pub struct ModifyQueryPayload {
    pub name: Option<String>,
    pub description: Option<String>,
    pub query: Option<String>,
    pub observer_can_run: Option<bool>,
    pub interval: Option<u32>,
    pub platform: Option<String>,
    pub min_osquery_version: Option<String>,
    pub automations_enabled: Option<bool>,
    pub logging: Option<String>,
    pub discard_data: Option<bool>,
}

/// Query spec for apply/get operations.
/// Corresponds to Go's `fleet.QuerySpec`.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct QuerySpec {
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    pub query: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observer_can_run: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub min_osquery_version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub automations_enabled: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logging: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub discard_data: Option<bool>,
}

/// Converts a Query to a QuerySpec.
fn query_to_spec(q: fleet_types::Query) -> QuerySpec {
    QuerySpec {
        name: q.name,
        description: Some(q.description),
        query: q.query,
        team_id: q.team_id,
        interval: Some(q.interval),
        observer_can_run: Some(q.observer_can_run),
        platform: Some(q.platform),
        min_osquery_version: Some(q.min_osquery_version),
        automations_enabled: Some(q.automations_enabled),
        logging: Some(q.logging),
        discard_data: Some(q.discard_data),
    }
}

/// Helper to create a new Query with default values.
fn default_query() -> fleet_types::Query {
    let now = chrono::Utc::now();
    fleet_types::Query {
        id: 0,
        team_id: None,
        interval: 0,
        platform: String::new(),
        min_osquery_version: String::new(),
        automations_enabled: false,
        logging: "snapshot".to_string(),
        name: String::new(),
        description: String::new(),
        query: String::new(),
        saved: false,
        observer_can_run: false,
        author_id: None,
        author_name: String::new(),
        author_email: String::new(),
        packs: Vec::new(),
        aggregated_stats: Default::default(),
        discard_data: false,
        labels_include_any: Vec::new(),
        created_at: now,
        updated_at: now,
    }
}
