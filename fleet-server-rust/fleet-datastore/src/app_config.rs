//! AppConfig CRUD operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/app_configs.go`.

use crate::error::Result;
use crate::mysql::MysqlDatastore;

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
