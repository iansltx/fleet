//! Enroll secret operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/app_configs.go`.

use chrono::{DateTime, Utc};

use crate::error::{is_duplicate, DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for enroll secret queries.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct EnrollSecretRow {
    pub secret: String,
    pub team_id: Option<u32>,
    pub created_at: DateTime<Utc>,
}

impl MysqlDatastore {
    /// Verifies an enrollment secret. Matches Go's `VerifyEnrollSecret`.
    ///
    /// SELECT team_id FROM enroll_secrets WHERE secret = ?
    pub async fn verify_enroll_secret(&self, secret: &str) -> Result<EnrollSecretRow> {
        sqlx::query_as::<_, EnrollSecretRow>(
            "SELECT secret, team_id, created_at FROM enroll_secrets WHERE secret = ?",
        )
        .bind(secret)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found("EnrollSecret"))
    }

    /// Gets enrollment secrets for a team (or global if team_id is None).
    /// Matches Go's `GetEnrollSecrets`.
    ///
    /// SELECT secret, team_id, created_at FROM enroll_secrets
    /// WHERE team_id = ? (or team_id IS NULL)
    /// ORDER BY secret
    pub async fn get_enroll_secrets(
        &self,
        team_id: Option<u32>,
    ) -> Result<Vec<EnrollSecretRow>> {
        let (sql, bind_team_id) = if let Some(tid) = team_id {
            (
                "SELECT secret, team_id, created_at FROM enroll_secrets WHERE team_id = ? ORDER BY secret",
                Some(tid),
            )
        } else {
            (
                "SELECT secret, team_id, created_at FROM enroll_secrets WHERE team_id IS NULL ORDER BY secret",
                None,
            )
        };

        if let Some(tid) = bind_team_id {
            Ok(sqlx::query_as::<_, EnrollSecretRow>(sql)
                .bind(tid)
                .fetch_all(self.pool())
                .await?)
        } else {
            Ok(sqlx::query_as::<_, EnrollSecretRow>(sql)
                .fetch_all(self.pool())
                .await?)
        }
    }

    /// Applies enrollment secrets for a team or globally.
    /// Matches Go's `ApplyEnrollSecrets` / `applyEnrollSecretsDB`.
    ///
    /// This uses the same pattern as Go:
    /// 1. Load existing secrets with created_at timestamps
    /// 2. Delete all existing secrets for the team/global
    /// 3. Re-insert with preserved created_at timestamps
    pub async fn apply_enroll_secrets(
        &self,
        team_id: Option<u32>,
        secrets: &[String],
    ) -> Result<()> {
        let mut tx = self.pool().begin().await?;

        // 1. Load existing secrets to preserve created_at
        let existing = if let Some(tid) = team_id {
            sqlx::query_as::<_, EnrollSecretRow>(
                "SELECT secret, team_id, created_at FROM enroll_secrets WHERE team_id = ?",
            )
            .bind(tid)
            .fetch_all(&mut *tx)
            .await?
        } else {
            sqlx::query_as::<_, EnrollSecretRow>(
                "SELECT secret, team_id, created_at FROM enroll_secrets WHERE team_id IS NULL",
            )
            .fetch_all(&mut *tx)
            .await?
        };

        let mut created_at_map = std::collections::HashMap::new();
        for es in &existing {
            created_at_map.insert(es.secret.clone(), es.created_at);
        }

        // 2. Delete existing secrets
        if let Some(tid) = team_id {
            sqlx::query("DELETE FROM enroll_secrets WHERE team_id = ?")
                .bind(tid)
                .execute(&mut *tx)
                .await?;
        } else {
            sqlx::query("DELETE FROM enroll_secrets WHERE team_id IS NULL")
                .execute(&mut *tx)
                .await?;
        }

        // 3. Re-insert with preserved timestamps
        let default_created_at = Utc::now();
        for secret in secrets {
            let created_at = created_at_map
                .get(secret)
                .copied()
                .unwrap_or(default_created_at);

            let result = sqlx::query(
                "INSERT INTO enroll_secrets (secret, team_id, created_at) VALUES (?, ?, ?)",
            )
            .bind(secret)
            .bind(team_id)
            .bind(created_at)
            .execute(&mut *tx)
            .await;

            if let Err(e) = result {
                if is_duplicate(&e) {
                    return Err(DatastoreError::already_exists("secret", "********"));
                }
                return Err(e.into());
            }
        }

        tx.commit().await?;
        Ok(())
    }

    /// Checks if an enrollment secret is available. Matches Go's `IsEnrollSecretAvailable`.
    ///
    /// SELECT team_id FROM enroll_secrets WHERE secret = ?
    pub async fn is_enroll_secret_available(
        &self,
        secret: &str,
        is_new: bool,
        team_id: Option<u32>,
    ) -> Result<bool> {
        let row: Option<(Option<i64>,)> =
            sqlx::query_as("SELECT team_id FROM enroll_secrets WHERE secret = ?")
                .bind(secret)
                .fetch_optional(self.pool())
                .await?;

        match row {
            None => Ok(true), // Not in use
            Some(_) if is_new => Ok(false), // Already in use, new team can't use it
            Some((secret_team_id,)) => {
                // Check if it's already assigned to this team
                match (team_id, secret_team_id) {
                    (None, None) => Ok(true),
                    (Some(tid), Some(stid)) => Ok(tid as i64 == stid),
                    _ => Ok(false),
                }
            }
        }
    }
}
