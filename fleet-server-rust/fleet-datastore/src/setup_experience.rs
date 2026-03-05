//! Setup experience query operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/setup_experience.go`.

use chrono::{DateTime, Utc};

use crate::error::{DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for setup_experience_scripts table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SetupExperienceScriptRow {
    pub id: u32,
    pub team_id: Option<u32>,
    pub global_or_team_id: u32,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    #[sqlx(default)]
    pub script_content_id: Option<u32>,
}

impl MysqlDatastore {
    /// Gets the setup experience script for a team (or global if team_id is None).
    ///
    /// SELECT * FROM setup_experience_scripts WHERE global_or_team_id = ?
    pub async fn get_setup_experience_script(
        &self,
        team_id: Option<u32>,
    ) -> Result<SetupExperienceScriptRow> {
        let global_or_team_id = team_id.unwrap_or(0);
        sqlx::query_as::<_, SetupExperienceScriptRow>(
            r#"
            SELECT id, team_id, global_or_team_id, name, created_at, updated_at, script_content_id
            FROM setup_experience_scripts
            WHERE global_or_team_id = ?
            "#,
        )
        .bind(global_or_team_id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found("SetupExperienceScript"))
    }

    /// Deletes the setup experience script for a team (or global if team_id is None).
    ///
    /// DELETE FROM setup_experience_scripts WHERE global_or_team_id = ?
    pub async fn delete_setup_experience_script(
        &self,
        team_id: Option<u32>,
    ) -> Result<()> {
        let global_or_team_id = team_id.unwrap_or(0);
        sqlx::query("DELETE FROM setup_experience_scripts WHERE global_or_team_id = ?")
            .bind(global_or_team_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    /// Lists software title IDs configured for setup experience for a team.
    ///
    /// Matches Go's ListSetupExperienceSoftwareTitles (simplified).
    pub async fn list_setup_experience_software_title_ids(
        &self,
        team_id: Option<u32>,
    ) -> Result<Vec<u32>> {
        let global_or_team_id = team_id.unwrap_or(0);
        let rows: Vec<(u32,)> = sqlx::query_as(
            r#"
            SELECT si.title_id
            FROM software_installers si
            WHERE si.global_or_team_id = ? AND si.install_during_setup = 1
            "#,
        )
        .bind(global_or_team_id)
        .fetch_all(self.pool())
        .await?;
        Ok(rows.into_iter().map(|(id,)| id).collect())
    }

    /// Sets which software titles should be installed during setup experience.
    ///
    /// UPDATE software_installers SET install_during_setup = ...
    pub async fn set_setup_experience_software(
        &self,
        team_id: Option<u32>,
        title_ids: &[u32],
    ) -> Result<()> {
        let global_or_team_id = team_id.unwrap_or(0);
        // Clear all first
        sqlx::query(
            "UPDATE software_installers SET install_during_setup = 0 WHERE global_or_team_id = ?",
        )
        .bind(global_or_team_id)
        .execute(self.pool())
        .await?;

        // Set the specified ones
        if !title_ids.is_empty() {
            let placeholders: Vec<&str> = title_ids.iter().map(|_| "?").collect();
            let sql = format!(
                "UPDATE software_installers SET install_during_setup = 1 WHERE global_or_team_id = ? AND title_id IN ({})",
                placeholders.join(",")
            );
            let mut query = sqlx::query(&sql).bind(global_or_team_id);
            for id in title_ids {
                query = query.bind(id);
            }
            query.execute(self.pool()).await?;
        }
        Ok(())
    }
}
