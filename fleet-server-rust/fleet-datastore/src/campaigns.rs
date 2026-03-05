//! Campaign datastore operations.
//!
//! Implements distributed query campaign MySQL operations.
//! Matches Go's `server/datastore/mysql/campaigns.go`.

use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::error::{DatastoreError, Result};
use crate::MysqlDatastore;

/// Row type for distributed_query_campaigns table.
#[derive(Debug, Clone, FromRow)]
pub struct CampaignRow {
    pub id: u32,
    pub query_id: u32,
    pub status: u8,
    pub user_id: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl MysqlDatastore {
    /// Creates a new distributed query campaign.
    pub async fn new_distributed_query_campaign(
        &self,
        query_id: u32,
        user_id: u32,
    ) -> Result<CampaignRow> {
        let now = Utc::now();
        let result = sqlx::query(
            "INSERT INTO distributed_query_campaigns (query_id, status, user_id, created_at, updated_at) VALUES (?, ?, ?, ?, ?)",
        )
        .bind(query_id)
        .bind(0u8) // Waiting
        .bind(user_id)
        .bind(now)
        .bind(now)
        .execute(self.pool())
        .await?;

        let id = result.last_insert_id() as u32;

        Ok(CampaignRow {
            id,
            query_id,
            status: 0,
            user_id,
            created_at: now,
            updated_at: now,
        })
    }

    /// Creates a new distributed query campaign target.
    pub async fn new_distributed_query_campaign_target(
        &self,
        campaign_id: u32,
        type_val: i32,
        target_id: u32,
    ) -> Result<()> {
        sqlx::query(
            "INSERT INTO distributed_query_campaign_targets (type, distributed_query_campaign_id, target_id) VALUES (?, ?, ?)",
        )
        .bind(type_val)
        .bind(campaign_id)
        .bind(target_id)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Gets a distributed query campaign by ID.
    pub async fn get_distributed_query_campaign(
        &self,
        id: u32,
    ) -> Result<CampaignRow> {
        sqlx::query_as::<_, CampaignRow>(
            "SELECT id, query_id, status, user_id, created_at, updated_at FROM distributed_query_campaigns WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_id("campaign", id as u64))
    }

    /// Saves (updates) a distributed query campaign status.
    pub async fn save_distributed_query_campaign(
        &self,
        id: u32,
        status: u8,
    ) -> Result<()> {
        let now = Utc::now();
        sqlx::query(
            "UPDATE distributed_query_campaigns SET status = ?, updated_at = ? WHERE id = ?",
        )
        .bind(status)
        .bind(now)
        .bind(id)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Returns the host IDs that match the given targets (hosts, labels, teams).
    pub async fn hosts_ids_for_targets(
        &self,
        host_ids: &[u32],
        label_ids: &[u32],
        team_ids: &[u32],
    ) -> Result<Vec<u32>> {
        let mut result = Vec::new();

        // Direct host IDs
        result.extend_from_slice(host_ids);

        // Hosts from labels
        if !label_ids.is_empty() {
            let placeholders: String = label_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query_str = format!(
                "SELECT DISTINCT host_id FROM label_membership WHERE label_id IN ({})",
                placeholders
            );
            let mut q = sqlx::query_scalar::<_, u32>(&query_str);
            for &label_id in label_ids {
                q = q.bind(label_id);
            }
            let label_hosts = q.fetch_all(self.pool()).await?;
            result.extend(label_hosts);
        }

        // Hosts from teams
        if !team_ids.is_empty() {
            let placeholders: String = team_ids.iter().map(|_| "?").collect::<Vec<_>>().join(",");
            let query_str = format!(
                "SELECT id FROM hosts WHERE team_id IN ({})",
                placeholders
            );
            let mut q = sqlx::query_scalar::<_, u32>(&query_str);
            for &team_id in team_ids {
                q = q.bind(team_id);
            }
            let team_hosts = q.fetch_all(self.pool()).await?;
            result.extend(team_hosts);
        }

        // Deduplicate
        result.sort();
        result.dedup();

        Ok(result)
    }
}
