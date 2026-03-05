//! Software query operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/software.go`.

use chrono::{DateTime, Utc};

use crate::error::{DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for software queries matching the software table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SoftwareRow {
    pub id: u32,
    pub name: String,
    pub version: String,
    pub source: String,
    #[sqlx(default)]
    pub bundle_identifier: Option<String>,
    #[sqlx(rename = "release")]
    pub sw_release: String,
    pub vendor: String,
    pub arch: String,
    #[sqlx(default)]
    pub title_id: Option<u32>,
    #[sqlx(default)]
    pub checksum: String,
}

/// Row type for host software entries.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HostSoftwareRow {
    pub host_id: u32,
    pub software_id: u32,
    #[sqlx(default)]
    pub last_opened_at: Option<DateTime<Utc>>,
}

/// Row type for software titles.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SoftwareTitleRow {
    pub id: u32,
    pub name: String,
    pub source: String,
    pub browser: String,
    #[sqlx(default)]
    pub hosts_count: Option<u32>,
}

impl MysqlDatastore {
    /// Lists software with optional team filter and pagination.
    ///
    /// Simplified version of Go's `ListSoftware`.
    pub async fn list_software(
        &self,
        team_id: Option<u32>,
        per_page: u32,
        page: u32,
    ) -> Result<Vec<SoftwareRow>> {
        let limit = if per_page == 0 { 20 } else { per_page };
        let offset = page * limit;

        if let Some(tid) = team_id {
            Ok(sqlx::query_as::<_, SoftwareRow>(
                r#"
                SELECT DISTINCT s.id, s.name, s.version, s.source, s.bundle_identifier,
                    s.`release`, s.vendor, s.arch, s.title_id, s.checksum
                FROM software s
                JOIN host_software hs ON s.id = hs.software_id
                JOIN hosts h ON hs.host_id = h.id
                WHERE h.team_id = ?
                ORDER BY s.name, s.version
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(tid)
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await?)
        } else {
            Ok(sqlx::query_as::<_, SoftwareRow>(
                r#"
                SELECT s.id, s.name, s.version, s.source, s.bundle_identifier,
                    s.`release`, s.vendor, s.arch, s.title_id, s.checksum
                FROM software s
                ORDER BY s.name, s.version
                LIMIT ? OFFSET ?
                "#,
            )
            .bind(limit)
            .bind(offset)
            .fetch_all(self.pool())
            .await?)
        }
    }

    /// Lists software for a host. Matches Go's host software queries.
    ///
    /// SELECT s.* FROM software s
    /// JOIN host_software hs ON s.id = hs.software_id
    /// WHERE hs.host_id = ?
    pub async fn list_software_for_host(&self, host_id: u32) -> Result<Vec<SoftwareRow>> {
        Ok(sqlx::query_as::<_, SoftwareRow>(
            r#"
            SELECT s.id, s.name, s.version, s.source, s.bundle_identifier,
                s.`release`, s.vendor, s.arch, s.title_id, s.checksum
            FROM software s
            JOIN host_software hs ON s.id = hs.software_id
            WHERE hs.host_id = ?
            ORDER BY s.name, s.version
            "#,
        )
        .bind(host_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Gets software by ID. Matches Go's software-by-id pattern.
    ///
    /// SELECT * FROM software WHERE id = ?
    pub async fn software_by_id(&self, id: u32) -> Result<SoftwareRow> {
        sqlx::query_as::<_, SoftwareRow>(
            r#"
            SELECT id, name, version, source, bundle_identifier,
                `release`, vendor, arch, title_id, checksum
            FROM software WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_id("Software", id as u64))
    }

    /// Lists software titles with host counts.
    ///
    /// SELECT st.id, st.name, st.source, st.browser,
    ///   (SELECT COUNT(DISTINCT hs.host_id) ...) AS hosts_count
    /// FROM software_titles st
    pub async fn list_software_titles(
        &self,
        team_id: Option<u32>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<SoftwareTitleRow>> {
        let mut sql = r#"
            SELECT st.id, st.name, st.source, st.browser, stc.hosts_count
            FROM software_titles st
            LEFT JOIN software_titles_host_counts stc ON st.id = stc.software_title_id
                AND stc.team_id = ?
                AND stc.hosts_count > 0
            WHERE TRUE
        "#
        .to_string();

        let effective_team_id = team_id.unwrap_or(0);

        sql.push_str(&format!(
            " ORDER BY st.name ASC LIMIT {} OFFSET {}",
            limit, offset
        ));

        Ok(sqlx::query_as::<_, SoftwareTitleRow>(&sql)
            .bind(effective_team_id)
            .fetch_all(self.pool())
            .await?)
    }

    /// Updates a software title name. Matches Go's `UpdateSoftwareTitleName`.
    ///
    /// UPDATE software_titles SET name = ? WHERE id = ?
    pub async fn update_software_title_name(&self, id: u32, name: &str) -> Result<()> {
        let result = sqlx::query("UPDATE software_titles SET name = ? WHERE id = ?")
            .bind(name)
            .bind(id)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("SoftwareTitle", id as u64));
        }
        Ok(())
    }

    /// Deletes a software installer for a title.
    ///
    /// DELETE FROM software_installers WHERE title_id = ?
    pub async fn delete_software_installer(&self, title_id: u32) -> Result<()> {
        let result = sqlx::query("DELETE FROM software_installers WHERE title_id = ?")
            .bind(title_id)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("SoftwareInstaller", title_id as u64));
        }
        Ok(())
    }

    /// Gets installed paths for a host's software.
    ///
    /// SELECT * FROM host_software_installed_paths WHERE host_id = ?
    pub async fn get_host_software_installed_paths(
        &self,
        host_id: u32,
    ) -> Result<Vec<HostSoftwareInstalledPathRow>> {
        Ok(sqlx::query_as::<_, HostSoftwareInstalledPathRow>(
            r#"
            SELECT id, host_id, software_id, installed_path
            FROM host_software_installed_paths
            WHERE host_id = ?
            "#,
        )
        .bind(host_id)
        .fetch_all(self.pool())
        .await?)
    }
}

/// Row type for host_software_installed_paths.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HostSoftwareInstalledPathRow {
    pub id: u32,
    pub host_id: u32,
    pub software_id: u32,
    pub installed_path: String,
}
