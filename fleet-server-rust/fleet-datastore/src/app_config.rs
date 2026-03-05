//! AppConfig CRUD operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/app_configs.go`.

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::mysql::MysqlDatastore;

/// Row type for secret_variables table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SecretVariableRow {
    pub id: u32,
    pub name: String,
    pub value: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MysqlDatastore {
    /// Loads the app config from the database. Matches Go's `AppConfig`.
    ///
    /// SELECT json_value FROM app_config_json LIMIT 1
    pub async fn app_config(&self) -> Result<serde_json::Value> {
        let row: Option<(Vec<u8>,)> =
            sqlx::query_as("SELECT json_value FROM app_config_json LIMIT 1")
                .fetch_optional(self.pool())
                .await?;

        match row {
            Some((bytes,)) => {
                let config: serde_json::Value = serde_json::from_slice(&bytes)?;
                Ok(config)
            }
            None => Ok(serde_json::json!({})),
        }
    }

    /// Saves the app config. Matches Go's `SaveAppConfig`.
    ///
    /// INSERT INTO app_config_json(json_value) VALUES(?)
    /// ON DUPLICATE KEY UPDATE json_value = VALUES(json_value)
    pub async fn save_app_config(&self, config: &serde_json::Value) -> Result<()> {
        let config_bytes = serde_json::to_vec(config)?;

        sqlx::query(
            r#"
            INSERT INTO app_config_json(json_value) VALUES(?)
            ON DUPLICATE KEY UPDATE json_value = VALUES(json_value)
            "#,
        )
        .bind(config_bytes)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Lists all secret variables.
    pub async fn list_secret_variables(&self) -> Result<Vec<SecretVariableRow>> {
        Ok(sqlx::query_as::<_, SecretVariableRow>(
            "SELECT id, name, value, created_at, updated_at FROM secret_variables ORDER BY name",
        )
        .fetch_all(self.pool())
        .await?)
    }

    /// Creates a new secret variable.
    pub async fn create_secret_variable(&self, name: &str, value: &str) -> Result<u32> {
        let result = sqlx::query(
            "INSERT INTO secret_variables (name, value) VALUES (?, ?)",
        )
        .bind(name)
        .bind(value)
        .execute(self.pool())
        .await?;
        Ok(result.last_insert_id() as u32)
    }

    /// Deletes a secret variable by ID.
    pub async fn delete_secret_variable(&self, id: u32) -> Result<()> {
        let result = sqlx::query("DELETE FROM secret_variables WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(crate::error::DatastoreError::not_found_with_id("SecretVariable", id as u64));
        }
        Ok(())
    }

    /// Upserts secret variables (batch create/update by name).
    pub async fn upsert_secret_variables(&self, secrets: &[(String, String)]) -> Result<()> {
        for (name, value) in secrets {
            sqlx::query(
                r#"
                INSERT INTO secret_variables (name, value) VALUES (?, ?)
                ON DUPLICATE KEY UPDATE value = VALUES(value)
                "#,
            )
            .bind(name)
            .bind(value)
            .execute(self.pool())
            .await?;
        }
        Ok(())
    }

    /// Gets a secret variable by ID.
    pub async fn secret_variable_by_id(&self, id: u32) -> Result<SecretVariableRow> {
        sqlx::query_as::<_, SecretVariableRow>(
            "SELECT id, name, value, created_at, updated_at FROM secret_variables WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| crate::error::DatastoreError::not_found_with_id("SecretVariable", id as u64))
    }

    /// Gets the current database time. Matches Go's `GetCurrentTime`.
    ///
    /// SELECT NOW()
    pub async fn get_current_time(&self) -> Result<chrono::DateTime<chrono::Utc>> {
        let (now,): (chrono::DateTime<chrono::Utc>,) = sqlx::query_as("SELECT NOW()")
            .fetch_one(self.pool())
            .await?;
        Ok(now)
    }
}
