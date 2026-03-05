//! Script database operations.
//!
//! Implements SQL queries for script management,
//! matching Go's `server/datastore/mysql/scripts.go`.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::error::DatastoreError;
use crate::MysqlDatastore;

/// Row type for scripts table.
#[derive(Debug, Clone, FromRow)]
pub struct ScriptRow {
    pub id: u32,
    pub team_id: Option<u32>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Row type for script content.
#[derive(Debug, Clone, FromRow)]
pub struct ScriptContentRow {
    pub id: u32,
    pub contents: String,
}

impl MysqlDatastore {
    /// Creates a new script record with its content.
    pub async fn new_script(
        &self,
        team_id: Option<u32>,
        name: &str,
        contents: &str,
    ) -> Result<ScriptRow> {
        // Insert script content first
        let content_result = sqlx::query(
            "INSERT INTO script_contents (contents) VALUES (?)",
        )
        .bind(contents)
        .execute(self.pool())
        .await?;
        let content_id = content_result.last_insert_id();

        // Insert script record
        let result = sqlx::query(
            "INSERT INTO scripts (team_id, name, script_content_id) VALUES (?, ?, ?)",
        )
        .bind(team_id)
        .bind(name)
        .bind(content_id)
        .execute(self.pool())
        .await?;

        let id = result.last_insert_id() as u32;
        self.script_by_id(id).await
    }

    /// Retrieves a script by ID.
    pub async fn script_by_id(&self, id: u32) -> Result<ScriptRow> {
        sqlx::query_as::<_, ScriptRow>(
            "SELECT id, team_id, name, created_at, updated_at FROM scripts WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| {
            DatastoreError::NotFound {
                entity: "script".to_string(),
                id: Some(id as u64),
                name: None,
            }
            .into()
        })
    }

    /// Lists scripts, optionally filtered by team.
    pub async fn list_scripts(&self, team_id: Option<u32>) -> Result<Vec<ScriptRow>> {
        let rows = if let Some(tid) = team_id {
            sqlx::query_as::<_, ScriptRow>(
                "SELECT id, team_id, name, created_at, updated_at FROM scripts WHERE team_id = ? ORDER BY name",
            )
            .bind(tid)
            .fetch_all(self.pool())
            .await?
        } else {
            sqlx::query_as::<_, ScriptRow>(
                "SELECT id, team_id, name, created_at, updated_at FROM scripts WHERE team_id IS NULL ORDER BY name",
            )
            .fetch_all(self.pool())
            .await?
        };
        Ok(rows)
    }

    /// Deletes a script by ID.
    pub async fn delete_script(&self, id: u32) -> Result<()> {
        let result = sqlx::query("DELETE FROM scripts WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::NotFound {
                entity: "script".to_string(),
                id: Some(id as u64),
                name: None,
            }
            .into());
        }
        Ok(())
    }

    /// Gets the contents of a script by script ID.
    pub async fn get_script_contents(&self, script_id: u32) -> Result<String> {
        let row: Option<(String,)> = sqlx::query_as(
            "SELECT sc.contents FROM script_contents sc JOIN scripts s ON s.script_content_id = sc.id WHERE s.id = ?",
        )
        .bind(script_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(|r| r.0).ok_or_else(|| {
            DatastoreError::NotFound {
                entity: "script contents".to_string(),
                id: Some(script_id as u64),
                name: None,
            }
            .into()
        })
    }

    /// Creates a new host script execution request.
    pub async fn new_host_script_execution(
        &self,
        host_id: u32,
        execution_id: &str,
        script_id: Option<u32>,
        script_content_id: Option<u64>,
        user_id: Option<u32>,
    ) -> Result<()> {
        sqlx::query(
            r#"INSERT INTO host_script_results
                (host_id, execution_id, script_id, script_content_id, user_id)
                VALUES (?, ?, ?, ?, ?)"#,
        )
        .bind(host_id)
        .bind(execution_id)
        .bind(script_id)
        .bind(script_content_id)
        .bind(user_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }
}
