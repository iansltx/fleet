//! Team query operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/teams.go`.

use chrono::{DateTime, Utc};

use crate::error::{DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for team queries matching the teams table.
/// Note: config is stored as JSON in the `config` column.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TeamRow {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub name: String,
    pub filename: Option<String>,
    pub description: String,
    pub config: serde_json::Value,
}

/// Row type for team users (from user_teams JOIN users).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct TeamUserRow {
    pub id: u32,
    pub name: String,
    pub email: String,
    pub role: String,
}

impl MysqlDatastore {
    /// Creates a new team. Matches Go's `NewTeam`.
    ///
    /// INSERT INTO teams (name, filename, description, config) VALUES (?, ?, ?, ?)
    pub async fn new_team(
        &self,
        name: &str,
        filename: Option<&str>,
        description: &str,
        config: &serde_json::Value,
    ) -> Result<u32> {
        let result = sqlx::query(
            "INSERT INTO teams (name, filename, description, config) VALUES (?, ?, ?, ?)",
        )
        .bind(name)
        .bind(filename)
        .bind(description)
        .bind(config)
        .execute(self.pool())
        .await?;

        Ok(result.last_insert_id() as u32)
    }

    /// Gets a team by ID. Matches Go's `teamDB`.
    ///
    /// SELECT id, created_at, name, filename, description, config FROM teams WHERE id = ?
    pub async fn team_by_id(&self, id: u32) -> Result<TeamRow> {
        sqlx::query_as::<_, TeamRow>(
            "SELECT id, created_at, name, filename, description, config FROM teams WHERE id = ?",
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_id("Team", id as u64))
    }

    /// Gets a team by name. Matches Go's `TeamByName`.
    ///
    /// SELECT id, created_at, name, filename, description, config FROM teams WHERE name = ?
    pub async fn team_by_name(&self, name: &str) -> Result<TeamRow> {
        sqlx::query_as::<_, TeamRow>(
            "SELECT id, created_at, name, filename, description, config FROM teams WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_name("Team", name))
    }

    /// Lists teams with optional search. Matches Go's `ListTeams`.
    pub async fn list_teams(
        &self,
        match_query: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<TeamRow>> {
        let mut sql =
            "SELECT id, created_at, name, filename, description, config FROM teams WHERE TRUE"
                .to_string();

        if let Some(mq) = match_query {
            if !mq.is_empty() {
                sql.push_str(&format!(" AND name LIKE '%{}%'", mq.replace('\'', "''")));
            }
        }

        sql.push_str(&format!(
            " ORDER BY name ASC LIMIT {} OFFSET {}",
            limit, offset
        ));

        Ok(sqlx::query_as::<_, TeamRow>(&sql)
            .fetch_all(self.pool())
            .await?)
    }

    /// Saves/updates a team. Matches Go's `SaveTeam`.
    ///
    /// UPDATE teams SET name=?, description=?, config=? WHERE id=?
    pub async fn save_team(
        &self,
        id: u32,
        name: &str,
        description: &str,
        config: &serde_json::Value,
    ) -> Result<()> {
        let result = sqlx::query(
            "UPDATE teams SET name = ?, description = ?, config = ? WHERE id = ?",
        )
        .bind(name)
        .bind(description)
        .bind(config)
        .bind(id)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Team", id as u64));
        }
        Ok(())
    }

    /// Deletes a team. Matches Go's `DeleteTeam`.
    ///
    /// DELETE FROM teams WHERE id = ?
    /// Note: In Go, there's a complex transaction that also deletes related records.
    /// This simplified version relies on ON DELETE CASCADE foreign keys.
    pub async fn delete_team(&self, id: u32) -> Result<()> {
        let mut tx = self.pool().begin().await?;

        // Delete team policies first (matching Go)
        sqlx::query("DELETE FROM policies WHERE team_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        // Delete the team
        let result = sqlx::query("DELETE FROM teams WHERE id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Team", id as u64));
        }

        tx.commit().await?;
        Ok(())
    }

    /// Loads users for a team. Matches Go's `loadUsersForTeamDB`.
    ///
    /// SELECT u.id, u.name, u.email, ut.role
    /// FROM user_teams ut JOIN users u ON ut.user_id = u.id
    /// WHERE ut.team_id = ?
    pub async fn load_users_for_team(&self, team_id: u32) -> Result<Vec<TeamUserRow>> {
        Ok(sqlx::query_as::<_, TeamUserRow>(
            r#"
            SELECT u.id, u.name, u.email, ut.role
            FROM user_teams ut JOIN users u ON ut.user_id = u.id
            WHERE ut.team_id = ?
            ORDER BY u.name
            "#,
        )
        .bind(team_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Gets the host count for a team. Matches Go's `loadHostCountForTeamDB`.
    ///
    /// SELECT COUNT(*) FROM hosts WHERE team_id = ?
    pub async fn host_count_for_team(&self, team_id: u32) -> Result<i64> {
        let (count,): (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM hosts WHERE team_id = ?")
                .bind(team_id)
                .fetch_one(self.pool())
                .await?;
        Ok(count)
    }

    /// Loads enroll secrets for teams. Matches Go's `loadSecretsForTeamsDB`.
    pub async fn load_secrets_for_team(
        &self,
        team_id: u32,
    ) -> Result<Vec<EnrollSecretRow>> {
        Ok(sqlx::query_as::<_, EnrollSecretRow>(
            "SELECT secret, team_id, created_at FROM enroll_secrets WHERE team_id = ? ORDER BY secret",
        )
        .bind(team_id)
        .fetch_all(self.pool())
        .await?)
    }
}

/// Row type for enroll secrets.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EnrollSecretRow {
    pub secret: String,
    pub team_id: Option<u32>,
    pub created_at: DateTime<Utc>,
}
