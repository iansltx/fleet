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
}
