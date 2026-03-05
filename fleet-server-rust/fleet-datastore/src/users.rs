//! User CRUD operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/users.go`.

use chrono::Utc;
use crate::error::{is_duplicate, DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Columns selected for user queries (matches Go's userSelectColumns).
/// Excludes `settings` for security (only included via UserSettings method).
const USER_SELECT_COLUMNS: &str = r#"
    id, created_at, updated_at, password, salt, name, email,
    admin_forced_password_reset, gravatar_url, position, sso_enabled, global_role,
    api_only, mfa_enabled, invite_id
"#;

/// Row type for user queries matching the exact column set.
#[derive(Debug, sqlx::FromRow)]
pub struct UserRow {
    pub id: u32,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub password: Vec<u8>,
    pub salt: String,
    pub name: String,
    pub email: String,
    pub admin_forced_password_reset: bool,
    pub gravatar_url: String,
    pub position: String,
    pub sso_enabled: bool,
    pub global_role: Option<String>,
    pub api_only: bool,
    pub mfa_enabled: bool,
    pub invite_id: Option<u32>,
}

/// Row type for team memberships loaded via loadTeamsForUsers.
#[derive(Debug, sqlx::FromRow)]
pub struct UserTeamRow {
    pub id: u32,
    pub user_id: u32,
    pub role: String,
    pub name: String,
}

/// Parameters for creating a new user, matching Go's fleet.User fields used in INSERT.
pub struct NewUserParams {
    pub password: Vec<u8>,
    pub salt: String,
    pub name: String,
    pub email: String,
    pub admin_forced_password_reset: bool,
    pub gravatar_url: String,
    pub position: String,
    pub sso_enabled: bool,
    pub mfa_enabled: bool,
    pub api_only: bool,
    pub global_role: Option<String>,
    pub invite_id: Option<u32>,
}

/// Parameters for saving/updating a user, matching Go's saveUserDB.
pub struct SaveUserParams {
    pub id: u32,
    pub password: Vec<u8>,
    pub salt: String,
    pub name: String,
    pub email: String,
    pub admin_forced_password_reset: bool,
    pub gravatar_url: String,
    pub position: String,
    pub sso_enabled: bool,
    pub mfa_enabled: bool,
    pub api_only: bool,
    pub global_role: Option<String>,
    pub settings_json: Option<Vec<u8>>,
}

/// Row type for password_reset_requests table.
#[derive(Debug, sqlx::FromRow)]
pub struct PasswordResetRow {
    pub id: u32,
    pub user_id: u32,
    pub token: String,
    pub expires_at: chrono::DateTime<Utc>,
}

impl MysqlDatastore {
    /// Creates a new user. Matches Go's `NewUser`.
    ///
    /// INSERT INTO users (password, salt, name, email, admin_forced_password_reset,
    ///   gravatar_url, position, sso_enabled, mfa_enabled, api_only, global_role, invite_id)
    /// VALUES (?,?,?,?,?,?,?,?,?,?,?,?)
    pub async fn new_user(&self, params: NewUserParams) -> Result<UserRow> {
        let now = Utc::now();

        let result = sqlx::query(
            r#"
            INSERT INTO users (
                password, salt, name, email, admin_forced_password_reset,
                gravatar_url, position, sso_enabled, mfa_enabled, api_only,
                global_role, invite_id
            ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(&params.password)
        .bind(&params.salt)
        .bind(&params.name)
        .bind(&params.email)
        .bind(params.admin_forced_password_reset)
        .bind(&params.gravatar_url)
        .bind(&params.position)
        .bind(params.sso_enabled)
        .bind(params.mfa_enabled)
        .bind(params.api_only)
        .bind(&params.global_role)
        .bind(params.invite_id)
        .execute(self.pool())
        .await;

        match result {
            Ok(res) => {
                let id = res.last_insert_id() as u32;
                Ok(UserRow {
                    id,
                    created_at: now,
                    updated_at: now,
                    password: params.password,
                    salt: params.salt,
                    name: params.name,
                    email: params.email,
                    admin_forced_password_reset: params.admin_forced_password_reset,
                    gravatar_url: params.gravatar_url,
                    position: params.position,
                    sso_enabled: params.sso_enabled,
                    global_role: params.global_role,
                    api_only: params.api_only,
                    mfa_enabled: params.mfa_enabled,
                    invite_id: params.invite_id,
                })
            }
            Err(e) => {
                if is_duplicate(&e) {
                    Err(DatastoreError::already_exists("User", &params.email))
                } else {
                    Err(e.into())
                }
            }
        }
    }

    /// Finds a user by ID. Matches Go's `UserByID` -> `findUser(ctx, "id", id)`.
    ///
    /// SELECT {userSelectColumns} FROM users WHERE id = ? LIMIT 1
    pub async fn user_by_id(&self, id: u32) -> Result<UserRow> {
        let query = format!(
            "SELECT {} FROM users WHERE id = ? LIMIT 1",
            USER_SELECT_COLUMNS
        );

        sqlx::query_as::<_, UserRow>(&query)
            .bind(id)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_id("User", id as u64))
    }

    /// Finds a user by email. Matches Go's `UserByEmail` -> `findUser(ctx, "email", email)`.
    ///
    /// SELECT {userSelectColumns} FROM users WHERE email = ? LIMIT 1
    pub async fn user_by_email(&self, email: &str) -> Result<UserRow> {
        let query = format!(
            "SELECT {} FROM users WHERE email = ? LIMIT 1",
            USER_SELECT_COLUMNS
        );

        sqlx::query_as::<_, UserRow>(&query)
            .bind(email)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_name("User", email))
    }

    /// Updates an existing user. Matches Go's `saveUserDB`.
    ///
    /// UPDATE users SET password=?, salt=?, name=?, email=?,
    ///   admin_forced_password_reset=?, gravatar_url=?, position=?,
    ///   sso_enabled=?, mfa_enabled=?, api_only=?, settings=?, global_role=?
    /// WHERE id = ?
    pub async fn save_user(&self, params: SaveUserParams) -> Result<()> {
        let result = sqlx::query(
            r#"
            UPDATE users SET
                password = ?,
                salt = ?,
                name = ?,
                email = ?,
                admin_forced_password_reset = ?,
                gravatar_url = ?,
                position = ?,
                sso_enabled = ?,
                mfa_enabled = ?,
                api_only = ?,
                settings = ?,
                global_role = ?
            WHERE id = ?
            "#,
        )
        .bind(&params.password)
        .bind(&params.salt)
        .bind(&params.name)
        .bind(&params.email)
        .bind(params.admin_forced_password_reset)
        .bind(&params.gravatar_url)
        .bind(&params.position)
        .bind(params.sso_enabled)
        .bind(params.mfa_enabled)
        .bind(params.api_only)
        .bind(&params.settings_json)
        .bind(&params.global_role)
        .bind(params.id)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("User", params.id as u64));
        }

        Ok(())
    }

    /// Lists users with optional team filter and search. Matches Go's `ListUsers`.
    ///
    /// SELECT * FROM users WHERE TRUE
    ///   [AND id IN (SELECT user_id FROM user_teams WHERE team_id = ?)]
    ///   [AND (name LIKE ? OR email LIKE ?)]
    ///   ORDER BY ... LIMIT ... OFFSET ...
    pub async fn list_users(
        &self,
        team_id: Option<u32>,
        match_query: Option<&str>,
        order_key: &str,
        order_desc: bool,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<UserRow>> {
        let mut sql = "SELECT * FROM users WHERE TRUE".to_string();
        let mut args: Vec<Box<dyn sqlx::Encode<'_, sqlx::MySql> + Send + Sync>> = Vec::new();

        if let Some(tid) = team_id {
            sql.push_str(" AND id IN (SELECT user_id FROM user_teams WHERE team_id = ?)");
            args.push(Box::new(tid));
        }

        if let Some(mq) = match_query {
            if !mq.is_empty() {
                let like_val = format!("%{}%", mq);
                sql.push_str(" AND (name LIKE ? OR email LIKE ?)");
                args.push(Box::new(like_val.clone()));
                args.push(Box::new(like_val));
            }
        }

        // Validate order key against allowlist (matching Go's userAllowedOrderKeys)
        let order_col = match order_key {
            "name" => "name",
            "email" => "email",
            "created_at" => "created_at",
            "updated_at" => "updated_at",
            "id" | "" => "id",
            _ => "id",
        };

        let direction = if order_desc { "DESC" } else { "ASC" };
        sql.push_str(&format!(" ORDER BY {} {}", order_col, direction));

        if limit > 0 {
            sql.push_str(&format!(" LIMIT {} OFFSET {}", limit, offset));
        }

        // Use raw query building since we have dynamic args
        let users = sqlx::query_as::<_, UserRow>(&sql)
            .fetch_all(self.pool())
            .await?;

        Ok(users)
    }

    /// Deletes a user. Matches Go's `DeleteUser`.
    ///
    /// First archives the user to users_deleted, then deletes from users.
    pub async fn delete_user(&self, id: u32) -> Result<()> {
        // Archive to users_deleted (matching Go)
        sqlx::query(
            r#"
            INSERT INTO users_deleted (id, name, email)
            SELECT u.id, u.name, u.email
            FROM users AS u
            WHERE u.id = ?
            ON DUPLICATE KEY UPDATE
                name = u.name,
                email = u.email
            "#,
        )
        .bind(id)
        .execute(self.pool())
        .await?;

        // Delete from users table
        let result = sqlx::query("DELETE FROM users WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("User", id as u64));
        }

        Ok(())
    }

    /// Loads team memberships for a set of user IDs. Matches Go's `loadTeamsForUsers`.
    ///
    /// SELECT ut.team_id AS id, ut.user_id, ut.role, t.name
    /// FROM user_teams ut INNER JOIN teams t ON ut.team_id = t.id
    /// WHERE ut.user_id IN (?)
    /// ORDER BY user_id, team_id
    pub async fn load_teams_for_users(&self, user_ids: &[u32]) -> Result<Vec<UserTeamRow>> {
        if user_ids.is_empty() {
            return Ok(Vec::new());
        }

        // Build IN clause
        let placeholders: Vec<&str> = user_ids.iter().map(|_| "?").collect();
        let sql = format!(
            r#"
            SELECT ut.team_id AS id, ut.user_id, ut.role, t.name
            FROM user_teams ut INNER JOIN teams t ON ut.team_id = t.id
            WHERE ut.user_id IN ({})
            ORDER BY user_id, team_id
            "#,
            placeholders.join(",")
        );

        let mut query = sqlx::query_as::<_, UserTeamRow>(&sql);
        for uid in user_ids {
            query = query.bind(uid);
        }

        Ok(query.fetch_all(self.pool()).await?)
    }

    /// Checks if there are any users registered. Matches Go's `HasUsers`.
    ///
    /// SELECT id FROM users LIMIT 1
    pub async fn has_users(&self) -> Result<bool> {
        let row = sqlx::query("SELECT id FROM users LIMIT 1")
            .fetch_optional(self.pool())
            .await?;

        Ok(row.is_some())
    }

    /// Counts global admins. Matches Go's `CountGlobalAdmins`.
    ///
    /// SELECT COUNT(*) FROM users WHERE global_role = 'admin'
    pub async fn count_global_admins(&self) -> Result<i64> {
        let row: (i64,) =
            sqlx::query_as("SELECT COUNT(*) FROM users WHERE global_role = 'admin'")
                .fetch_one(self.pool())
                .await?;
        Ok(row.0)
    }

    /// Gets user settings. Matches Go's `UserSettings`.
    ///
    /// SELECT settings FROM users WHERE id = ?
    pub async fn user_settings(&self, user_id: u32) -> Result<Option<Vec<u8>>> {
        let row: Option<(Option<Vec<u8>>,)> =
            sqlx::query_as("SELECT settings FROM users WHERE id = ?")
                .bind(user_id)
                .fetch_optional(self.pool())
                .await?;

        match row {
            Some((settings,)) => Ok(settings),
            None => Err(DatastoreError::not_found_with_id(
                "UserSettings",
                user_id as u64,
            )),
        }
    }

    /// Saves team memberships for a user. Matches Go's `saveTeamsForUserDB`.
    /// Deletes existing teams then bulk-inserts new ones.
    pub async fn save_teams_for_user(
        &self,
        user_id: u32,
        teams: &[(u32, String)], // (team_id, role)
    ) -> Result<()> {
        // Delete existing
        sqlx::query("DELETE FROM user_teams WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await?;

        if teams.is_empty() {
            return Ok(());
        }

        // Bulk insert
        let placeholders: Vec<String> = teams.iter().map(|_| "(?,?,?)".to_string()).collect();
        let sql = format!(
            "INSERT INTO user_teams (user_id, team_id, role) VALUES {}",
            placeholders.join(",")
        );

        let mut query = sqlx::query(&sql);
        for (team_id, role) in teams {
            query = query.bind(user_id).bind(team_id).bind(role);
        }

        query.execute(self.pool()).await?;
        Ok(())
    }

    /// Creates a new password reset request. Matches Go's `NewPasswordResetRequest`.
    pub async fn new_password_reset_request(
        &self,
        user_id: u32,
        expires_at: chrono::DateTime<Utc>,
        token: &str,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO password_reset_requests (user_id, token, expires_at)
            VALUES (?, ?, ?)
            ON DUPLICATE KEY UPDATE token = VALUES(token), expires_at = VALUES(expires_at)
            "#,
        )
        .bind(user_id)
        .bind(token)
        .bind(expires_at)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    /// Finds a password reset request by token. Matches Go's `FindPassResetByToken`.
    pub async fn find_password_reset_by_token(&self, token: &str) -> Result<PasswordResetRow> {
        sqlx::query_as::<_, PasswordResetRow>(
            "SELECT id, user_id, token, expires_at FROM password_reset_requests WHERE token = ?",
        )
        .bind(token)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_name("PasswordResetRequest", token))
    }

    /// Deletes all password reset requests for a user.
    pub async fn delete_password_reset_requests_for_user(&self, user_id: u32) -> Result<()> {
        sqlx::query("DELETE FROM password_reset_requests WHERE user_id = ?")
            .bind(user_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }
}
