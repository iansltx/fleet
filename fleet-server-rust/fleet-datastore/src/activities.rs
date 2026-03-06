//! Activity logging operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/activities.go`.

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::mysql::MysqlDatastore;

/// Row type for activity log entries matching the activities table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ActivityRow {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub user_id: Option<u32>,
    pub user_name: Option<String>,
    pub user_email: Option<String>,
    pub activity_type: String,
    pub details: Option<serde_json::Value>,
    #[sqlx(default)]
    pub streamed: bool,
}

/// Row type for upcoming activities.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct UpcomingActivityRow {
    pub id: u32,
    pub host_id: u32,
    pub user_id: Option<u32>,
    pub activity_type: String,
    pub execution_id: String,
    pub created_at: DateTime<Utc>,
    pub activated_at: Option<DateTime<Utc>>,
    pub fleet_initiated: bool,
    pub priority: i32,
}

impl MysqlDatastore {
    /// Creates a new activity log entry. Matches Go's `NewActivity`.
    ///
    /// INSERT INTO activities (user_id, user_name, activity_type, details)
    /// VALUES (?, ?, ?, ?)
    pub async fn new_activity(
        &self,
        user_id: Option<u32>,
        user_name: Option<&str>,
        user_email: Option<&str>,
        activity_type: &str,
        details: &serde_json::Value,
    ) -> Result<u32> {
        let details_bytes = serde_json::to_vec(details)?;

        let result = sqlx::query(
            r#"
            INSERT INTO activities (user_id, user_name, user_email, activity_type, details)
            VALUES (?, ?, ?, ?, ?)
            "#,
        )
        .bind(user_id)
        .bind(user_name)
        .bind(user_email)
        .bind(activity_type)
        .bind(details_bytes)
        .execute(self.pool())
        .await?;

        Ok(result.last_insert_id() as u32)
    }

    /// Lists activities with pagination. Matches Go's `ListActivities`.
    ///
    /// SELECT a.id, a.created_at, a.user_id, a.user_name, a.user_email,
    ///   a.activity_type, a.details, a.streamed
    /// FROM activities a
    /// ORDER BY a.created_at DESC
    /// LIMIT ? OFFSET ?
    pub async fn list_activities(
        &self,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<ActivityRow>> {
        Ok(sqlx::query_as::<_, ActivityRow>(
            r#"
            SELECT id, created_at, user_id, user_name, user_email,
                activity_type, details, streamed
            FROM activities
            ORDER BY created_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool())
        .await?)
    }

    /// Lists upcoming activities for a host. Matches Go's `ListHostUpcomingActivities`.
    ///
    /// SELECT COUNT(*) FROM upcoming_activities WHERE host_id = ?
    pub async fn count_host_upcoming_activities(&self, host_id: u32) -> Result<u32> {
        let (count,): (u32,) = sqlx::query_as(
            "SELECT COUNT(*) FROM upcoming_activities WHERE host_id = ?",
        )
        .bind(host_id)
        .fetch_one(self.pool())
        .await?;
        Ok(count)
    }

    /// Lists upcoming activities for a host.
    ///
    /// SELECT * FROM upcoming_activities WHERE host_id = ? ORDER BY priority DESC, created_at ASC
    pub async fn list_host_upcoming_activities(&self, host_id: u32) -> Result<Vec<UpcomingActivityRow>> {
        Ok(sqlx::query_as::<_, UpcomingActivityRow>(
            r#"
            SELECT id, host_id, user_id, activity_type, execution_id,
                created_at, activated_at, fleet_initiated, priority
            FROM upcoming_activities
            WHERE host_id = ?
            ORDER BY priority DESC, created_at ASC
            "#,
        )
        .bind(host_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Deletes an upcoming activity by ID and host ID.
    ///
    /// DELETE FROM upcoming_activities WHERE id = ? AND host_id = ?
    pub async fn delete_host_upcoming_activity(&self, host_id: u32, activity_id: u32) -> Result<()> {
        let result = sqlx::query(
            "DELETE FROM upcoming_activities WHERE id = ? AND host_id = ?",
        )
        .bind(activity_id)
        .bind(host_id)
        .execute(self.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Err(crate::error::DatastoreError::not_found_with_id("UpcomingActivity", activity_id as u64));
        }
        Ok(())
    }

    /// Marks activities as streamed. Used for activity streaming/webhooks.
    ///
    /// UPDATE activities SET streamed = TRUE WHERE id IN (...)
    pub async fn mark_activities_streamed(&self, ids: &[u32]) -> Result<()> {
        if ids.is_empty() {
            return Ok(());
        }

        let placeholders: Vec<&str> = ids.iter().map(|_| "?").collect();
        let sql = format!(
            "UPDATE activities SET streamed = TRUE WHERE id IN ({})",
            placeholders.join(",")
        );

        let mut query = sqlx::query(&sql);
        for id in ids {
            query = query.bind(id);
        }

        query.execute(self.pool()).await?;
        Ok(())
    }

    /// Unblocks hosts whose upcoming activity queue is stuck (no active entry).
    /// Matches Go's `UnblockHostsUpcomingActivityQueue`.
    ///
    /// Finds hosts that have upcoming activities but none with activated_at set,
    /// then activates the next activity for each.
    pub async fn unblock_hosts_upcoming_activity_queue(&self, max_hosts: u32) -> Result<u32> {
        let blocked_host_ids: Vec<(u32,)> = sqlx::query_as(
            r#"
            SELECT DISTINCT inactive_ua.host_id
            FROM upcoming_activities inactive_ua
            LEFT OUTER JOIN upcoming_activities active_ua
                ON active_ua.host_id = inactive_ua.host_id
                AND active_ua.activated_at IS NOT NULL
            WHERE active_ua.host_id IS NULL
                AND inactive_ua.activated_at IS NULL
            LIMIT ?
            "#,
        )
        .bind(max_hosts)
        .fetch_all(self.pool())
        .await?;

        if blocked_host_ids.is_empty() {
            return Ok(0);
        }

        let count = blocked_host_ids.len() as u32;

        // Activate the next upcoming activity for each blocked host
        for (host_id,) in &blocked_host_ids {
            self.activate_next_upcoming_activity(*host_id).await?;
        }

        Ok(count)
    }

    /// Activates the next upcoming activity for a host by setting activated_at
    /// on the highest-priority, oldest pending activity.
    /// Matches Go's `activateNextUpcomingActivity`.
    async fn activate_next_upcoming_activity(&self, host_id: u32) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE upcoming_activities
            SET activated_at = NOW()
            WHERE host_id = ?
                AND activated_at IS NULL
            ORDER BY priority DESC, created_at ASC
            LIMIT 1
            "#,
        )
        .bind(host_id)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Marks batch activities as completed when all targeted hosts have results.
    /// Matches Go's `MarkActivitiesAsCompleted` in `server/datastore/mysql/scripts.go`.
    pub async fn mark_activities_as_completed(&self) -> Result<u64> {
        let result = sqlx::query(
            r#"
            UPDATE batch_activities AS ba
            JOIN (
                SELECT
                    ba2.id AS batch_id,
                    COUNT(bahr.host_id) AS num_targeted,
                    COUNT(bahr.error) AS num_incompatible,
                    COUNT(IF(hsr.exit_code = 0, 1, NULL)) AS num_ran,
                    COUNT(IF(hsr.exit_code <> 0, 1, NULL)) AS num_errored,
                    COUNT(IF(
                        (hsr.canceled = 1 AND hsr.exit_code IS NULL) OR
                        (hsr.host_id IS NULL AND bahr.error IS NULL AND ba2.canceled = 1),
                        1, NULL
                    )) AS num_canceled
                FROM batch_activities AS ba2
                LEFT JOIN batch_activity_host_results AS bahr
                    ON ba2.execution_id = bahr.batch_execution_id
                LEFT JOIN host_script_results AS hsr
                    ON bahr.host_execution_id = hsr.execution_id
                WHERE ba2.status = 'started'
                GROUP BY ba2.id
                HAVING (num_incompatible + num_ran + num_errored + num_canceled) >= num_targeted
            ) AS agg
                ON agg.batch_id = ba.id
            SET
                ba.status = 'finished',
                ba.finished_at = NOW(),
                ba.num_targeted = agg.num_targeted,
                ba.num_incompatible = agg.num_incompatible,
                ba.num_ran = agg.num_ran,
                ba.num_errored = agg.num_errored,
                ba.num_canceled = agg.num_canceled,
                ba.num_pending = 0
            WHERE ba.status = 'started'
            "#,
        )
        .execute(self.pool())
        .await?;

        Ok(result.rows_affected())
    }
}
