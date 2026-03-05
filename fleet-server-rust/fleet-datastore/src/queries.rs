//! Query CRUD operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/queries.go`.

use chrono::{DateTime, Utc};

use crate::error::{is_duplicate, DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for query results matching the queries table + author JOIN.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct QueryRow {
    pub id: u32,
    pub team_id: Option<u32>,
    pub name: String,
    pub description: String,
    pub query: String,
    pub author_id: Option<u32>,
    pub saved: bool,
    pub observer_can_run: bool,
    pub schedule_interval: u32,
    pub platform: String,
    pub min_osquery_version: String,
    pub automations_enabled: bool,
    pub logging_type: String,
    pub discard_data: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    // These are populated via LEFT JOIN to users and aggregated_stats
    #[sqlx(default)]
    pub author_name: String,
    #[sqlx(default)]
    pub author_email: String,
    #[sqlx(default)]
    pub user_time_p50: Option<f64>,
    #[sqlx(default)]
    pub user_time_p95: Option<f64>,
    #[sqlx(default)]
    pub system_time_p50: Option<f64>,
    #[sqlx(default)]
    pub system_time_p95: Option<f64>,
    #[sqlx(default)]
    pub total_executions: Option<f64>,
}

/// Row type for query result rows from the query_result_rows table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct QueryResultRow {
    pub host_id: u32,
    pub last_fetched: DateTime<Utc>,
    pub data: Option<String>,
    pub hostname: String,
}

/// Parameters for creating a new query.
pub struct NewQueryParams {
    pub name: String,
    pub description: String,
    pub query: String,
    pub saved: bool,
    pub author_id: Option<u32>,
    pub observer_can_run: bool,
    pub team_id: Option<u32>,
    pub team_id_char: String,
    pub platform: String,
    pub min_osquery_version: String,
    pub schedule_interval: u32,
    pub automations_enabled: bool,
    pub logging_type: String,
    pub discard_data: bool,
}

impl MysqlDatastore {
    /// Creates a new query. Matches Go's `NewQuery`.
    ///
    /// INSERT INTO queries (name, description, query, saved, author_id,
    ///   observer_can_run, team_id, team_id_char, platform, min_osquery_version,
    ///   schedule_interval, automations_enabled, logging_type, discard_data,
    ///   created_at, updated_at)
    /// VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    pub async fn new_query(&self, params: NewQueryParams) -> Result<u32> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            INSERT INTO queries (
                name, description, query, saved, author_id, observer_can_run,
                team_id, team_id_char, platform, min_osquery_version,
                schedule_interval, automations_enabled, logging_type,
                discard_data, created_at, updated_at
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&params.name)
        .bind(&params.description)
        .bind(&params.query)
        .bind(params.saved)
        .bind(params.author_id)
        .bind(params.observer_can_run)
        .bind(params.team_id)
        .bind(&params.team_id_char)
        .bind(&params.platform)
        .bind(&params.min_osquery_version)
        .bind(params.schedule_interval)
        .bind(params.automations_enabled)
        .bind(&params.logging_type)
        .bind(params.discard_data)
        .bind(now)
        .bind(now)
        .execute(self.pool())
        .await;

        match result {
            Ok(res) => Ok(res.last_insert_id() as u32),
            Err(e) => {
                if is_duplicate(&e) {
                    Err(DatastoreError::already_exists("Query", &params.name))
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Gets a query by ID with author info and stats. Matches Go's `Query`.
    ///
    /// Uses the same complex SELECT with LEFT JOINs to users and aggregated_stats.
    pub async fn query_by_id(&self, id: u32) -> Result<QueryRow> {
        sqlx::query_as::<_, QueryRow>(
            r#"
            SELECT
                q.id, q.team_id, q.name, q.description, q.query, q.author_id,
                q.saved, q.observer_can_run, q.schedule_interval, q.platform,
                q.min_osquery_version, q.automations_enabled, q.logging_type,
                q.discard_data, q.created_at, q.updated_at, q.discard_data,
                COALESCE(NULLIF(u.name, ''), u.email, '') AS author_name,
                COALESCE(u.email, '') AS author_email,
                JSON_EXTRACT(json_value, '$.user_time_p50') as user_time_p50,
                JSON_EXTRACT(json_value, '$.user_time_p95') as user_time_p95,
                JSON_EXTRACT(json_value, '$.system_time_p50') as system_time_p50,
                JSON_EXTRACT(json_value, '$.system_time_p95') as system_time_p95,
                JSON_EXTRACT(json_value, '$.total_executions') as total_executions
            FROM queries q
            LEFT JOIN users u ON q.author_id = u.id
            LEFT JOIN aggregated_stats ag ON (ag.id = q.id AND ag.global_stats = ? AND ag.type = ?)
            WHERE q.id = ?
            "#,
        )
        .bind(false)
        .bind("scheduled_query")
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_id("Query", id as u64))
    }

    /// Gets a query by name and team. Matches Go's `QueryByName`.
    pub async fn query_by_name(&self, team_id: Option<u32>, name: &str) -> Result<QueryRow> {
        let (where_clause, team_id_char) = if let Some(tid) = team_id {
            (" AND team_id_char = ?", tid.to_string())
        } else {
            (" AND team_id_char = ''", String::new())
        };

        let sql = format!(
            r#"
            SELECT id, team_id, name, description, query, author_id, saved,
                observer_can_run, schedule_interval, platform, min_osquery_version,
                automations_enabled, logging_type, discard_data, created_at, updated_at,
                '' as author_name, '' as author_email,
                NULL as user_time_p50, NULL as user_time_p95,
                NULL as system_time_p50, NULL as system_time_p95,
                NULL as total_executions
            FROM queries
            WHERE name = ? {}
            "#,
            where_clause
        );

        let mut query_builder = sqlx::query_as::<_, QueryRow>(&sql).bind(name);
        if team_id.is_some() {
            query_builder = query_builder.bind(&team_id_char);
        }

        query_builder
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_name("Query", name))
    }

    /// Updates an existing query. Matches Go's `SaveQuery`.
    ///
    /// UPDATE queries SET name=?, description=?, query=?, author_id=?, saved=?,
    ///   observer_can_run=?, team_id=?, team_id_char=?, platform=?,
    ///   min_osquery_version=?, schedule_interval=?, automations_enabled=?,
    ///   logging_type=?, discard_data=?
    /// WHERE id = ?
    pub async fn save_query(
        &self,
        id: u32,
        name: &str,
        description: &str,
        query: &str,
        author_id: Option<u32>,
        saved: bool,
        observer_can_run: bool,
        team_id: Option<u32>,
        team_id_char: &str,
        platform: &str,
        min_osquery_version: &str,
        schedule_interval: u32,
        automations_enabled: bool,
        logging_type: &str,
        discard_data: bool,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE queries SET
                name = ?, description = ?, query = ?, author_id = ?, saved = ?,
                observer_can_run = ?, team_id = ?, team_id_char = ?, platform = ?,
                min_osquery_version = ?, schedule_interval = ?, automations_enabled = ?,
                logging_type = ?, discard_data = ?
            WHERE id = ?
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(query)
        .bind(author_id)
        .bind(saved)
        .bind(observer_can_run)
        .bind(team_id)
        .bind(team_id_char)
        .bind(platform)
        .bind(min_osquery_version)
        .bind(schedule_interval)
        .bind(automations_enabled)
        .bind(logging_type)
        .bind(discard_data)
        .bind(id)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Query", id as u64));
        }
        Ok(())
    }

    /// Deletes a query by team and name. Matches Go's `DeleteQuery`.
    pub async fn delete_query(&self, team_id: Option<u32>, name: &str) -> Result<()> {
        let (where_clause, team_id_char) = if let Some(tid) = team_id {
            (" AND team_id_char = ?", tid.to_string())
        } else {
            (" AND team_id_char = ''", String::new())
        };

        let select_sql = format!("SELECT id FROM queries WHERE name = ? {}", where_clause);

        let mut select_query = sqlx::query_as::<_, (u32,)>(&select_sql).bind(name);
        if team_id.is_some() {
            select_query = select_query.bind(&team_id_char);
        }

        let (query_id,) = select_query
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_name("Query", name))?;

        let result = sqlx::query("DELETE FROM queries WHERE id = ?")
            .bind(query_id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() != 1 {
            return Err(DatastoreError::not_found_with_name("Query", name));
        }
        Ok(())
    }

    /// Deletes queries by IDs. Matches Go's `DeleteQueries`.
    /// Returns the number of deleted queries.
    pub async fn delete_queries(&self, ids: &[u32]) -> Result<u64> {
        if ids.is_empty() {
            return Ok(0);
        }

        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let sql = format!(
            "DELETE FROM queries WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut query = sqlx::query(&sql);
        for id in ids {
            query = query.bind(id);
        }

        let result = query.execute(self.pool()).await?;
        Ok(result.rows_affected())
    }

    /// Gets query result rows for a query report.
    /// Matches Go's `QueryResultRowsForHost` / `QueryResultRows`.
    pub async fn query_result_rows(
        &self,
        query_id: u32,
    ) -> Result<Vec<QueryResultRow>> {
        Ok(sqlx::query_as::<_, QueryResultRow>(
            r#"
            SELECT qrr.host_id, qrr.last_fetched, qrr.data,
                   COALESCE(h.hostname, '') as hostname
            FROM query_result_rows qrr
            LEFT JOIN hosts h ON h.id = qrr.host_id
            WHERE qrr.query_id = ?
            ORDER BY qrr.host_id, qrr.last_fetched DESC
            "#,
        )
        .bind(query_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Checks if an observer can run a query. Matches Go's `ObserverCanRunQuery`.
    ///
    /// SELECT observer_can_run FROM queries WHERE id = ?
    pub async fn observer_can_run_query(&self, query_id: u32) -> Result<bool> {
        let row: (bool,) = sqlx::query_as("SELECT observer_can_run FROM queries WHERE id = ?")
            .bind(query_id)
            .fetch_one(self.pool())
            .await?;
        Ok(row.0)
    }

    /// Checks if a query is saved. Matches Go's `IsSavedQuery`.
    ///
    /// SELECT saved FROM queries WHERE id = ?
    pub async fn is_saved_query(&self, query_id: u32) -> Result<bool> {
        let row: (bool,) = sqlx::query_as("SELECT saved FROM queries WHERE id = ?")
            .bind(query_id)
            .fetch_one(self.pool())
            .await?;
        Ok(row.0)
    }

    /// Lists queries with optional team filter. Matches Go's `ListQueries`.
    pub async fn list_queries(
        &self,
        team_id: Option<u32>,
        match_query: &str,
        order_key: &str,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<QueryRow>> {
        let mut sql = r#"
            SELECT
                q.id, q.team_id, q.name, q.description, q.query, q.author_id,
                q.saved, q.observer_can_run, q.schedule_interval, q.platform,
                q.min_osquery_version, q.automations_enabled, q.logging_type,
                q.discard_data, q.created_at, q.updated_at,
                COALESCE(NULLIF(u.name, ''), u.email, '') AS author_name,
                COALESCE(u.email, '') AS author_email,
                JSON_EXTRACT(ag.json_value, '$.user_time_p50') as user_time_p50,
                JSON_EXTRACT(ag.json_value, '$.user_time_p95') as user_time_p95,
                JSON_EXTRACT(ag.json_value, '$.system_time_p50') as system_time_p50,
                JSON_EXTRACT(ag.json_value, '$.system_time_p95') as system_time_p95,
                JSON_EXTRACT(ag.json_value, '$.total_executions') as total_executions
            FROM queries q
            LEFT JOIN users u ON q.author_id = u.id
            LEFT JOIN aggregated_stats ag ON (ag.id = q.id AND ag.global_stats = FALSE AND ag.type = 'scheduled_query')
            WHERE q.saved = TRUE
        "#.to_string();

        if let Some(tid) = team_id {
            sql.push_str(&format!(" AND q.team_id = {}", tid));
        } else {
            sql.push_str(" AND q.team_id IS NULL");
        }

        if !match_query.is_empty() {
            sql.push_str(" AND q.name LIKE CONCAT('%', ?, '%')");
        }

        let order_col = match order_key {
            "name" => "q.name",
            "created_at" => "q.created_at",
            "updated_at" => "q.updated_at",
            _ => "q.name",
        };
        sql.push_str(&format!(" ORDER BY {} ASC", order_col));

        if limit > 0 {
            sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));
        }

        let mut query = sqlx::query_as::<_, QueryRow>(&sql);
        if !match_query.is_empty() {
            query = query.bind(match_query);
        }

        Ok(query.fetch_all(self.pool()).await?)
    }
}
