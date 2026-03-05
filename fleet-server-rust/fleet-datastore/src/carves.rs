//! Carve database operations.
//!
//! Implements SQL queries for file carve management,
//! matching Go's `server/datastore/mysql/carves.go`.

use anyhow::Result;
use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::error::DatastoreError;
use crate::MysqlDatastore;

/// Maximum carve size: 8GB.
pub const MAX_CARVE_SIZE: i64 = 8 * 1024 * 1024 * 1024;
/// Maximum block size: 256MB.
pub const MAX_BLOCK_SIZE: i64 = 256 * 1024 * 1024;

/// Row type for carve_metadata table.
#[derive(Debug, Clone, FromRow)]
pub struct CarveRow {
    pub id: i64,
    pub host_id: u32,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub block_count: i64,
    pub block_size: i64,
    pub carve_size: i64,
    pub carve_id: String,
    pub request_id: String,
    pub session_id: String,
    pub expired: bool,
    pub max_block: i64,
    pub error: Option<String>,
}

impl MysqlDatastore {
    /// Creates a new carve metadata record.
    pub async fn new_carve(&self, carve: &CarveRow) -> Result<CarveRow> {
        let result = sqlx::query(
            r#"INSERT INTO carve_metadata
                (host_id, created_at, name, block_count, block_size, carve_size, carve_id, request_id, session_id, error)
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)"#,
        )
        .bind(carve.host_id)
        .bind(carve.created_at)
        .bind(&carve.name)
        .bind(carve.block_count)
        .bind(carve.block_size)
        .bind(carve.carve_size)
        .bind(&carve.carve_id)
        .bind(&carve.request_id)
        .bind(&carve.session_id)
        .bind(&carve.error)
        .execute(self.pool())
        .await?;

        let id = result.last_insert_id() as i64;
        self.carve_by_id(id).await
    }

    /// Updates carve metadata (max_block, expired, error).
    pub async fn update_carve(&self, id: i64, max_block: i64, expired: bool, error: Option<&str>) -> Result<()> {
        sqlx::query("UPDATE carve_metadata SET max_block = ?, expired = ?, error = ? WHERE id = ?")
            .bind(max_block)
            .bind(expired)
            .bind(error)
            .bind(id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    /// Retrieves a carve by ID.
    pub async fn carve_by_id(&self, id: i64) -> Result<CarveRow> {
        sqlx::query_as::<_, CarveRow>(
            "SELECT id, host_id, created_at, name, block_count, block_size, carve_size, carve_id, request_id, session_id, expired, max_block, error FROM carve_metadata WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| {
            DatastoreError::NotFound {
                entity: "carve".to_string(),
                id: Some(id as u64),
                name: None,
            }
            .into()
        })
    }

    /// Retrieves a carve by session ID.
    pub async fn carve_by_session_id(&self, session_id: &str) -> Result<CarveRow> {
        sqlx::query_as::<_, CarveRow>(
            "SELECT id, host_id, created_at, name, block_count, block_size, carve_size, carve_id, request_id, session_id, expired, max_block, error FROM carve_metadata WHERE session_id = ?",
        )
        .bind(session_id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| {
            DatastoreError::NotFound {
                entity: "carve".to_string(),
                id: None,
                name: Some(session_id.to_string()),
            }
            .into()
        })
    }

    /// Lists carves, optionally including expired ones.
    pub async fn list_carves(&self, include_expired: bool) -> Result<Vec<CarveRow>> {
        let query = if include_expired {
            "SELECT id, host_id, created_at, name, block_count, block_size, carve_size, carve_id, request_id, session_id, expired, max_block, error FROM carve_metadata ORDER BY id DESC"
        } else {
            "SELECT id, host_id, created_at, name, block_count, block_size, carve_size, carve_id, request_id, session_id, expired, max_block, error FROM carve_metadata WHERE NOT expired ORDER BY id DESC"
        };
        let rows = sqlx::query_as::<_, CarveRow>(query)
            .fetch_all(self.pool())
            .await?;
        Ok(rows)
    }

    /// Inserts a new carve block and updates max_block.
    pub async fn new_carve_block(&self, metadata_id: i64, block_id: i64, data: &[u8]) -> Result<()> {
        sqlx::query("INSERT INTO carve_blocks (metadata_id, block_id, data) VALUES (?, ?, ?)")
            .bind(metadata_id)
            .bind(block_id)
            .bind(data)
            .execute(self.pool())
            .await?;

        // Update max_block if this is a new highest block
        sqlx::query("UPDATE carve_metadata SET max_block = GREATEST(max_block, ?) WHERE id = ?")
            .bind(block_id)
            .bind(metadata_id)
            .execute(self.pool())
            .await?;

        Ok(())
    }

    /// Retrieves a carve block by metadata ID and block ID.
    pub async fn get_carve_block(&self, metadata_id: i64, block_id: i64) -> Result<Vec<u8>> {
        let row: Option<(Vec<u8>,)> = sqlx::query_as(
            "SELECT data FROM carve_blocks WHERE metadata_id = ? AND block_id = ?",
        )
        .bind(metadata_id)
        .bind(block_id)
        .fetch_optional(self.pool())
        .await?;

        row.map(|r| r.0).ok_or_else(|| {
            DatastoreError::NotFound {
                entity: "carve block".to_string(),
                id: Some(block_id as u64),
                name: None,
            }
            .into()
        })
    }
}
