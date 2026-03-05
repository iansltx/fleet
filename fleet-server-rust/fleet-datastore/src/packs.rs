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

/// Row type for scheduled queries in a pack.
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
