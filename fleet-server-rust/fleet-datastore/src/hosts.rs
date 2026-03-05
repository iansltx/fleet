//! Host operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/hosts.go`.

use chrono::{DateTime, Utc};

use crate::error::{DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for host queries matching the hosts table columns.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HostRow {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub detail_updated_at: DateTime<Utc>,
    pub label_updated_at: DateTime<Utc>,
    pub policy_updated_at: DateTime<Utc>,
    pub osquery_host_id: Option<String>,
    pub node_key: Option<String>,
    pub hostname: String,
    pub computer_name: String,
    pub uuid: String,
    pub platform: String,
    pub platform_like: String,
    pub osquery_version: String,
    pub os_version: String,
    pub uptime: i64,
    pub memory: i64,
    pub cpu_type: String,
    pub cpu_subtype: String,
    pub cpu_brand: String,
    pub cpu_physical_cores: i32,
    pub cpu_logical_cores: i32,
    pub hardware_vendor: String,
    pub hardware_model: String,
    pub hardware_version: String,
    pub hardware_serial: String,
    pub team_id: Option<u32>,
    pub distributed_interval: i32,
    pub logger_tls_period: i32,
    pub config_tls_refresh: i32,
    pub primary_ip: String,
    pub primary_mac: String,
    pub public_ip: String,
    pub refetch_requested: bool,
    pub refetch_critical_queries_until: Option<DateTime<Utc>>,
    pub last_enrolled_at: Option<DateTime<Utc>>,
    pub last_restarted_at: Option<DateTime<Utc>>,
}

/// Parameters for creating a new host. Matches Go's `NewHost`.
pub struct NewHostParams {
    pub osquery_host_id: Option<String>,
    pub detail_updated_at: DateTime<Utc>,
    pub label_updated_at: DateTime<Utc>,
    pub policy_updated_at: DateTime<Utc>,
    pub node_key: Option<String>,
    pub hostname: String,
    pub computer_name: String,
    pub uuid: String,
    pub platform: String,
    pub platform_like: String,
    pub osquery_version: String,
    pub os_version: String,
    pub uptime: i64,
    pub memory: i64,
    pub team_id: Option<u32>,
    pub distributed_interval: i32,
    pub logger_tls_period: i32,
    pub config_tls_refresh: i32,
    pub refetch_requested: bool,
    pub hardware_serial: String,
    pub refetch_critical_queries_until: Option<DateTime<Utc>>,
    pub seen_time: DateTime<Utc>,
    pub display_name: String,
}

/// Row type for host summary counts.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HostSummaryRow {
    pub totals_count: i64,
    pub linux_count: i64,
    pub macos_count: i64,
    pub windows_count: i64,
    pub chrome_count: i64,
}

impl MysqlDatastore {
    /// Creates a new host. Matches Go's `NewHost`.
    ///
    /// INSERT INTO hosts (osquery_host_id, detail_updated_at, label_updated_at,
    ///   policy_updated_at, node_key, hostname, computer_name, uuid, platform,
    ///   platform_like, osquery_version, os_version, uptime, memory, team_id,
    ///   distributed_interval, logger_tls_period, config_tls_refresh,
    ///   refetch_requested, hardware_serial, refetch_critical_queries_until)
    /// VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
    pub async fn new_host(&self, params: NewHostParams) -> Result<u32> {
        let mut tx = self.pool().begin().await?;

        let result = sqlx::query(
            r#"
            INSERT INTO hosts (
                osquery_host_id, detail_updated_at, label_updated_at,
                policy_updated_at, node_key, hostname, computer_name, uuid,
                platform, platform_like, osquery_version, os_version, uptime,
                memory, team_id, distributed_interval, logger_tls_period,
                config_tls_refresh, refetch_requested, hardware_serial,
                refetch_critical_queries_until
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&params.osquery_host_id)
        .bind(params.detail_updated_at)
        .bind(params.label_updated_at)
        .bind(params.policy_updated_at)
        .bind(&params.node_key)
        .bind(&params.hostname)
        .bind(&params.computer_name)
        .bind(&params.uuid)
        .bind(&params.platform)
        .bind(&params.platform_like)
        .bind(&params.osquery_version)
        .bind(&params.os_version)
        .bind(params.uptime)
        .bind(params.memory)
        .bind(params.team_id)
        .bind(params.distributed_interval)
        .bind(params.logger_tls_period)
        .bind(params.config_tls_refresh)
        .bind(params.refetch_requested)
        .bind(&params.hardware_serial)
        .bind(params.refetch_critical_queries_until)
        .execute(&mut *tx)
        .await?;

        let host_id = result.last_insert_id() as u32;

        // Insert into host_seen_times (matches Go)
        sqlx::query("INSERT INTO host_seen_times (host_id, seen_time) VALUES (?, ?)")
            .bind(host_id)
            .bind(params.seen_time)
            .execute(&mut *tx)
            .await?;

        // Insert into host_display_names (matches Go)
        sqlx::query("INSERT INTO host_display_names (host_id, display_name) VALUES (?, ?)")
            .bind(host_id)
            .bind(&params.display_name)
            .execute(&mut *tx)
            .await?;

        tx.commit().await?;

        Ok(host_id)
    }

    /// Finds a host by ID. Matches Go's host-by-id pattern.
    ///
    /// SELECT * FROM hosts WHERE id = ?
    pub async fn host_by_id(&self, id: u32) -> Result<HostRow> {
        sqlx::query_as::<_, HostRow>("SELECT * FROM hosts WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_id("Host", id as u64))
    }

    /// Lists hosts with filtering. Simplified version of Go's `ListHosts`.
    /// The full Go implementation is very complex with many filter options.
    pub async fn list_hosts(
        &self,
        team_id: Option<u32>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<HostRow>> {
        let mut sql = "SELECT * FROM hosts WHERE TRUE".to_string();

        if let Some(tid) = team_id {
            sql.push_str(&format!(" AND team_id = {}", tid));
        }

        sql.push_str(&format!(" ORDER BY id ASC LIMIT {} OFFSET {}", limit, offset));

        Ok(sqlx::query_as::<_, HostRow>(&sql)
            .fetch_all(self.pool())
            .await?)
    }

    /// Enrolls a host (inserts or updates based on osquery_host_id).
    /// Matches Go's enrollment pattern using node_key and osquery_host_id.
    ///
    /// This is a simplified version; the full Go implementation handles
    /// multiple enrollment scenarios.
    pub async fn enroll_host(
        &self,
        osquery_host_id: &str,
        node_key: &str,
        team_id: Option<u32>,
        hostname: &str,
        platform: &str,
        hardware_serial: &str,
    ) -> Result<u32> {
        let now = Utc::now();

        // Try to find existing host by osquery_host_id
        let existing: Option<(u32,)> = sqlx::query_as(
            "SELECT id FROM hosts WHERE osquery_host_id = ? LIMIT 1",
        )
        .bind(osquery_host_id)
        .fetch_optional(self.pool())
        .await?;

        if let Some((host_id,)) = existing {
            // Update existing host
            sqlx::query(
                r#"
                UPDATE hosts SET
                    node_key = ?,
                    team_id = ?,
                    last_enrolled_at = ?
                WHERE id = ?
                "#,
            )
            .bind(node_key)
            .bind(team_id)
            .bind(now)
            .bind(host_id)
            .execute(self.pool())
            .await?;

            return Ok(host_id);
        }

        // Create new host
        let result = sqlx::query(
            r#"
            INSERT INTO hosts (
                osquery_host_id, node_key, team_id, hostname, platform,
                hardware_serial, detail_updated_at, label_updated_at,
                policy_updated_at, last_enrolled_at
            )
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(osquery_host_id)
        .bind(node_key)
        .bind(team_id)
        .bind(hostname)
        .bind(platform)
        .bind(hardware_serial)
        .bind(now)
        .bind(now)
        .bind(now)
        .bind(now)
        .execute(self.pool())
        .await?;

        let host_id = result.last_insert_id() as u32;

        // Insert seen time
        sqlx::query("INSERT INTO host_seen_times (host_id, seen_time) VALUES (?, ?)")
            .bind(host_id)
            .bind(now)
            .execute(self.pool())
            .await?;

        Ok(host_id)
    }

    /// Authenticates a host by its node_key. Returns the host if found.
    ///
    /// SELECT * FROM hosts WHERE node_key = ?
    pub async fn authenticate_host(&self, node_key: &str) -> Result<HostRow> {
        sqlx::query_as::<_, HostRow>("SELECT * FROM hosts WHERE node_key = ? LIMIT 1")
            .bind(node_key)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_name("Host", "node_key"))
    }

    /// Marks a host's seen_time. Matches Go's host_seen_times table update.
    ///
    /// INSERT INTO host_seen_times (host_id, seen_time) VALUES (?, ?)
    /// ON DUPLICATE KEY UPDATE seen_time = VALUES(seen_time)
    pub async fn mark_host_seen(&self, host_id: u32, seen_time: DateTime<Utc>) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO host_seen_times (host_id, seen_time) VALUES (?, ?)
            ON DUPLICATE KEY UPDATE seen_time = VALUES(seen_time)
            "#,
        )
        .bind(host_id)
        .bind(seen_time)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    /// Deletes a host by ID.
    ///
    /// DELETE FROM hosts WHERE id = ?
    pub async fn delete_host(&self, id: u32) -> Result<()> {
        let result = sqlx::query("DELETE FROM hosts WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Host", id as u64));
        }
        Ok(())
    }

    /// Gets the display name for a host.
    ///
    /// SELECT display_name FROM host_display_names WHERE host_id = ?
    pub async fn host_display_name(&self, host_id: u32) -> Result<String> {
        let row: Option<(String,)> =
            sqlx::query_as("SELECT display_name FROM host_display_names WHERE host_id = ?")
                .bind(host_id)
                .fetch_optional(self.pool())
                .await?;

        Ok(row.map(|(name,)| name).unwrap_or_default())
    }

    /// Gets host summary counts. Matches Go's `GenerateHostStatusStatistics`.
    pub async fn host_summary(&self) -> Result<HostSummaryRow> {
        let row = sqlx::query_as::<_, HostSummaryRow>(
            r#"
            SELECT
                COUNT(*) as totals_count,
                COALESCE(SUM(CASE WHEN platform = 'linux' OR platform_like LIKE '%linux%' THEN 1 ELSE 0 END), 0) as linux_count,
                COALESCE(SUM(CASE WHEN platform = 'darwin' THEN 1 ELSE 0 END), 0) as macos_count,
                COALESCE(SUM(CASE WHEN platform = 'windows' THEN 1 ELSE 0 END), 0) as windows_count,
                COALESCE(SUM(CASE WHEN platform = 'chrome' THEN 1 ELSE 0 END), 0) as chrome_count
            FROM hosts
            "#,
        )
        .fetch_one(self.pool())
        .await?;

        Ok(row)
    }
}
