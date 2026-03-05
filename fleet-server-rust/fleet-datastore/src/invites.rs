//! Invite CRUD operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/invites.go`.

use chrono::{DateTime, Utc};

use crate::error::{is_duplicate, DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for invite queries matching the invites table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InviteRow {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub invited_by: u32,
    pub email: String,
    pub name: String,
    pub position: String,
    pub token: String,
    pub sso_enabled: bool,
    pub mfa_enabled: bool,
    pub global_role: Option<String>,
}

/// Row type for invite team memberships.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct InviteTeamRow {
    pub id: u32,
    pub invite_id: u32,
    pub role: String,
    pub name: String,
}

impl MysqlDatastore {
    /// Creates a new invite. Matches Go's `NewInvite`.
    ///
    /// INSERT INTO invites (invited_by, email, name, position, token, sso_enabled, mfa_enabled, global_role)
    /// VALUES (?, ?, ?, ?, ?, ?, ?, ?)
    pub async fn new_invite(
        &self,
        invited_by: u32,
        email: &str,
        name: &str,
        position: &str,
        token: &str,
        sso_enabled: bool,
        mfa_enabled: bool,
        global_role: Option<&str>,
        teams: &[(u32, String)], // (team_id, role)
    ) -> Result<u32> {
        let mut tx = self.pool().begin().await?;

        let result = sqlx::query(
            r#"
            INSERT INTO invites (invited_by, email, name, position, token, sso_enabled, mfa_enabled, global_role)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(invited_by)
        .bind(email)
        .bind(name)
        .bind(position)
        .bind(token)
        .bind(sso_enabled)
        .bind(mfa_enabled)
        .bind(global_role)
        .execute(&mut *tx)
        .await;

        let invite_id = match result {
            Ok(res) => res.last_insert_id() as u32,
            Err(e) => {
                if is_duplicate(&e) {
                    return Err(DatastoreError::already_exists("Invite", email));
                }
                return Err(e.into());
            }
        };

        // Bulk insert teams (matching Go)
        if !teams.is_empty() {
            let placeholders: Vec<String> = teams.iter().map(|_| "(?,?,?)".to_string()).collect();
            let sql = format!(
                "INSERT INTO invite_teams (invite_id, team_id, role) VALUES {}",
                placeholders.join(",")
            );

            let mut query = sqlx::query(&sql);
            for (team_id, role) in teams {
                query = query.bind(invite_id).bind(team_id).bind(role);
            }
            query.execute(&mut *tx).await?;
        }

        tx.commit().await?;
        Ok(invite_id)
    }

    /// Lists invites with optional search. Matches Go's `ListInvites`.
    ///
    /// SELECT * FROM invites WHERE true
    /// [AND (name LIKE ? OR email LIKE ?)]
    pub async fn list_invites(&self, match_query: Option<&str>) -> Result<Vec<InviteRow>> {
        let mut sql = "SELECT * FROM invites WHERE true".to_string();

        if let Some(mq) = match_query {
            if !mq.is_empty() {
                let like_val = format!("%{}%", mq.replace('\'', "''"));
                sql.push_str(&format!(
                    " AND (name LIKE '{}' OR email LIKE '{}')",
                    like_val, like_val
                ));
            }
        }

        Ok(sqlx::query_as::<_, InviteRow>(&sql)
            .fetch_all(self.pool())
            .await?)
    }

    /// Gets an invite by ID. Matches Go's `Invite`.
    ///
    /// SELECT * FROM invites WHERE id = ?
    pub async fn invite_by_id(&self, id: u32) -> Result<InviteRow> {
        sqlx::query_as::<_, InviteRow>("SELECT * FROM invites WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_id("Invite", id as u64))
    }

    /// Gets an invite by email. Matches Go's `InviteByEmail`.
    ///
    /// SELECT * FROM invites WHERE email = ?
    pub async fn invite_by_email(&self, email: &str) -> Result<InviteRow> {
        sqlx::query_as::<_, InviteRow>("SELECT * FROM invites WHERE email = ?")
            .bind(email)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_name("Invite", email))
    }

    /// Gets an invite by token. Matches Go's `InviteByToken`.
    ///
    /// SELECT * FROM invites WHERE token = ?
    pub async fn invite_by_token(&self, token: &str) -> Result<InviteRow> {
        sqlx::query_as::<_, InviteRow>("SELECT * FROM invites WHERE token = ?")
            .bind(token)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found("Invite"))
    }

    /// Deletes an invite by ID. Matches Go's `DeleteInvite`.
    ///
    /// DELETE FROM invites WHERE id = ?
    pub async fn delete_invite(&self, id: u32) -> Result<()> {
        let result = sqlx::query("DELETE FROM invites WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Invite", id as u64));
        }
        Ok(())
    }

    /// Updates an invite. Matches Go's `UpdateInvite`.
    ///
    /// UPDATE invites SET invited_by=?, email=?, name=?, position=?, sso_enabled=?, mfa_enabled=?, global_role=?
    /// WHERE id = ?
    pub async fn update_invite(
        &self,
        id: u32,
        invited_by: u32,
        email: &str,
        name: &str,
        position: &str,
        sso_enabled: bool,
        mfa_enabled: bool,
        global_role: Option<&str>,
        teams: &[(u32, String)], // (team_id, role)
    ) -> Result<()> {
        let mut tx = self.pool().begin().await?;

        sqlx::query(
            r#"
            UPDATE invites SET invited_by = ?, email = ?, name = ?, position = ?,
                sso_enabled = ?, mfa_enabled = ?, global_role = ?
            WHERE id = ?
            "#,
        )
        .bind(invited_by)
        .bind(email)
        .bind(name)
        .bind(position)
        .bind(sso_enabled)
        .bind(mfa_enabled)
        .bind(global_role)
        .bind(id)
        .execute(&mut *tx)
        .await?;

        // Replace teams
        sqlx::query("DELETE FROM invite_teams WHERE invite_id = ?")
            .bind(id)
            .execute(&mut *tx)
            .await?;

        for (team_id, role) in teams {
            sqlx::query("INSERT INTO invite_teams(invite_id, team_id, role) VALUES(?, ?, ?)")
                .bind(id)
                .bind(team_id)
                .bind(role)
                .execute(&mut *tx)
                .await?;
        }

        tx.commit().await?;
        Ok(())
    }

    /// Loads team memberships for invites. Matches Go's `loadTeamsForInvites`.
    pub async fn load_teams_for_invites(&self, invite_ids: &[u32]) -> Result<Vec<InviteTeamRow>> {
        if invite_ids.is_empty() {
            return Ok(Vec::new());
        }

        let placeholders: Vec<&str> = invite_ids.iter().map(|_| "?").collect();
        let sql = format!(
            r#"
            SELECT ut.team_id AS id, ut.invite_id, ut.role, t.name
            FROM invite_teams ut INNER JOIN teams t ON ut.team_id = t.id
            WHERE ut.invite_id IN ({})
            ORDER BY invite_id, team_id
            "#,
            placeholders.join(",")
        );

        let mut query = sqlx::query_as::<_, InviteTeamRow>(&sql);
        for id in invite_ids {
            query = query.bind(id);
        }

        Ok(query.fetch_all(self.pool()).await?)
    }
}
