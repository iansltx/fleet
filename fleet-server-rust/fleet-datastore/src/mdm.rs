//! MDM datastore operations.
//!
//! SQL queries for MDM configuration profiles, commands, and summaries.
//! Matches Go code in `server/datastore/mysql/mdm.go` and `server/datastore/mysql/apple_mdm.go`.

use chrono::{DateTime, Utc};

use crate::error::{DatastoreError, Result};
use crate::mysql::MysqlDatastore;

// ---------------------------------------------------------------------------
// Row types
// ---------------------------------------------------------------------------

/// Row type for combined MDM config profile listing.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MDMConfigProfileRow {
    pub profile_uuid: String,
    #[sqlx(default)]
    pub team_id: Option<u32>,
    pub name: String,
    pub platform: String,
    #[sqlx(default)]
    pub identifier: Option<String>,
    #[sqlx(default)]
    pub scope: Option<String>,
    #[sqlx(default)]
    pub checksum: Option<String>,
    pub created_at: DateTime<Utc>,
    pub uploaded_at: DateTime<Utc>,
}

/// Row type for profile label associations.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProfileLabelRow {
    pub profile_uuid: String,
    pub label_name: String,
    #[sqlx(default)]
    pub label_id: Option<u32>,
    #[sqlx(default)]
    pub broken: Option<bool>,
    #[sqlx(default)]
    pub exclude: bool,
    #[sqlx(default)]
    pub require_all: bool,
}

/// Row type for MDM profiles summary counts.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MDMProfilesSummaryRow {
    pub status: String,
    pub count: u32,
}

/// Row type for MDM commands.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MDMCommandRow {
    pub host_uuid: String,
    pub command_uuid: String,
    pub updated_at: DateTime<Utc>,
    pub request_type: String,
    pub status: String,
    pub hostname: String,
    #[sqlx(default)]
    pub team_id: Option<u32>,
}

/// Row type for MDM command results.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct MDMCommandResultRow {
    pub host_uuid: String,
    pub command_uuid: String,
    pub status: String,
    pub updated_at: DateTime<Utc>,
    pub request_type: String,
    #[sqlx(default)]
    pub result: Vec<u8>,
    #[sqlx(default)]
    pub payload: Vec<u8>,
}

/// Row type for host MDM profile status.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HostMDMProfileRow {
    pub profile_uuid: String,
    pub name: String,
    pub status: String,
    pub operation_type: String,
    pub detail: String,
    #[sqlx(default)]
    pub scope: Option<String>,
    #[sqlx(default)]
    pub managed_local_account: Option<String>,
}

/// Row type for profile status counts.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ProfileStatusCountRow {
    pub status: String,
    pub count: u32,
}

// ---------------------------------------------------------------------------
// Implementations
// ---------------------------------------------------------------------------

impl MysqlDatastore {
    /// Lists MDM configuration profiles for a team.
    /// Combines Apple profiles, Windows profiles, Apple declarations, and Android profiles.
    /// Simplified version of Go's `ListMDMConfigProfiles`.
    pub async fn list_mdm_config_profiles(
        &self,
        team_id: Option<u32>,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<MDMConfigProfileRow>> {
        let limit = if per_page == 0 { 20 } else { per_page };
        let offset = page * limit;
        let tid = team_id.unwrap_or(0);

        Ok(sqlx::query_as::<_, MDMConfigProfileRow>(
            r#"
            SELECT * FROM (
                SELECT
                    profile_uuid,
                    team_id,
                    name,
                    'darwin' AS platform,
                    identifier,
                    scope,
                    HEX(checksum) AS checksum,
                    created_at,
                    uploaded_at
                FROM mdm_apple_configuration_profiles
                WHERE team_id = ?

                UNION ALL

                SELECT
                    profile_uuid,
                    team_id,
                    name,
                    'windows' AS platform,
                    NULL AS identifier,
                    NULL AS scope,
                    NULL AS checksum,
                    created_at,
                    uploaded_at
                FROM mdm_windows_configuration_profiles
                WHERE team_id = ?

                UNION ALL

                SELECT
                    declaration_uuid AS profile_uuid,
                    team_id,
                    name,
                    'darwin' AS platform,
                    identifier,
                    scope,
                    token AS checksum,
                    created_at,
                    uploaded_at
                FROM mdm_apple_declarations
                WHERE team_id = ?

                UNION ALL

                SELECT
                    profile_uuid,
                    team_id,
                    name,
                    'android' AS platform,
                    NULL AS identifier,
                    NULL AS scope,
                    NULL AS checksum,
                    created_at,
                    uploaded_at
                FROM mdm_android_configuration_profiles
                WHERE team_id = ?
            ) AS combined_profiles
            ORDER BY name ASC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(tid)
        .bind(tid)
        .bind(tid)
        .bind(tid)
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool())
        .await?)
    }

    /// Gets a single MDM config profile by UUID.
    /// Checks Apple profiles, Windows profiles, Apple declarations, and Android profiles.
    pub async fn get_mdm_config_profile(
        &self,
        profile_uuid: &str,
    ) -> Result<MDMConfigProfileRow> {
        // Try Apple config profiles first
        let row = sqlx::query_as::<_, MDMConfigProfileRow>(
            r#"
            SELECT
                profile_uuid, team_id, name, 'darwin' AS platform,
                identifier, scope, HEX(checksum) AS checksum,
                created_at, uploaded_at
            FROM mdm_apple_configuration_profiles
            WHERE profile_uuid = ?
            "#,
        )
        .bind(profile_uuid)
        .fetch_optional(self.pool())
        .await?;

        if let Some(row) = row {
            return Ok(row);
        }

        // Try Windows profiles
        let row = sqlx::query_as::<_, MDMConfigProfileRow>(
            r#"
            SELECT
                profile_uuid, team_id, name, 'windows' AS platform,
                NULL AS identifier, NULL AS scope, NULL AS checksum,
                created_at, uploaded_at
            FROM mdm_windows_configuration_profiles
            WHERE profile_uuid = ?
            "#,
        )
        .bind(profile_uuid)
        .fetch_optional(self.pool())
        .await?;

        if let Some(row) = row {
            return Ok(row);
        }

        // Try Apple declarations
        let row = sqlx::query_as::<_, MDMConfigProfileRow>(
            r#"
            SELECT
                declaration_uuid AS profile_uuid, team_id, name, 'darwin' AS platform,
                identifier, scope, token AS checksum,
                created_at, uploaded_at
            FROM mdm_apple_declarations
            WHERE declaration_uuid = ?
            "#,
        )
        .bind(profile_uuid)
        .fetch_optional(self.pool())
        .await?;

        if let Some(row) = row {
            return Ok(row);
        }

        // Try Android profiles
        let row = sqlx::query_as::<_, MDMConfigProfileRow>(
            r#"
            SELECT
                profile_uuid, team_id, name, 'android' AS platform,
                NULL AS identifier, NULL AS scope, NULL AS checksum,
                created_at, uploaded_at
            FROM mdm_android_configuration_profiles
            WHERE profile_uuid = ?
            "#,
        )
        .bind(profile_uuid)
        .fetch_optional(self.pool())
        .await?;

        row.ok_or_else(|| DatastoreError::not_found_with_name("MDMConfigProfile", profile_uuid))
    }

    /// Deletes an MDM config profile by UUID.
    /// Tries all platform-specific tables.
    pub async fn delete_mdm_config_profile(&self, profile_uuid: &str) -> Result<()> {
        // Try Apple config profiles
        let result = sqlx::query("DELETE FROM mdm_apple_configuration_profiles WHERE profile_uuid = ?")
            .bind(profile_uuid)
            .execute(self.pool())
            .await?;
        if result.rows_affected() > 0 {
            // Clean up host profile associations
            let _ = sqlx::query(
                "DELETE FROM host_mdm_apple_profiles WHERE profile_uuid = ? AND status IS NULL AND operation_type = 'Install'"
            )
            .bind(profile_uuid)
            .execute(self.pool())
            .await;
            return Ok(());
        }

        // Try Windows profiles
        let result = sqlx::query("DELETE FROM mdm_windows_configuration_profiles WHERE profile_uuid = ?")
            .bind(profile_uuid)
            .execute(self.pool())
            .await?;
        if result.rows_affected() > 0 {
            let _ = sqlx::query("DELETE FROM host_mdm_windows_profiles WHERE profile_uuid = ?")
                .bind(profile_uuid)
                .execute(self.pool())
                .await;
            return Ok(());
        }

        // Try Apple declarations
        let result = sqlx::query("DELETE FROM mdm_apple_declarations WHERE declaration_uuid = ?")
            .bind(profile_uuid)
            .execute(self.pool())
            .await?;
        if result.rows_affected() > 0 {
            return Ok(());
        }

        // Try Android profiles
        let result = sqlx::query("DELETE FROM mdm_android_configuration_profiles WHERE profile_uuid = ?")
            .bind(profile_uuid)
            .execute(self.pool())
            .await?;
        if result.rows_affected() > 0 {
            let _ = sqlx::query("DELETE FROM host_mdm_android_profiles WHERE profile_uuid = ?")
                .bind(profile_uuid)
                .execute(self.pool())
                .await;
            return Ok(());
        }

        Err(DatastoreError::not_found_with_name("MDMConfigProfile", profile_uuid))
    }

    /// Gets MDM profiles summary counts for a team.
    /// Simplified version of Go's `GetMDMProfilesSummary`.
    pub async fn get_mdm_profiles_summary(
        &self,
        team_id: Option<u32>,
    ) -> Result<Vec<MDMProfilesSummaryRow>> {
        let tid = team_id.unwrap_or(0);

        Ok(sqlx::query_as::<_, MDMProfilesSummaryRow>(
            r#"
            SELECT
                COUNT(*) AS count,
                COALESCE(status, 'pending') AS status
            FROM (
                SELECT
                    CASE
                        WHEN status = 'failed' THEN 'failed'
                        WHEN status IS NULL OR status = 'pending' THEN 'pending'
                        WHEN status = 'verifying' THEN 'verifying'
                        WHEN status = 'verified' THEN 'verified'
                        ELSE status
                    END AS status
                FROM host_mdm_apple_profiles hmap
                JOIN hosts h ON h.uuid = hmap.host_uuid
                WHERE h.team_id = ?

                UNION ALL

                SELECT
                    CASE
                        WHEN status = 'failed' THEN 'failed'
                        WHEN status IS NULL OR status = 'pending' THEN 'pending'
                        WHEN status = 'verifying' THEN 'verifying'
                        WHEN status = 'verified' THEN 'verified'
                        ELSE status
                    END AS status
                FROM host_mdm_windows_profiles hwp
                JOIN hosts h ON h.uuid = hwp.host_uuid
                WHERE h.team_id = ?
            ) AS combined_statuses
            GROUP BY status
            "#,
        )
        .bind(tid)
        .bind(tid)
        .fetch_all(self.pool())
        .await?)
    }

    /// Lists MDM commands (Apple + Windows combined).
    /// Simplified version of Go's `ListMDMCommands`.
    pub async fn list_mdm_commands(
        &self,
        page: u32,
        per_page: u32,
    ) -> Result<Vec<MDMCommandRow>> {
        let limit = if per_page == 0 { 20 } else { per_page };
        let offset = page * limit;

        Ok(sqlx::query_as::<_, MDMCommandRow>(
            r#"
            SELECT * FROM (
                SELECT
                    nvq.id AS host_uuid,
                    nvq.command_uuid,
                    COALESCE(NULLIF(nvq.status, ''), 'Pending') AS status,
                    COALESCE(nvq.result_updated_at, nvq.created_at) AS updated_at,
                    nvq.request_type,
                    h.hostname,
                    h.team_id
                FROM nano_view_queue nvq
                INNER JOIN hosts h ON nvq.id = h.uuid
                WHERE nvq.active = 1

                UNION ALL

                SELECT
                    mwe.host_uuid,
                    wmc.command_uuid,
                    COALESCE(NULLIF(wmcr.status_code, ''), '101') AS status,
                    COALESCE(wmc.updated_at, wmc.created_at) AS updated_at,
                    wmc.target_loc_uri AS request_type,
                    h.hostname,
                    h.team_id
                FROM windows_mdm_commands wmc
                LEFT JOIN windows_mdm_command_queue wmcq ON wmcq.command_uuid = wmc.command_uuid
                LEFT JOIN windows_mdm_command_results wmcr ON wmc.command_uuid = wmcr.command_uuid
                INNER JOIN mdm_windows_enrollments mwe ON wmcq.enrollment_id = mwe.id OR wmcr.enrollment_id = mwe.id
                INNER JOIN hosts h ON h.uuid = mwe.host_uuid
            ) AS combined_commands
            ORDER BY updated_at DESC
            LIMIT ? OFFSET ?
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool())
        .await?)
    }

    /// Gets MDM command results by command UUID.
    /// Matches Go's `GetMDMAppleCommandResults` + Windows equivalent.
    pub async fn get_mdm_command_results(
        &self,
        command_uuid: &str,
    ) -> Result<Vec<MDMCommandResultRow>> {
        // Try Apple command results first
        let apple_results = sqlx::query_as::<_, MDMCommandResultRow>(
            r#"
            SELECT
                nq.id AS host_uuid,
                nc.command_uuid,
                COALESCE(ncr.updated_at, nc.created_at) AS updated_at,
                COALESCE(ncr.status, 'Pending') AS status,
                nc.request_type,
                COALESCE(nc.command, '') AS payload,
                COALESCE(ncr.result, '') AS result
            FROM nano_enrollment_queue nq
            JOIN nano_commands nc ON nq.command_uuid = nc.command_uuid
            LEFT JOIN nano_command_results ncr ON nq.id = ncr.id AND nc.command_uuid = ncr.command_uuid
            WHERE nq.active = 1 AND nc.command_uuid = ?
            "#,
        )
        .bind(command_uuid)
        .fetch_all(self.pool())
        .await?;

        if !apple_results.is_empty() {
            return Ok(apple_results);
        }

        // Try Windows command results
        Ok(sqlx::query_as::<_, MDMCommandResultRow>(
            r#"
            SELECT
                mwe.host_uuid,
                wmc.command_uuid,
                COALESCE(wmc.updated_at, wmc.created_at) AS updated_at,
                COALESCE(NULLIF(wmcr.status_code, ''), '101') AS status,
                wmc.target_loc_uri AS request_type,
                COALESCE(wmc.command, '') AS payload,
                COALESCE(wmcr.raw, '') AS result
            FROM windows_mdm_commands wmc
            LEFT JOIN windows_mdm_command_results wmcr ON wmc.command_uuid = wmcr.command_uuid
            LEFT JOIN mdm_windows_enrollments mwe ON wmcr.enrollment_id = mwe.id
            WHERE wmc.command_uuid = ?
            "#,
        )
        .bind(command_uuid)
        .fetch_all(self.pool())
        .await?)
    }

    /// Gets MDM profiles for a specific host.
    /// Combines Apple profiles, declarations, Windows profiles, and Android profiles.
    pub async fn get_host_mdm_profiles(
        &self,
        host_uuid: &str,
    ) -> Result<Vec<HostMDMProfileRow>> {
        Ok(sqlx::query_as::<_, HostMDMProfileRow>(
            r#"
            SELECT * FROM (
                SELECT
                    profile_uuid,
                    profile_name AS name,
                    COALESCE(status, 'pending') AS status,
                    COALESCE(operation_type, '') AS operation_type,
                    COALESCE(detail, '') AS detail,
                    scope,
                    '' AS managed_local_account
                FROM host_mdm_apple_profiles
                WHERE host_uuid = ?
                    AND NOT (operation_type = 'Remove' AND COALESCE(status, 'pending') IN ('verifying', 'verified'))

                UNION ALL

                SELECT
                    declaration_uuid AS profile_uuid,
                    declaration_name AS name,
                    COALESCE(status, 'pending') AS status,
                    COALESCE(operation_type, '') AS operation_type,
                    COALESCE(detail, '') AS detail,
                    scope,
                    '' AS managed_local_account
                FROM host_mdm_apple_declarations
                WHERE host_uuid = ?
                    AND NOT (operation_type = 'Remove' AND COALESCE(status, 'pending') IN ('verifying', 'verified'))

                UNION ALL

                SELECT
                    profile_uuid,
                    profile_name AS name,
                    COALESCE(status, 'pending') AS status,
                    COALESCE(operation_type, '') AS operation_type,
                    COALESCE(detail, '') AS detail,
                    NULL AS scope,
                    NULL AS managed_local_account
                FROM host_mdm_windows_profiles
                WHERE host_uuid = ?
                    AND NOT (operation_type = 'Remove' AND COALESCE(status, 'pending') IN ('verifying', 'verified'))
            ) AS combined_profiles
            ORDER BY name ASC
            "#,
        )
        .bind(host_uuid)
        .bind(host_uuid)
        .bind(host_uuid)
        .fetch_all(self.pool())
        .await?)
    }

    /// Gets disk encryption summary counts.
    /// Simplified version of Go's `GetMDMDiskEncryptionSummary`.
    pub async fn get_mdm_disk_encryption_summary(
        &self,
        team_id: Option<u32>,
    ) -> Result<Vec<MDMProfilesSummaryRow>> {
        let tid = team_id.unwrap_or(0);

        Ok(sqlx::query_as::<_, MDMProfilesSummaryRow>(
            r#"
            SELECT
                COALESCE(status, 'enforcing') AS status,
                COUNT(*) AS count
            FROM (
                SELECT
                    CASE
                        WHEN hmap.status = 'failed' THEN 'failed'
                        WHEN hmap.status = 'verified' THEN 'verified'
                        WHEN hmap.status = 'verifying' THEN 'verifying'
                        WHEN hdek.decryptable IS NOT NULL AND hdek.decryptable = 0 THEN 'action_required'
                        WHEN hmap.operation_type = 'Remove' THEN 'removing_enforcement'
                        ELSE 'enforcing'
                    END AS status
                FROM host_mdm_apple_profiles hmap
                JOIN hosts h ON h.uuid = hmap.host_uuid
                LEFT JOIN host_disk_encryption_keys hdek ON h.id = hdek.host_id
                WHERE hmap.profile_identifier = 'com.apple.security.filevault'
                    AND h.team_id = ?
            ) AS de_statuses
            GROUP BY status
            "#,
        )
        .bind(tid)
        .fetch_all(self.pool())
        .await?)
    }

    /// Gets Apple FileVault summary counts.
    pub async fn get_mdm_apple_filevault_summary(
        &self,
        team_id: Option<u32>,
    ) -> Result<Vec<MDMProfilesSummaryRow>> {
        // Reuses the same query as disk encryption summary for Apple
        self.get_mdm_disk_encryption_summary(team_id).await
    }

    /// Gets status counts for a specific profile.
    pub async fn get_mdm_config_profile_status(
        &self,
        profile_uuid: &str,
        _page: u32,
        _per_page: u32,
    ) -> Result<Vec<ProfileStatusCountRow>> {
        // Try Apple profiles
        let rows = sqlx::query_as::<_, ProfileStatusCountRow>(
            r#"
            SELECT
                COALESCE(status, 'pending') AS status,
                COUNT(*) AS count
            FROM host_mdm_apple_profiles
            WHERE profile_uuid = ?
            GROUP BY status
            "#,
        )
        .bind(profile_uuid)
        .fetch_all(self.pool())
        .await?;

        if !rows.is_empty() {
            return Ok(rows);
        }

        // Try Windows profiles
        let rows = sqlx::query_as::<_, ProfileStatusCountRow>(
            r#"
            SELECT
                COALESCE(status, 'pending') AS status,
                COUNT(*) AS count
            FROM host_mdm_windows_profiles
            WHERE profile_uuid = ?
            GROUP BY status
            "#,
        )
        .bind(profile_uuid)
        .fetch_all(self.pool())
        .await?;

        if !rows.is_empty() {
            return Ok(rows);
        }

        // Try Apple declarations
        Ok(sqlx::query_as::<_, ProfileStatusCountRow>(
            r#"
            SELECT
                COALESCE(status, 'pending') AS status,
                COUNT(*) AS count
            FROM host_mdm_apple_declarations
            WHERE declaration_uuid = ?
            GROUP BY status
            "#,
        )
        .bind(profile_uuid)
        .fetch_all(self.pool())
        .await?)
    }
}
