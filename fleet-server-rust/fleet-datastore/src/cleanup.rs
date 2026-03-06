//! Cleanup and aggregation datastore operations.
//!
//! These methods are called by cron jobs to perform periodic maintenance tasks.
//! Matches Go's cleanup methods in `server/datastore/mysql/campaigns.go`,
//! `server/datastore/mysql/carves.go`, `server/datastore/mysql/hosts.go`, etc.

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::MysqlDatastore;

/// Campaign status constants matching Go's DistributedQueryStatus.
const CAMPAIGN_STATUS_WAITING: u8 = 0;
const CAMPAIGN_STATUS_RUNNING: u8 = 1;
const CAMPAIGN_STATUS_COMPLETE: u8 = 2;

impl MysqlDatastore {
    // -----------------------------------------------------------------------
    // Campaign cleanup
    // -----------------------------------------------------------------------

    /// Marks stale distributed query campaigns as complete.
    ///
    /// - Waiting campaigns older than 1 minute → complete
    /// - Running campaigns older than 24 hours → complete
    ///
    /// Matches Go's `CleanupDistributedQueryCampaigns`.
    pub async fn cleanup_distributed_query_campaigns(
        &self,
        now: DateTime<Utc>,
    ) -> Result<u64> {
        let one_minute_ago = now - chrono::Duration::minutes(1);
        let one_day_ago = now - chrono::Duration::hours(24);

        let result = sqlx::query(
            r#"UPDATE distributed_query_campaigns
               SET status = ?
               WHERE (status = ? AND created_at < ?)
                  OR (status = ? AND created_at < ?)"#,
        )
        .bind(CAMPAIGN_STATUS_COMPLETE)
        .bind(CAMPAIGN_STATUS_WAITING)
        .bind(one_minute_ago)
        .bind(CAMPAIGN_STATUS_RUNNING)
        .bind(one_day_ago)
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected())
    }

    /// Deletes targets for completed campaigns older than the given cutoff.
    ///
    /// Deletes in batches (up to 10,000 per batch) to avoid large locks.
    /// Matches Go's `CleanupCompletedCampaignTargets`.
    pub async fn cleanup_completed_campaign_targets(
        &self,
        older_than: DateTime<Utc>,
    ) -> Result<u64> {
        let mut total_deleted: u64 = 0;
        let batch_size: u32 = 10_000;

        loop {
            let ids: Vec<(u64,)> = sqlx::query_as(
                r#"SELECT dqct.id
                   FROM distributed_query_campaign_targets dqct
                   INNER JOIN distributed_query_campaigns dqc
                       ON dqc.id = dqct.distributed_query_campaign_id
                   WHERE dqc.status = ? AND dqc.updated_at < ?
                   ORDER BY dqct.id
                   LIMIT ?"#,
            )
            .bind(CAMPAIGN_STATUS_COMPLETE)
            .bind(older_than)
            .bind(batch_size)
            .fetch_all(self.pool())
            .await?;

            if ids.is_empty() {
                break;
            }

            let placeholders: String = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let sql = format!(
                "DELETE FROM distributed_query_campaign_targets WHERE id IN ({})",
                placeholders
            );
            let mut q = sqlx::query(&sql);
            for (id,) in &ids {
                q = q.bind(id);
            }
            let result = q.execute(self.pool()).await?;
            total_deleted += result.rows_affected();

            if ids.len() < batch_size as usize {
                break;
            }
        }

        Ok(total_deleted)
    }

    /// Returns IDs of campaigns that are complete. Used by frequent_cleanups
    /// to clean up stale Redis live query keys.
    ///
    /// Matches Go's `GetCompletedCampaigns`.
    pub async fn get_completed_campaigns(&self, ids: &[u32]) -> Result<Vec<u32>> {
        if ids.is_empty() {
            return Ok(vec![]);
        }

        let placeholders: String = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
        let sql = format!(
            "SELECT id FROM distributed_query_campaigns WHERE id IN ({}) AND status = ?",
            placeholders
        );
        let mut q = sqlx::query_scalar::<_, u32>(&sql);
        for &id in ids {
            q = q.bind(id);
        }
        q = q.bind(CAMPAIGN_STATUS_COMPLETE);

        Ok(q.fetch_all(self.pool()).await?)
    }

    // -----------------------------------------------------------------------
    // Carve cleanup
    // -----------------------------------------------------------------------

    /// Expires carves older than 24 hours and deletes their block data.
    ///
    /// Matches Go's `CleanupCarves`.
    pub async fn cleanup_carves(&self, now: DateTime<Utc>) -> Result<u64> {
        let cutoff = now - chrono::Duration::hours(24);

        // Get IDs of unexpired carves older than 24h
        let ids: Vec<(i64,)> = sqlx::query_as(
            "SELECT id FROM carve_metadata WHERE expired = 0 AND created_at < ? LIMIT 50000",
        )
        .bind(cutoff)
        .fetch_all(self.pool())
        .await?;

        if ids.is_empty() {
            return Ok(0);
        }

        let count = ids.len() as u64;
        let placeholders: String = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        // Delete block data
        let sql = format!(
            "DELETE FROM carve_blocks WHERE metadata_id IN ({})",
            placeholders
        );
        let mut q = sqlx::query(&sql);
        for (id,) in &ids {
            q = q.bind(id);
        }
        q.execute(self.pool()).await?;

        // Mark metadata as expired
        let sql = format!(
            "UPDATE carve_metadata SET expired = 1 WHERE id IN ({})",
            placeholders
        );
        let mut q = sqlx::query(&sql);
        for (id,) in &ids {
            q = q.bind(id);
        }
        q.execute(self.pool()).await?;

        Ok(count)
    }

    // -----------------------------------------------------------------------
    // Host cleanup
    // -----------------------------------------------------------------------

    /// Removes hosts that enrolled but never reported status details
    /// (hostname, osquery_version, and hardware_serial all empty, older than 5 minutes).
    ///
    /// Matches Go's `CleanupIncomingHosts`.
    pub async fn cleanup_incoming_hosts(&self, now: DateTime<Utc>) -> Result<Vec<u32>> {
        let cutoff = now - chrono::Duration::minutes(5);

        // Find incoming host IDs
        let ids: Vec<(u32,)> = sqlx::query_as(
            r#"SELECT id FROM hosts
               WHERE hostname = '' AND osquery_version = '' AND hardware_serial = ''
               AND created_at < ?"#,
        )
        .bind(cutoff)
        .fetch_all(self.pool())
        .await?;

        if ids.is_empty() {
            return Ok(vec![]);
        }

        let host_ids: Vec<u32> = ids.iter().map(|(id,)| *id).collect();
        let placeholders: String = ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");

        // Delete display names
        let sql = format!(
            "DELETE FROM host_display_names WHERE host_id IN ({})",
            placeholders
        );
        let mut q = sqlx::query(&sql);
        for &id in &host_ids {
            q = q.bind(id);
        }
        q.execute(self.pool()).await?;

        // Delete host_seen_times
        let sql = format!(
            "DELETE FROM host_seen_times WHERE host_id IN ({})",
            placeholders
        );
        let mut q = sqlx::query(&sql);
        for &id in &host_ids {
            q = q.bind(id);
        }
        q.execute(self.pool()).await?;

        // Delete hosts
        let sql = format!("DELETE FROM hosts WHERE id IN ({})", placeholders);
        let mut q = sqlx::query(&sql);
        for &id in &host_ids {
            q = q.bind(id);
        }
        q.execute(self.pool()).await?;

        Ok(host_ids)
    }

    // -----------------------------------------------------------------------
    // Policy cleanup
    // -----------------------------------------------------------------------

    /// Removes stale policy membership for hosts that have been removed.
    ///
    /// Matches Go's `CleanupPolicyMembership`.
    pub async fn cleanup_policy_membership(&self, now: DateTime<Utc>) -> Result<u64> {
        let _ = now;
        let result = sqlx::query(
            r#"DELETE pm FROM policy_membership pm
               LEFT JOIN hosts h ON pm.host_id = h.id
               WHERE h.id IS NULL"#,
        )
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected())
    }

    // -----------------------------------------------------------------------
    // Password reset cleanup
    // -----------------------------------------------------------------------

    /// Deletes expired password reset requests (older than 24 hours by default).
    ///
    /// Matches Go's `CleanupExpiredPasswordResetRequests`.
    pub async fn cleanup_expired_password_reset_requests(&self) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM password_reset_requests WHERE created_at < DATE_SUB(NOW(), INTERVAL 24 HOUR)",
        )
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected())
    }

    // -----------------------------------------------------------------------
    // Session cleanup (for expired/idle sessions)
    // -----------------------------------------------------------------------

    /// Deletes sessions that haven't been accessed within the given duration.
    pub async fn cleanup_expired_sessions(&self, max_idle_secs: u64) -> Result<u64> {
        let result = sqlx::query(
            "DELETE FROM sessions WHERE accessed_at < DATE_SUB(NOW(), INTERVAL ? SECOND)",
        )
        .bind(max_idle_secs)
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected())
    }

    // -----------------------------------------------------------------------
    // Host operating system cleanup
    // -----------------------------------------------------------------------

    /// Removes operating system entries not associated with any host.
    ///
    /// Matches Go's `CleanupHostOperatingSystems`.
    pub async fn cleanup_host_operating_systems(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"DELETE os FROM operating_systems os
               LEFT JOIN host_operating_system hos ON os.id = hos.os_id
               WHERE hos.os_id IS NULL"#,
        )
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected())
    }

    // -----------------------------------------------------------------------
    // Query results cleanup
    // -----------------------------------------------------------------------

    /// Deletes all query_results when query reports are globally disabled.
    ///
    /// Matches Go's `CleanupGlobalDiscardQueryResults`.
    pub async fn cleanup_global_discard_query_results(&self) -> Result<u64> {
        let result = sqlx::query("DELETE FROM query_results")
            .execute(self.pool())
            .await?;
        Ok(result.rows_affected())
    }

    /// Deletes query_results for queries that have discard_data enabled.
    ///
    /// Matches Go's `CleanupDiscardedQueryResults`.
    pub async fn cleanup_discarded_query_results(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"DELETE qr FROM query_results qr
               INNER JOIN queries q ON qr.query_id = q.id
               WHERE q.discard_data = 1"#,
        )
        .execute(self.pool())
        .await?;
        Ok(result.rows_affected())
    }

    /// Deletes excess query result rows that exceed the maximum allowed per query.
    /// Returns a map of query_id → remaining row count.
    ///
    /// Matches Go's `CleanupExcessQueryResultRows`.
    pub async fn cleanup_excess_query_result_rows(
        &self,
        max_rows: u32,
    ) -> Result<std::collections::HashMap<u32, u32>> {
        let mut counts = std::collections::HashMap::new();
        if max_rows == 0 {
            return Ok(counts);
        }

        // Get saved query IDs with snapshot logging
        let query_ids: Vec<(u32,)> = sqlx::query_as(
            "SELECT id FROM queries WHERE saved = 1 AND discard_data = 0 AND logging_type = 'snapshot'",
        )
        .fetch_all(self.pool())
        .await?;

        for (query_id,) in query_ids {
            // Count rows for this query
            let (row_count,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM query_results WHERE query_id = ? AND data IS NOT NULL",
            )
            .bind(query_id)
            .fetch_one(self.pool())
            .await?;

            if row_count <= max_rows as i64 {
                counts.insert(query_id, row_count as u32);
                continue;
            }

            // Find the cutoff ID to keep only the most recent max_rows
            let cutoff: Option<(u64,)> = sqlx::query_as(
                r#"SELECT id FROM query_results
                   WHERE query_id = ? AND data IS NOT NULL
                   ORDER BY id DESC
                   LIMIT 1 OFFSET ?"#,
            )
            .bind(query_id)
            .bind(max_rows)
            .fetch_optional(self.pool())
            .await?;

            if let Some((cutoff_id,)) = cutoff {
                // Delete in batches
                let batch_limit: u32 = 10_000;
                loop {
                    let result = sqlx::query(
                        "DELETE FROM query_results WHERE query_id = ? AND id < ? AND data IS NOT NULL LIMIT ?",
                    )
                    .bind(query_id)
                    .bind(cutoff_id)
                    .bind(batch_limit)
                    .execute(self.pool())
                    .await?;

                    if result.rows_affected() < batch_limit as u64 {
                        break;
                    }
                }
            }

            // Get the actual remaining count
            let (remaining,): (i64,) = sqlx::query_as(
                "SELECT COUNT(*) FROM query_results WHERE query_id = ? AND data IS NOT NULL",
            )
            .bind(query_id)
            .fetch_one(self.pool())
            .await?;

            counts.insert(query_id, remaining as u32);
        }

        Ok(counts)
    }

    // -----------------------------------------------------------------------
    // Aggregation methods
    // -----------------------------------------------------------------------

    /// Updates aggregated query performance statistics (p50, p95 percentiles).
    ///
    /// Matches Go's `UpdateQueryAggregatedStats`.
    pub async fn update_query_aggregated_stats(&self) -> Result<()> {
        // Get all scheduled query IDs that have stats
        let query_ids: Vec<(u32,)> = sqlx::query_as(
            "SELECT DISTINCT scheduled_query_id FROM scheduled_query_stats WHERE executions > 0",
        )
        .fetch_all(self.pool())
        .await?;

        for (query_id,) in query_ids {
            // Calculate total executions
            let (total_executions,): (i64,) = sqlx::query_as(
                "SELECT COALESCE(SUM(executions), 0) FROM scheduled_query_stats WHERE scheduled_query_id = ?",
            )
            .bind(query_id)
            .fetch_one(self.pool())
            .await?;

            // Calculate user_time and system_time averages per host, then compute percentiles
            let host_stats: Vec<(f64, f64)> = sqlx::query_as(
                r#"SELECT
                       COALESCE(SUM(user_time) / NULLIF(SUM(executions), 0), 0) as avg_user_time,
                       COALESCE(SUM(system_time) / NULLIF(SUM(executions), 0), 0) as avg_system_time
                   FROM scheduled_query_stats
                   WHERE scheduled_query_id = ? AND executions > 0
                   GROUP BY host_id
                   ORDER BY avg_user_time"#,
            )
            .bind(query_id)
            .fetch_all(self.pool())
            .await?;

            if host_stats.is_empty() {
                continue;
            }

            let n = host_stats.len();
            let p50_idx = (n as f64 * 0.5).floor() as usize;
            let p95_idx = (n as f64 * 0.95).floor() as usize;
            let p50_idx = p50_idx.min(n - 1);
            let p95_idx = p95_idx.min(n - 1);

            // Sort by user_time for user_time percentiles
            let user_time_p50 = host_stats[p50_idx].0;
            let user_time_p95 = host_stats[p95_idx].0;

            // Sort by system_time for system_time percentiles
            let mut sys_sorted = host_stats.clone();
            sys_sorted.sort_by(|a, b| a.1.partial_cmp(&b.1).unwrap_or(std::cmp::Ordering::Equal));
            let system_time_p50 = sys_sorted[p50_idx].1;
            let system_time_p95 = sys_sorted[p95_idx].1;

            let json_value = serde_json::json!({
                "user_time_p50": user_time_p50,
                "user_time_p95": user_time_p95,
                "system_time_p50": system_time_p50,
                "system_time_p95": system_time_p95,
                "total_executions": total_executions,
            });

            // Upsert into aggregated_stats
            sqlx::query(
                r#"INSERT INTO aggregated_stats (id, type, global_stats, json_value)
                   VALUES (?, 'scheduled_query', 0, ?)
                   ON DUPLICATE KEY UPDATE json_value = VALUES(json_value)"#,
            )
            .bind(query_id)
            .bind(json_value.to_string())
            .execute(self.pool())
            .await?;
        }

        Ok(())
    }

    /// Updates host policy violation counts.
    ///
    /// Matches Go's `UpdateHostPolicyCounts`.
    pub async fn update_host_policy_counts(&self) -> Result<()> {
        // Count failing policies per host and update the count
        sqlx::query(
            r#"INSERT INTO host_issues (host_id, total_issues_count)
               SELECT host_id, COUNT(*) as failing_count
               FROM policy_membership
               WHERE passes = 0
               GROUP BY host_id
               ON DUPLICATE KEY UPDATE total_issues_count = VALUES(total_issues_count)"#,
        )
        .execute(self.pool())
        .await?;

        // Remove entries for hosts with no failing policies
        sqlx::query(
            r#"DELETE hi FROM host_issues hi
               LEFT JOIN (
                   SELECT host_id FROM policy_membership WHERE passes = 0 GROUP BY host_id
               ) pm ON hi.host_id = pm.host_id
               WHERE pm.host_id IS NULL"#,
        )
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Regenerates aggregated munki and MDM statistics.
    ///
    /// Matches Go's `GenerateAggregatedMunkiAndMDM`.
    pub async fn generate_aggregated_munki_and_mdm(&self) -> Result<()> {
        // Aggregate MDM status counts
        let mdm_status: Vec<(Option<u32>, String, i64)> = sqlx::query_as(
            r#"SELECT h.team_id,
                   CASE WHEN hm.enrolled = 1 THEN 'enrolled_manual'
                        WHEN hm.installed_from_dep = 1 THEN 'enrolled_automated'
                        ELSE 'unenrolled' END as status,
                   COUNT(*) as count
               FROM hosts h
               LEFT JOIN host_mdm hm ON h.id = hm.host_id
               GROUP BY h.team_id, status"#,
        )
        .fetch_all(self.pool())
        .await?;

        // Build JSON for each team
        let mut team_stats: std::collections::HashMap<Option<u32>, serde_json::Value> =
            std::collections::HashMap::new();
        for (team_id, status, count) in &mdm_status {
            let entry = team_stats
                .entry(*team_id)
                .or_insert_with(|| serde_json::json!({}));
            entry[status] = serde_json::json!(count);
        }

        // Upsert global and per-team MDM status stats
        for (team_id, json_value) in &team_stats {
            let (id, global) = match team_id {
                Some(tid) => (*tid, false),
                None => (0, true),
            };
            sqlx::query(
                r#"INSERT INTO aggregated_stats (id, type, global_stats, json_value)
                   VALUES (?, 'mdm_status', ?, ?)
                   ON DUPLICATE KEY UPDATE json_value = VALUES(json_value)"#,
            )
            .bind(id)
            .bind(global)
            .bind(json_value.to_string())
            .execute(self.pool())
            .await?;
        }

        // Aggregate munki versions
        let munki_versions: Vec<(Option<u32>, String, i64)> = sqlx::query_as(
            r#"SELECT h.team_id, hmi.version, COUNT(*) as count
               FROM hosts h
               INNER JOIN host_munki_info hmi ON h.id = hmi.host_id
               WHERE hmi.deleted_at IS NULL
               GROUP BY h.team_id, hmi.version"#,
        )
        .fetch_all(self.pool())
        .await?;

        let mut team_munki: std::collections::HashMap<Option<u32>, Vec<serde_json::Value>> =
            std::collections::HashMap::new();
        for (team_id, version, count) in &munki_versions {
            team_munki
                .entry(*team_id)
                .or_default()
                .push(serde_json::json!({"version": version, "hosts_count": count}));
        }

        for (team_id, versions) in &team_munki {
            let (id, global) = match team_id {
                Some(tid) => (*tid, false),
                None => (0, true),
            };
            sqlx::query(
                r#"INSERT INTO aggregated_stats (id, type, global_stats, json_value)
                   VALUES (?, 'munki_versions', ?, ?)
                   ON DUPLICATE KEY UPDATE json_value = VALUES(json_value)"#,
            )
            .bind(id)
            .bind(global)
            .bind(serde_json::to_string(versions).unwrap_or_default())
            .execute(self.pool())
            .await?;
        }

        Ok(())
    }

    // -----------------------------------------------------------------------
    // Script content cleanup
    // -----------------------------------------------------------------------

    /// Removes script_contents entries not referenced by any script or execution.
    ///
    /// Matches Go's `CleanupUnusedScriptContents`.
    pub async fn cleanup_unused_script_contents(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"DELETE sc FROM script_contents sc
               LEFT JOIN scripts s ON sc.id = s.script_content_id
               LEFT JOIN host_script_results hsr ON sc.id = hsr.script_content_id
               WHERE s.id IS NULL AND hsr.script_content_id IS NULL"#,
        )
        .execute(self.pool())
        .await?;
        Ok(result.rows_affected())
    }

    // -----------------------------------------------------------------------
    // Cron stats cleanup
    // -----------------------------------------------------------------------

    /// Cleans up old cron_stats entries and expires stuck ones.
    ///
    /// Matches Go's `CleanupCronStats`.
    pub async fn cleanup_cron_stats(&self) -> Result<()> {
        // Delete entries older than 2 days
        sqlx::query("DELETE FROM cron_stats WHERE created_at < DATE_SUB(NOW(), INTERVAL 2 DAY)")
            .execute(self.pool())
            .await?;

        // Mark pending/queued entries as expired if they're older than 2 hours
        // without an active lock, or older than 12 hours regardless
        sqlx::query(
            r#"UPDATE cron_stats cs
               SET cs.status = 'expired'
               WHERE cs.status IN ('pending', 'queued')
               AND (
                   (cs.created_at < DATE_SUB(NOW(), INTERVAL 2 HOUR)
                    AND NOT EXISTS (
                        SELECT 1 FROM locks l
                        WHERE l.name = cs.name
                        AND l.expires_at >= CURRENT_TIMESTAMP
                    ))
                   OR cs.created_at < DATE_SUB(NOW(), INTERVAL 12 HOUR)
               )"#,
        )
        .execute(self.pool())
        .await?;

        Ok(())
    }

    // -----------------------------------------------------------------------
    // App config helpers for cron jobs
    // -----------------------------------------------------------------------

    /// Returns whether query reports are globally disabled.
    pub async fn are_query_reports_disabled(&self) -> Result<bool> {
        let config = self.app_config().await?;
        let disabled = config
            .get("server_settings")
            .and_then(|s| s.get("query_reports_disabled"))
            .and_then(|v| v.as_bool())
            .unwrap_or(false);
        Ok(disabled)
    }

    /// Returns the max query report rows (default 1000).
    pub async fn get_query_report_cap(&self) -> Result<u32> {
        let config = self.app_config().await?;
        let cap = config
            .get("server_settings")
            .and_then(|s| s.get("query_report_cap"))
            .and_then(|v| v.as_u64())
            .unwrap_or(1000) as u32;
        Ok(cap)
    }
}
