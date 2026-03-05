//! Pack CRUD operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/packs.go`.

use chrono::{DateTime, Utc};

use crate::error::{DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for pack queries matching the packs table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PackRow {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: Option<String>,
    pub platform: Option<String>,
    pub disabled: bool,
    pub pack_type: Option<String>,
}

/// Row type for pack target queries.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct PackTargetRow {
    #[sqlx(rename = "type")]
    pub target_type: u32,
    pub target_id: u32,
    pub display_text: String,
}

/// Row type for scheduled queries in a pack (spec/apply format).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ScheduledQueryRow {
    pub query_name: String,
    pub name: String,
    pub description: String,
    pub interval: u32,
    pub snapshot: Option<bool>,
    pub removed: Option<bool>,
    pub shard: Option<u32>,
    pub platform: Option<String>,
    pub version: Option<String>,
    pub denylist: Option<bool>,
}

/// Full row type for scheduled queries with all fields.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ScheduledQueryFullRow {
    pub id: u32,
    pub pack_id: u32,
    pub query_id: u32,
    pub name: String,
    pub description: String,
    pub interval: u32,
    pub snapshot: Option<bool>,
    pub removed: Option<bool>,
    pub shard: Option<u32>,
    pub platform: Option<String>,
    pub version: Option<String>,
    pub denylist: Option<bool>,
    pub query_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MysqlDatastore {
    /// Creates a new pack. Matches Go's `NewPack`.
    ///
    /// INSERT INTO packs (name, description, platform, disabled) VALUES (?, ?, ?, ?)
    pub async fn new_pack(
        &self,
        name: &str,
        description: &str,
        platform: &str,
        disabled: bool,
    ) -> Result<u32> {
        let result = sqlx::query(
            "INSERT INTO packs (name, description, platform, disabled) VALUES (?, ?, ?, ?)",
        )
        .bind(name)
        .bind(description)
        .bind(platform)
        .bind(disabled)
        .execute(self.pool())
        .await?;

        Ok(result.last_insert_id() as u32)
    }

    /// Gets a pack by ID. Matches Go's `Pack`.
    ///
    /// SELECT * FROM packs WHERE id = ?
    pub async fn pack_by_id(&self, id: u32) -> Result<PackRow> {
        sqlx::query_as::<_, PackRow>("SELECT * FROM packs WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_id("Pack", id as u64))
    }

    /// Gets a pack by name. Matches Go's `PackByName`.
    ///
    /// SELECT * FROM packs WHERE name = ?
    pub async fn pack_by_name(&self, name: &str) -> Result<Option<PackRow>> {
        Ok(sqlx::query_as::<_, PackRow>("SELECT * FROM packs WHERE name = ?")
            .bind(name)
            .fetch_optional(self.pool())
            .await?)
    }

    /// Updates a pack. Matches Go's `SavePack`.
    ///
    /// UPDATE packs SET name = ?, platform = ?, disabled = ?, description = ? WHERE id = ?
    pub async fn save_pack(
        &self,
        id: u32,
        name: &str,
        platform: &str,
        disabled: bool,
        description: &str,
    ) -> Result<()> {
        let result = sqlx::query(
            "UPDATE packs SET name = ?, platform = ?, disabled = ?, description = ? WHERE id = ?",
        )
        .bind(name)
        .bind(platform)
        .bind(disabled)
        .bind(description)
        .bind(id)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Pack", id as u64));
        }
        Ok(())
    }

    /// Deletes a pack by name. Matches Go's `DeletePack`.
    ///
    /// DELETE FROM packs WHERE name = ?
    pub async fn delete_pack(&self, name: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM packs WHERE name = ?")
            .bind(name)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_name("Pack", name));
        }
        Ok(())
    }

    /// Lists packs. Matches Go's `ListPacks`.
    ///
    /// SELECT * FROM packs WHERE pack_type IS NULL OR pack_type = ''
    pub async fn list_packs(&self, include_system_packs: bool) -> Result<Vec<PackRow>> {
        let sql = if include_system_packs {
            "SELECT * FROM packs"
        } else {
            "SELECT * FROM packs WHERE pack_type IS NULL OR pack_type = ''"
        };

        Ok(sqlx::query_as::<_, PackRow>(sql)
            .fetch_all(self.pool())
            .await?)
    }

    /// Loads pack targets. Matches Go's `loadPackTargetsDB`.
    ///
    /// Uses the same COALESCE/CASE pattern from the Go code:
    /// SELECT type, target_id, COALESCE(CASE WHEN type = ? THEN ... END, '') AS display_text
    /// FROM pack_targets WHERE pack_id = ?
    pub async fn load_pack_targets(&self, pack_id: u32) -> Result<Vec<PackTargetRow>> {
        // TargetHost = 2, TargetTeam = 3, TargetLabel = 1 (matching Go constants)
        Ok(sqlx::query_as::<_, PackTargetRow>(
            r#"
            SELECT type, target_id,
                COALESCE(
                    CASE
                        WHEN type = 2 THEN (SELECT hostname FROM hosts WHERE id = target_id)
                        WHEN type = 3 THEN (SELECT name FROM teams WHERE id = target_id)
                        WHEN type = 1 THEN (SELECT name FROM labels WHERE id = target_id)
                    END
                , '') AS display_text
            FROM pack_targets
            WHERE pack_id = ?
            "#,
        )
        .bind(pack_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Lists packs for a specific host. Matches Go's `ListPacksForHost`.
    ///
    /// Uses the same UNION ALL pattern from Go to find packs via labels, hosts, and teams.
    pub async fn list_packs_for_host(&self, host_id: u32) -> Result<Vec<PackRow>> {
        // TargetLabel = 1, TargetHost = 2, TargetTeam = 3
        Ok(sqlx::query_as::<_, PackRow>(
            r#"
            SELECT DISTINCT packs.* FROM (
            (
                SELECT p.* FROM packs p
                JOIN pack_targets pt
                JOIN label_membership lm
                ON (p.id = pt.pack_id AND pt.target_id = lm.label_id AND pt.type = ?)
                WHERE lm.host_id = ? AND NOT p.disabled AND p.pack_type IS NULL
            )
            UNION ALL
            (
                SELECT p.* FROM packs p
                JOIN pack_targets pt ON (p.id = pt.pack_id AND pt.type = ? AND pt.target_id = ?)
                WHERE p.pack_type IS NULL
            )
            UNION ALL
            (
                SELECT p.*
                FROM packs p
                JOIN pack_targets pt
                ON (p.id = pt.pack_id AND pt.type = ? AND pt.target_id = (SELECT team_id FROM hosts WHERE id = ?))
                WHERE p.pack_type IS NULL
            )) packs
            "#,
        )
        .bind(1u32) // TargetLabel
        .bind(host_id)
        .bind(2u32) // TargetHost
        .bind(host_id)
        .bind(3u32) // TargetTeam
        .bind(host_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Gets a pack by its pack_type.
    pub async fn pack_by_type(&self, pack_type: &str) -> Result<Option<PackRow>> {
        Ok(sqlx::query_as::<_, PackRow>("SELECT * FROM packs WHERE pack_type = ?")
            .bind(pack_type)
            .fetch_optional(self.pool())
            .await?)
    }

    /// Ensures a pack with the given type exists, creating it if needed.
    pub async fn ensure_pack_by_type(&self, pack_type: &str, name: &str) -> Result<u32> {
        if let Some(row) = self.pack_by_type(pack_type).await? {
            return Ok(row.id);
        }
        let result = sqlx::query(
            "INSERT INTO packs (name, description, platform, disabled, pack_type) VALUES (?, '', '', 0, ?)",
        )
        .bind(name)
        .bind(pack_type)
        .execute(self.pool())
        .await?;
        Ok(result.last_insert_id() as u32)
    }

    /// Lists scheduled queries in a pack by pack_id.
    pub async fn list_scheduled_queries_in_pack_full(
        &self,
        pack_id: u32,
    ) -> Result<Vec<ScheduledQueryFullRow>> {
        Ok(sqlx::query_as::<_, ScheduledQueryFullRow>(
            r#"
            SELECT id, pack_id, query_id, name, description,
                   `interval`, snapshot, removed, shard,
                   platform, version, denylist, query_name,
                   created_at, updated_at
            FROM scheduled_queries
            WHERE pack_id = ?
            ORDER BY id
            "#,
        )
        .bind(pack_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Gets a single scheduled query by ID.
    pub async fn scheduled_query_by_id(&self, id: u32) -> Result<ScheduledQueryFullRow> {
        sqlx::query_as::<_, ScheduledQueryFullRow>(
            r#"
            SELECT id, pack_id, query_id, name, description,
                   `interval`, snapshot, removed, shard,
                   platform, version, denylist, query_name,
                   created_at, updated_at
            FROM scheduled_queries
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_id("ScheduledQuery", id as u64))
    }

    /// Inserts a new scheduled query.
    pub async fn insert_scheduled_query(
        &self,
        pack_id: u32,
        query_id: u32,
        query_name: &str,
        name: &str,
        description: &str,
        interval: u32,
        snapshot: Option<bool>,
        removed: Option<bool>,
        platform: &str,
        version: &str,
        shard: Option<u32>,
    ) -> Result<u32> {
        let result = sqlx::query(
            r#"
            INSERT INTO scheduled_queries
                (pack_id, query_id, query_name, name, description, `interval`, snapshot, removed, platform, version, shard)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(pack_id)
        .bind(query_id)
        .bind(query_name)
        .bind(name)
        .bind(description)
        .bind(interval)
        .bind(snapshot)
        .bind(removed)
        .bind(platform)
        .bind(version)
        .bind(shard)
        .execute(self.pool())
        .await?;
        Ok(result.last_insert_id() as u32)
    }

    /// Updates a scheduled query.
    pub async fn update_scheduled_query(
        &self,
        id: u32,
        interval: u32,
        snapshot: Option<bool>,
        removed: Option<bool>,
        platform: &str,
        version: &str,
        shard: Option<u32>,
    ) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE scheduled_queries
            SET `interval` = ?, snapshot = ?, removed = ?, platform = ?, version = ?, shard = ?
            WHERE id = ?
            "#,
        )
        .bind(interval)
        .bind(snapshot)
        .bind(removed)
        .bind(platform)
        .bind(version)
        .bind(shard)
        .bind(id)
        .execute(self.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("ScheduledQuery", id as u64));
        }
        Ok(())
    }

    /// Deletes a scheduled query by ID.
    pub async fn remove_scheduled_query(&self, id: u32) -> Result<()> {
        let result = sqlx::query("DELETE FROM scheduled_queries WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("ScheduledQuery", id as u64));
        }
        Ok(())
    }

    /// Applies pack specs (from YAML). Matches Go's `ApplyPackSpecs`.
    pub async fn apply_pack_spec(
        &self,
        name: &str,
        description: &str,
        platform: &str,
        disabled: bool,
        queries: &[ScheduledQueryRow],
    ) -> Result<u32> {
        // Upsert pack
        sqlx::query(
            r#"
            INSERT INTO packs (name, description, platform, disabled)
            VALUES (?, ?, ?, ?)
            ON DUPLICATE KEY UPDATE
                name = VALUES(name),
                description = VALUES(description),
                platform = VALUES(platform),
                disabled = VALUES(disabled)
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(platform)
        .bind(disabled)
        .execute(self.pool())
        .await?;

        // Get pack ID
        let (pack_id,): (u32,) = sqlx::query_as("SELECT id FROM packs WHERE name = ?")
            .bind(name)
            .fetch_one(self.pool())
            .await?;

        // Delete existing scheduled queries
        sqlx::query("DELETE FROM scheduled_queries WHERE pack_id = ?")
            .bind(pack_id)
            .execute(self.pool())
            .await?;

        // Insert new scheduled queries
        for q in queries {
            let q_name = if q.name.is_empty() {
                &q.query_name
            } else {
                &q.name
            };
            sqlx::query(
                r#"
                INSERT INTO scheduled_queries (
                    pack_id, query_name, name, description, `interval`,
                    snapshot, removed, shard, platform, version, denylist
                )
                VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
                "#,
            )
            .bind(pack_id)
            .bind(&q.query_name)
            .bind(q_name)
            .bind(&q.description)
            .bind(q.interval)
            .bind(q.snapshot)
            .bind(q.removed)
            .bind(q.shard)
            .bind(&q.platform)
            .bind(&q.version)
            .bind(q.denylist)
            .execute(self.pool())
            .await?;
        }

        Ok(pack_id)
    }
}
