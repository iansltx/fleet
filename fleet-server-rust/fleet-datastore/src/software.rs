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

    /// Lists CVEs for a software ID.
    ///
    /// SELECT sc.cve, sc.created_at, sc.resolved_in_version
    /// FROM software_cve sc WHERE sc.software_id = ?
    pub async fn list_cves_for_software(&self, software_id: u32) -> Result<Vec<SoftwareCveRow>> {
        Ok(sqlx::query_as::<_, SoftwareCveRow>(
            r#"
            SELECT sc.cve, sc.software_id, sc.created_at,
                sc.resolved_in_version
            FROM software_cve sc
            WHERE sc.software_id = ?
            "#,
        )
        .bind(software_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Lists CVEs for multiple software IDs.
    pub async fn list_cves_for_software_ids(&self, software_ids: &[u32]) -> Result<Vec<SoftwareCveRow>> {
        if software_ids.is_empty() {
            return Ok(Vec::new());
        }
        let placeholders: Vec<&str> = software_ids.iter().map(|_| "?").collect();
        let sql = format!(
            "SELECT cve, software_id, created_at, resolved_in_version FROM software_cve WHERE software_id IN ({})",
            placeholders.join(",")
        );
        let mut query = sqlx::query_as::<_, SoftwareCveRow>(&sql);
        for id in software_ids {
            query = query.bind(id);
        }
        Ok(query.fetch_all(self.pool()).await?)
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

    /// Lists vulnerabilities with host counts, joining cve_meta for scoring data.
    /// Simplified version of Go's ListVulnerabilities.
    pub async fn list_vulnerabilities(
        &self,
        team_id: Option<u32>,
        query: Option<&str>,
        exploit: Option<bool>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<VulnerabilityRow>> {
        let mut sql = r#"
            SELECT
                vhc.cve AS cve,
                (SELECT MIN(created_at) FROM (
                    SELECT created_at FROM software_cve WHERE cve = vhc.cve
                    UNION ALL
                    SELECT created_at FROM operating_system_vulnerabilities WHERE cve = vhc.cve
                ) AS combined_dates) AS created_at,
                cm.cvss_score,
                cm.epss_probability,
                cm.cisa_known_exploit,
                cm.published AS cve_published,
                cm.description,
                vhc.host_count AS hosts_count,
                vhc.updated_at AS hosts_count_updated_at
            FROM vulnerability_host_counts vhc
            LEFT JOIN cve_meta cm ON cm.cve = vhc.cve
            WHERE vhc.host_count > 0
            AND (
                EXISTS (SELECT 1 FROM software_cve WHERE cve = vhc.cve)
                OR EXISTS (SELECT 1 FROM operating_system_vulnerabilities WHERE cve = vhc.cve)
            )
        "#.to_string();

        let mut bind_values: Vec<String> = Vec::new();

        if let Some(tid) = team_id {
            sql.push_str(" AND vhc.global_stats = 0 AND vhc.team_id = ?");
            bind_values.push(tid.to_string());
        } else {
            sql.push_str(" AND vhc.global_stats = 1");
        }

        if exploit == Some(true) {
            sql.push_str(" AND cm.cisa_known_exploit = 1");
        }

        if let Some(q) = query {
            if !q.is_empty() {
                sql.push_str(" AND vhc.cve LIKE ?");
                bind_values.push(format!("%{}%", q));
            }
        }

        sql.push_str(&format!(" ORDER BY vhc.cve ASC LIMIT {} OFFSET {}", limit, offset));

        let mut db_query = sqlx::query_as::<_, VulnerabilityRow>(&sql);
        for val in &bind_values {
            db_query = db_query.bind(val);
        }
        Ok(db_query.fetch_all(self.pool()).await?)
    }

    /// Gets a single vulnerability by CVE, with host counts and metadata.
    /// Simplified version of Go's Vulnerability.
    pub async fn get_vulnerability(
        &self,
        cve: &str,
        team_id: Option<u32>,
    ) -> Result<VulnerabilityRow> {
        let mut sql = r#"
            SELECT DISTINCT
                cm.cve,
                LEAST(COALESCE(osv.created_at, NOW()), COALESCE(sc.created_at, NOW())) AS created_at,
                cm.cvss_score,
                cm.epss_probability,
                cm.cisa_known_exploit,
                cm.published AS cve_published,
                cm.description,
                COALESCE(vhc.host_count, 0) AS hosts_count,
                COALESCE(vhc.updated_at, NOW()) AS hosts_count_updated_at
            FROM cve_meta cm
            JOIN (
                SELECT cve FROM software_cve WHERE cve = ?
                UNION
                SELECT cve FROM operating_system_vulnerabilities WHERE cve = ?
            ) AS cve_table ON cm.cve = cve_table.cve
            LEFT JOIN operating_system_vulnerabilities osv ON osv.cve = cm.cve
            LEFT JOIN software_cve sc ON sc.cve = cm.cve
            LEFT JOIN vulnerability_host_counts vhc ON cm.cve = vhc.cve
        "#.to_string();

        if let Some(tid) = team_id {
            sql.push_str(" AND vhc.team_id = ? AND vhc.global_stats = 0");
            let row = sqlx::query_as::<_, VulnerabilityRow>(&sql)
                .bind(cve)
                .bind(cve)
                .bind(tid)
                .fetch_optional(self.pool())
                .await?
                .ok_or_else(|| DatastoreError::not_found_with_name("Vulnerability", cve))?;
            if row.hosts_count == 0 {
                return Err(DatastoreError::not_found_with_name("Vulnerability", cve));
            }
            Ok(row)
        } else {
            sql.push_str(" AND vhc.team_id = 0 AND vhc.global_stats = 1");
            let row = sqlx::query_as::<_, VulnerabilityRow>(&sql)
                .bind(cve)
                .bind(cve)
                .fetch_optional(self.pool())
                .await?
                .ok_or_else(|| DatastoreError::not_found_with_name("Vulnerability", cve))?;
            if row.hosts_count == 0 {
                return Err(DatastoreError::not_found_with_name("Vulnerability", cve));
            }
            Ok(row)
        }
    }

    /// Lists fleet maintained apps with optional pagination.
    pub async fn list_fleet_maintained_apps(
        &self,
        query: Option<&str>,
        limit: u32,
        offset: u32,
    ) -> Result<Vec<FleetMaintainedAppRow>> {
        if let Some(q) = query {
            if !q.is_empty() {
                let pattern = format!("%{}%", q);
                return Ok(sqlx::query_as::<_, FleetMaintainedAppRow>(
                    r#"SELECT id, name, slug, platform, unique_identifier, created_at, updated_at
                    FROM fleet_maintained_apps
                    WHERE name LIKE ?
                    ORDER BY name ASC
                    LIMIT ? OFFSET ?"#
                )
                .bind(pattern)
                .bind(limit)
                .bind(offset)
                .fetch_all(self.pool())
                .await?);
            }
        }
        Ok(sqlx::query_as::<_, FleetMaintainedAppRow>(
            r#"SELECT id, name, slug, platform, unique_identifier, created_at, updated_at
            FROM fleet_maintained_apps
            ORDER BY name ASC
            LIMIT ? OFFSET ?"#
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(self.pool())
        .await?)
    }

    /// Gets a fleet maintained app by ID.
    pub async fn get_fleet_maintained_app(&self, id: u32) -> Result<FleetMaintainedAppRow> {
        sqlx::query_as::<_, FleetMaintainedAppRow>(
            "SELECT id, name, slug, platform, unique_identifier, created_at, updated_at FROM fleet_maintained_apps WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_id("FleetMaintainedApp", id as u64))
    }

    /// Gets a software install result by execution_id.
    pub async fn get_software_install_result(&self, execution_id: &str) -> Result<SoftwareInstallResultRow> {
        sqlx::query_as::<_, SoftwareInstallResultRow>(
            r#"SELECT execution_id, host_id, software_installer_id, software_title_id,
                install_script_exit_code, install_script_output, pre_install_query_output,
                post_install_script_exit_code, post_install_script_output,
                self_service, created_at, updated_at
            FROM host_software_installs
            WHERE execution_id = ?"#
        )
        .bind(execution_id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_name("SoftwareInstallResult", execution_id))
    }
}

/// Row type for host_software_installs table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SoftwareInstallResultRow {
    pub execution_id: String,
    pub host_id: u32,
    #[sqlx(default)]
    pub software_installer_id: Option<u32>,
    #[sqlx(default)]
    pub software_title_id: Option<u32>,
    #[sqlx(default)]
    pub install_script_exit_code: Option<i32>,
    #[sqlx(default)]
    pub install_script_output: Option<String>,
    #[sqlx(default)]
    pub pre_install_query_output: Option<String>,
    #[sqlx(default)]
    pub post_install_script_exit_code: Option<i32>,
    #[sqlx(default)]
    pub post_install_script_output: Option<String>,
    pub self_service: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Row type for software_cve table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct SoftwareCveRow {
    pub cve: String,
    pub software_id: u32,
    pub created_at: DateTime<Utc>,
    #[sqlx(default)]
    pub resolved_in_version: Option<String>,
}

/// Row type for vulnerability listing queries (joins vulnerability_host_counts + cve_meta + earliest created_at).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct VulnerabilityRow {
    pub cve: String,
    pub created_at: DateTime<Utc>,
    pub hosts_count: u32,
    pub hosts_count_updated_at: DateTime<Utc>,
    #[sqlx(default)]
    pub cvss_score: Option<f64>,
    #[sqlx(default)]
    pub epss_probability: Option<f64>,
    #[sqlx(default)]
    pub cisa_known_exploit: Option<bool>,
    #[sqlx(default)]
    pub cve_published: Option<DateTime<Utc>>,
    #[sqlx(default)]
    pub description: Option<String>,
}

/// Row type for fleet_maintained_apps table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct FleetMaintainedAppRow {
    pub id: u32,
    pub name: String,
    pub slug: String,
    pub platform: String,
    pub unique_identifier: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Row type for host_software_installed_paths.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HostSoftwareInstalledPathRow {
    pub id: u32,
    pub host_id: u32,
    pub software_id: u32,
    pub installed_path: String,
}
