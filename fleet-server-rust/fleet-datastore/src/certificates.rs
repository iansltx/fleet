//! Certificate query operations.
//!
//! SQL queries for host_certificates, certificate_templates, and certificate_authorities.

use chrono::{DateTime, Utc};

use crate::error::{DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for host_certificates table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct HostCertificateRow {
    pub id: u64,
    pub host_id: u32,
    pub not_valid_after: DateTime<Utc>,
    pub not_valid_before: DateTime<Utc>,
    pub certificate_authority: bool,
    pub common_name: String,
    pub key_algorithm: String,
    pub key_strength: i32,
    pub key_usage: String,
    pub serial: String,
    pub signing_algorithm: String,
    pub subject_country: String,
    pub subject_org: String,
    pub subject_org_unit: String,
    pub subject_common_name: String,
}

/// Row type for certificate_templates table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CertificateTemplateRow {
    pub id: u32,
    pub team_id: u32,
    pub certificate_authority_id: i32,
    pub name: String,
    pub subject_name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Row type for certificate_authorities table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct CertificateAuthorityRow {
    pub id: i32,
    #[sqlx(rename = "type")]
    pub ca_type: String,
    pub name: String,
    pub url: String,
    #[sqlx(default)]
    pub profile_id: Option<String>,
    #[sqlx(default)]
    pub certificate_common_name: Option<String>,
    #[sqlx(default)]
    pub admin_url: Option<String>,
    #[sqlx(default)]
    pub username: Option<String>,
    #[sqlx(default)]
    pub challenge_url: Option<String>,
    #[sqlx(default)]
    pub client_id: Option<String>,
}

impl MysqlDatastore {
    /// Lists certificates for a host.
    pub async fn list_host_certificates(&self, host_id: u32) -> Result<Vec<HostCertificateRow>> {
        Ok(sqlx::query_as::<_, HostCertificateRow>(
            r#"
            SELECT id, host_id, not_valid_after, not_valid_before, certificate_authority,
                common_name, key_algorithm, key_strength, key_usage, serial,
                signing_algorithm, subject_country, subject_org, subject_org_unit,
                subject_common_name
            FROM host_certificates
            WHERE host_id = ?
            ORDER BY not_valid_after DESC
            "#,
        )
        .bind(host_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Creates a certificate template.
    pub async fn create_certificate_template(
        &self,
        team_id: u32,
        ca_id: i32,
        name: &str,
        subject_name: &str,
    ) -> Result<CertificateTemplateRow> {
        let result = sqlx::query(
            "INSERT INTO certificate_templates (team_id, certificate_authority_id, name, subject_name) VALUES (?, ?, ?, ?)"
        )
        .bind(team_id)
        .bind(ca_id)
        .bind(name)
        .bind(subject_name)
        .execute(self.pool())
        .await?;
        let id = result.last_insert_id() as u32;
        self.get_certificate_template(id).await
    }

    /// Gets a certificate template by ID.
    pub async fn get_certificate_template(&self, id: u32) -> Result<CertificateTemplateRow> {
        sqlx::query_as::<_, CertificateTemplateRow>(
            "SELECT id, team_id, certificate_authority_id, name, subject_name, created_at, updated_at FROM certificate_templates WHERE id = ?"
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_id("CertificateTemplate", id as u64))
    }

    /// Lists certificate templates.
    pub async fn list_certificate_templates(&self) -> Result<Vec<CertificateTemplateRow>> {
        Ok(sqlx::query_as::<_, CertificateTemplateRow>(
            "SELECT id, team_id, certificate_authority_id, name, subject_name, created_at, updated_at FROM certificate_templates ORDER BY name"
        )
        .fetch_all(self.pool())
        .await?)
    }

    /// Deletes a certificate template.
    pub async fn delete_certificate_template(&self, id: u32) -> Result<()> {
        let result = sqlx::query("DELETE FROM certificate_templates WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("CertificateTemplate", id as u64));
        }
        Ok(())
    }

    /// Creates a certificate authority.
    pub async fn create_certificate_authority(
        &self,
        ca_type: &str,
        name: &str,
        url: &str,
    ) -> Result<CertificateAuthorityRow> {
        let result = sqlx::query(
            "INSERT INTO certificate_authorities (`type`, name, url) VALUES (?, ?, ?)"
        )
        .bind(ca_type)
        .bind(name)
        .bind(url)
        .execute(self.pool())
        .await?;
        let id = result.last_insert_id() as i32;
        self.get_certificate_authority(id).await
    }

    /// Gets a certificate authority by ID.
    pub async fn get_certificate_authority(&self, id: i32) -> Result<CertificateAuthorityRow> {
        sqlx::query_as::<_, CertificateAuthorityRow>(
            r#"SELECT id, `type`, name, url, profile_id, certificate_common_name,
                admin_url, username, challenge_url, client_id
            FROM certificate_authorities WHERE id = ?"#
        )
        .bind(id)
        .fetch_optional(self.pool())
        .await?
        .ok_or_else(|| DatastoreError::not_found_with_id("CertificateAuthority", id as u64))
    }

    /// Lists certificate authorities.
    pub async fn list_certificate_authorities(&self) -> Result<Vec<CertificateAuthorityRow>> {
        Ok(sqlx::query_as::<_, CertificateAuthorityRow>(
            r#"SELECT id, `type`, name, url, profile_id, certificate_common_name,
                admin_url, username, challenge_url, client_id
            FROM certificate_authorities ORDER BY name"#
        )
        .fetch_all(self.pool())
        .await?)
    }

    /// Deletes a certificate authority.
    pub async fn delete_certificate_authority(&self, id: i32) -> Result<()> {
        let result = sqlx::query("DELETE FROM certificate_authorities WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;
        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("CertificateAuthority", id as u64));
        }
        Ok(())
    }

    /// Updates a certificate authority name and URL.
    pub async fn update_certificate_authority(
        &self,
        id: i32,
        name: &str,
        url: &str,
    ) -> Result<CertificateAuthorityRow> {
        let result = sqlx::query(
            "UPDATE certificate_authorities SET name = ?, url = ? WHERE id = ?"
        )
        .bind(name)
        .bind(url)
        .bind(id)
        .execute(self.pool())
        .await?;
        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("CertificateAuthority", id as u64));
        }
        self.get_certificate_authority(id).await
    }
}
