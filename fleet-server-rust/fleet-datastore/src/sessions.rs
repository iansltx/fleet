//! Session CRUD operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/sessions.go`.

use chrono::{DateTime, Utc};

use crate::error::{DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for session queries, matching the sessions table + user JOIN.
#[derive(Debug, sqlx::FromRow)]
pub struct SessionRow {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub accessed_at: DateTime<Utc>,
    pub user_id: u32,
    pub key: String,
    #[sqlx(default)]
    pub api_only: Option<bool>,
}

impl MysqlDatastore {
    /// Creates a new session for a user. Matches Go's `NewSession`.
    ///
    /// INSERT INTO sessions (user_id, `key`) VALUES(?,?)
    pub async fn new_session(&self, user_id: u32, session_key: &str) -> Result<SessionRow> {
        let result = sqlx::query(
            r#"INSERT INTO sessions (user_id, `key`) VALUES(?, ?)"#,
        )
        .bind(user_id)
        .bind(session_key)
        .execute(self.pool())
        .await?;

        let id = result.last_insert_id() as u32;
        self.session_by_id(id).await
    }

    /// Finds a session by its key. Matches Go's `SessionByKey`.
    ///
    /// SELECT s.*, u.api_only FROM sessions s
    /// LEFT JOIN users u ON s.user_id = u.id
    /// WHERE s.`key` = ? LIMIT 1
    pub async fn session_by_key(&self, key: &str) -> Result<SessionRow> {
        sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT s.*, u.api_only FROM sessions s
            LEFT JOIN users u ON s.user_id = u.id
            WHERE s.`key` = ? LIMIT 1
            "#,
        )
        .bind(key)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_name("Session", "<key redacted>"))
    }

    /// Finds a session by its ID. Matches Go's `SessionByID`.
    ///
    /// SELECT s.*, u.api_only FROM sessions s
    /// LEFT JOIN users u ON s.user_id = u.id
    /// WHERE s.id = ? LIMIT 1
    pub async fn session_by_id(&self, id: u32) -> Result<SessionRow> {
        sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT s.*, u.api_only FROM sessions s
            LEFT JOIN users u ON s.user_id = u.id
            WHERE s.id = ? LIMIT 1
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_id("Session", id as u64))
    }

    /// Lists all sessions for a user. Matches Go's `ListSessionsForUser`.
    ///
    /// SELECT s.*, u.api_only FROM sessions s
    /// INNER JOIN users u ON s.user_id = u.id
    /// WHERE s.user_id = ?
    pub async fn list_sessions_for_user(&self, user_id: u32) -> Result<Vec<SessionRow>> {
        Ok(sqlx::query_as::<_, SessionRow>(
            r#"
            SELECT s.*, u.api_only FROM sessions s
            INNER JOIN users u ON s.user_id = u.id
            WHERE s.user_id = ?
            "#,
        )
        .bind(user_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Deletes a session. Matches Go's `DestroySession`.
    ///
    /// DELETE FROM sessions WHERE id = ?
    pub async fn destroy_session(&self, id: u32) -> Result<()> {
        let result = sqlx::query("DELETE FROM sessions WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Session", id as u64));
        }
        Ok(())
    }

    /// Deletes all sessions for a user. Matches Go's `DestroyAllSessionsForUser`.
    ///
    /// DELETE FROM sessions WHERE user_id = ?
    pub async fn destroy_all_sessions_for_user(&self, user_id: u32) -> Result<()> {
        sqlx::query("DELETE FROM sessions WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    /// Updates the accessed_at timestamp for a session. Matches Go's `MarkSessionAccessed`.
    ///
    /// UPDATE sessions SET accessed_at = ? WHERE id = ?
    pub async fn mark_session_accessed(&self, id: u32) -> Result<()> {
        let now = Utc::now();
        let result = sqlx::query("UPDATE sessions SET accessed_at = ? WHERE id = ?")
            .bind(now)
            .bind(id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Session", id as u64));
        }
        Ok(())
    }
}
