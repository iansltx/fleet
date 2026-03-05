//! Policy CRUD operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/policies.go`.

use chrono::{DateTime, Utc};

use crate::error::{is_duplicate, DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Columns selected for policy queries (matches Go's policyCols).
#[allow(dead_code)]
const POLICY_COLS: &str = r#"
    p.id, p.team_id, p.resolution, p.name, p.query, p.description,
    p.author_id, p.platforms, p.created_at, p.updated_at, p.critical,
    p.calendar_events_enabled
"#;

/// Row type for policy queries.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PolicyRow {
    pub id: u32,
    pub team_id: Option<u32>,
    pub resolution: Option<String>,
    pub name: String,
    pub query: String,
    pub description: String,
    pub author_id: Option<u32>,
    pub platforms: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub critical: bool,
    pub calendar_events_enabled: bool,
    // Populated via subqueries/JOINs
    #[sqlx(default)]
    pub author_name: String,
    #[sqlx(default)]
    pub author_email: String,
    #[sqlx(default)]
    pub passing_host_count: Option<u32>,
    #[sqlx(default)]
    pub failing_host_count: Option<u32>,
    #[sqlx(default)]
    pub host_count_updated_at: Option<DateTime<Utc>>,
}

impl MysqlDatastore {
    /// Creates a new global policy. Matches Go's `NewGlobalPolicy`.
    ///
    /// INSERT INTO policies (name, query, description, resolution, author_id, platforms, critical, checksum)
    /// VALUES (?, ?, ?, ?, ?, ?, ?, ...)
    pub async fn new_global_policy(
        &self,
        name: &str,
        query: &str,
        description: &str,
        resolution: Option<&str>,
        author_id: Option<u32>,
        platform: &str,
        critical: bool,
    ) -> Result<u32> {
        let result = sqlx::query(
            r#"
            INSERT INTO policies (name, query, description, resolution, author_id, platforms, critical,
                checksum) VALUES (?, ?, ?, ?, ?, ?, ?, UNHEX(MD5(CONCAT(IFNULL(?, ''), IFNULL(CAST(? AS CHAR), '')))))
            "#,
        )
        .bind(name)
        .bind(query)
        .bind(description)
        .bind(resolution)
        .bind(author_id)
        .bind(platform)
        .bind(critical)
        .bind(name)
        .bind(Option::<u32>::None) // team_id for global
        .execute(self.pool())
        .await;

        match result {
            Ok(res) => Ok(res.last_insert_id() as u32),
            Err(e) => {
                if is_duplicate(&e) {
                    Err(DatastoreError::already_exists("Policy", name))
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Creates a new team policy. Matches Go's `NewTeamPolicy`.
    pub async fn new_team_policy(
        &self,
        team_id: u32,
        name: &str,
        query: &str,
        description: &str,
        resolution: Option<&str>,
        author_id: Option<u32>,
        platform: &str,
        critical: bool,
    ) -> Result<u32> {
        let result = sqlx::query(
            r#"
            INSERT INTO policies (name, query, description, resolution, author_id, platforms, critical, team_id,
                checksum) VALUES (?, ?, ?, ?, ?, ?, ?, ?,
                UNHEX(MD5(CONCAT(IFNULL(?, ''), IFNULL(CAST(? AS CHAR), '')))))
            "#,
        )
        .bind(name)
        .bind(query)
        .bind(description)
        .bind(resolution)
        .bind(author_id)
        .bind(platform)
        .bind(critical)
        .bind(team_id)
        .bind(name)
        .bind(team_id)
        .execute(self.pool())
        .await;

        match result {
            Ok(res) => Ok(res.last_insert_id() as u32),
            Err(e) => {
                if is_duplicate(&e) {
                    Err(DatastoreError::already_exists("Policy", name))
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Gets a policy by ID. Matches Go's `policyDB`.
    pub async fn policy_by_id(&self, id: u32) -> Result<PolicyRow> {
        sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT
                p.id, p.team_id, p.resolution, p.name, p.query, p.description,
                p.author_id, p.platforms, p.created_at, p.updated_at, p.critical,
                p.calendar_events_enabled,
                COALESCE(u.name, '') AS author_name,
                COALESCE(u.email, '') AS author_email,
                ps.passing_host_count,
                ps.failing_host_count,
                ps.updated_at AS host_count_updated_at
            FROM policies p
            LEFT JOIN users u ON p.author_id = u.id
            LEFT JOIN policy_stats ps ON p.id = ps.policy_id AND
                ((p.team_id IS NULL AND ps.inherited_team_id = 0) OR
                 (p.team_id IS NOT NULL AND ps.inherited_team_id = p.team_id))
            WHERE p.id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_id("Policy", id as u64))
    }

    /// Lists global policies. Matches Go's global policy listing.
    pub async fn list_global_policies(&self) -> Result<Vec<PolicyRow>> {
        Ok(sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT
                p.id, p.team_id, p.resolution, p.name, p.query, p.description,
                p.author_id, p.platforms, p.created_at, p.updated_at, p.critical,
                p.calendar_events_enabled,
                COALESCE(u.name, '') AS author_name,
                COALESCE(u.email, '') AS author_email,
                ps.passing_host_count,
                ps.failing_host_count,
                ps.updated_at AS host_count_updated_at
            FROM policies p
            LEFT JOIN users u ON p.author_id = u.id
            LEFT JOIN policy_stats ps ON p.id = ps.policy_id AND ps.inherited_team_id = 0
            WHERE p.team_id IS NULL
            ORDER BY p.name
            "#,
        )
        .fetch_all(self.pool())
        .await?)
    }

    /// Lists team policies. Matches Go's `ListTeamPolicies`.
    pub async fn list_team_policies(&self, team_id: u32) -> Result<Vec<PolicyRow>> {
        Ok(sqlx::query_as::<_, PolicyRow>(
            r#"
            SELECT
                p.id, p.team_id, p.resolution, p.name, p.query, p.description,
                p.author_id, p.platforms, p.created_at, p.updated_at, p.critical,
                p.calendar_events_enabled,
                COALESCE(u.name, '') AS author_name,
                COALESCE(u.email, '') AS author_email,
                ps.passing_host_count,
                ps.failing_host_count,
                ps.updated_at AS host_count_updated_at
            FROM policies p
            LEFT JOIN users u ON p.author_id = u.id
            LEFT JOIN policy_stats ps ON p.id = ps.policy_id AND ps.inherited_team_id = p.team_id
            WHERE p.team_id = ?
            ORDER BY p.name
            "#,
        )
        .bind(team_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Saves/updates a policy. Matches Go's `SavePolicy`.
    pub async fn save_policy(
        &self,
        id: u32,
        name: &str,
        query: &str,
        description: &str,
        resolution: Option<&str>,
        platform: &str,
        critical: bool,
        calendar_events_enabled: bool,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE policies SET
                name = ?, query = ?, description = ?, resolution = ?,
                platforms = ?, critical = ?, calendar_events_enabled = ?
            WHERE id = ?
            "#,
        )
        .bind(name)
        .bind(query)
        .bind(description)
        .bind(resolution)
        .bind(platform)
        .bind(critical)
        .bind(calendar_events_enabled)
        .bind(id)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Policy", id as u64));
        }
        Ok(())
    }

    /// Deletes policies by IDs. Returns deleted IDs.
    pub async fn delete_global_policies(&self, ids: &[u32]) -> Result<Vec<u32>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let sql = format!(
            "DELETE FROM policies WHERE id IN ({}) AND team_id IS NULL",
            placeholders.join(",")
        );

        let mut query = sqlx::query(&sql);
        for id in ids {
            query = query.bind(id);
        }

        query.execute(self.pool()).await?;
        Ok(ids.to_vec())
    }

    /// Deletes team policies by IDs. Returns deleted IDs.
    pub async fn delete_team_policies(&self, team_id: u32, ids: &[u32]) -> Result<Vec<u32>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let sql = format!(
            "DELETE FROM policies WHERE id IN ({}) AND team_id = ?",
            placeholders.join(",")
        );

        let mut query = sqlx::query(&sql);
        for id in ids {
            query = query.bind(id);
        }
        query = query.bind(team_id);

        query.execute(self.pool()).await?;
        Ok(ids.to_vec())
    }
}
