//! Host certificate query operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/host_certificates.go`.

use chrono::{DateTime, Utc};

use crate::error::Result;
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

impl MysqlDatastore {
    /// Lists certificates for a host.
    ///
    /// SELECT * FROM host_certificates WHERE host_id = ?
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
}
