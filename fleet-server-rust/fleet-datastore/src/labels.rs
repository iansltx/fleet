//! Label CRUD operations.
//!
//! SQL queries match the Go code in `server/datastore/mysql/labels.go`.

use chrono::{DateTime, Utc};

use crate::error::{DatastoreError, Result};
use crate::mysql::MysqlDatastore;

/// Row type for label queries matching the labels table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LabelRow {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub description: String,
    pub query: String,
    pub platform: String,
    pub label_type: u32,
    pub label_membership_type: u32,
    #[sqlx(default)]
    pub host_count: Option<i32>,
    pub team_id: Option<u32>,
}

/// Row type for label specs (YAML import/export).
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct LabelSpecRow {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub query: String,
    pub platform: String,
    pub label_type: u32,
    pub label_membership_type: u32,
}

impl MysqlDatastore {
    /// Creates a new label. Matches Go's label creation pattern.
    ///
    /// INSERT INTO labels (name, description, query, platform, label_type,
    ///   label_membership_type, team_id)
    /// VALUES (?, ?, ?, ?, ?, ?, ?)
    pub async fn new_label(
        &self,
        name: &str,
        description: &str,
        query: &str,
        platform: &str,
        label_type: u32,
        label_membership_type: u32,
        team_id: Option<u32>,
    ) -> Result<u32> {
        let result = sqlx::query(
            r#"
            INSERT INTO labels (name, description, query, platform,
                label_type, label_membership_type, team_id)
            VALUES (?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(name)
        .bind(description)
        .bind(query)
        .bind(platform)
        .bind(label_type)
        .bind(label_membership_type)
        .bind(team_id)
        .execute(self.pool())
        .await?;

        Ok(result.last_insert_id() as u32)
    }

    /// Gets a label by ID. Matches Go's `Label` function.
    ///
    /// SELECT * FROM labels WHERE id = ?
    pub async fn label_by_id(&self, id: u32) -> Result<LabelRow> {
        sqlx::query_as::<_, LabelRow>("SELECT * FROM labels WHERE id = ?")
            .bind(id)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_id("Label", id as u64))
    }

    /// Gets a label by name. Matches Go's `LabelByName`.
    ///
    /// SELECT * FROM labels WHERE name = ?
    pub async fn label_by_name(&self, name: &str) -> Result<LabelRow> {
        sqlx::query_as::<_, LabelRow>("SELECT * FROM labels WHERE name = ?")
            .bind(name)
            .fetch_optional(self.pool())
            .await?
            .ok_or_else(|| DatastoreError::not_found_with_name("Label", name))
    }

    /// Saves/updates a label. Matches Go's `SaveLabel`.
    ///
    /// UPDATE labels SET name=?, description=?, query=?, platform=? WHERE id=?
    pub async fn save_label(
        &self,
        id: u32,
        name: &str,
        description: &str,
        query: &str,
        platform: &str,
    ) -> Result<()> {
        let result = sqlx::query(
            "UPDATE labels SET name = ?, description = ?, query = ?, platform = ? WHERE id = ?",
        )
        .bind(name)
        .bind(description)
        .bind(query)
        .bind(platform)
        .bind(id)
        .execute(self.pool())
        .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Label", id as u64));
        }
        Ok(())
    }

    /// Deletes a label by ID. Matches Go's label deletion pattern.
    ///
    /// DELETE FROM labels WHERE id = ?
    pub async fn delete_label(&self, id: u32) -> Result<()> {
        let result = sqlx::query("DELETE FROM labels WHERE id = ?")
            .bind(id)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_id("Label", id as u64));
        }
        Ok(())
    }

    /// Deletes a label by name. Matches Go's `deleteEntityByName`.
    ///
    /// DELETE FROM labels WHERE name = ?
    pub async fn delete_label_by_name(&self, name: &str) -> Result<()> {
        let result = sqlx::query("DELETE FROM labels WHERE name = ?")
            .bind(name)
            .execute(self.pool())
            .await?;

        if result.rows_affected() == 0 {
            return Err(DatastoreError::not_found_with_name("Label", name));
        }
        Ok(())
    }

    /// Lists all labels. Matches Go's `ListLabels`.
    ///
    /// SELECT * FROM labels ORDER BY id
    pub async fn list_labels(&self, team_id: Option<u32>) -> Result<Vec<LabelRow>> {
        let mut sql = "SELECT * FROM labels WHERE TRUE".to_string();
        if let Some(tid) = team_id {
            if tid == 0 {
                sql.push_str(" AND team_id IS NULL");
            } else {
                sql.push_str(&format!(" AND team_id = {}", tid));
            }
        }
        sql.push_str(" ORDER BY id");

        Ok(sqlx::query_as::<_, LabelRow>(&sql)
            .fetch_all(self.pool())
            .await?)
    }

    /// Records label membership for a host. Matches Go's label_membership table.
    ///
    /// INSERT INTO label_membership (label_id, host_id) VALUES (?, ?)
    /// ON DUPLICATE KEY UPDATE updated_at = NOW()
    pub async fn record_label_membership(&self, label_id: u32, host_id: u32) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO label_membership (label_id, host_id) VALUES (?, ?)
            ON DUPLICATE KEY UPDATE updated_at = NOW()
            "#,
        )
        .bind(label_id)
        .bind(host_id)
        .execute(self.pool())
        .await?;
        Ok(())
    }

    /// Deletes label membership for a host.
    ///
    /// DELETE FROM label_membership WHERE label_id = ? AND host_id = ?
    pub async fn delete_label_membership(&self, label_id: u32, host_id: u32) -> Result<()> {
        sqlx::query("DELETE FROM label_membership WHERE label_id = ? AND host_id = ?")
            .bind(label_id)
            .bind(host_id)
            .execute(self.pool())
            .await?;
        Ok(())
    }

    /// Lists labels for a specific host. Matches Go's `ListLabelsForHost`.
    ///
    /// SELECT l.* FROM labels l
    /// JOIN label_membership lm ON l.id = lm.label_id
    /// WHERE lm.host_id = ?
    pub async fn list_labels_for_host(&self, host_id: u32) -> Result<Vec<LabelRow>> {
        Ok(sqlx::query_as::<_, LabelRow>(
            r#"
            SELECT l.* FROM labels l
            JOIN label_membership lm ON l.id = lm.label_id
            WHERE lm.host_id = ?
            ORDER BY l.id
            "#,
        )
        .bind(host_id)
        .fetch_all(self.pool())
        .await?)
    }

    /// Updates label membership for host vitals labels.
    /// Matches Go's `cronHostVitalsLabelMembership` + `UpdateLabelMembershipByHostCriteria`.
    ///
    /// For each label with label_membership_type = 2 (HostVitals), re-evaluates
    /// membership based on the label's host vitals criteria query.
    ///
    /// The Go implementation parses JSON criteria from the label's `query` field
    /// and dynamically builds SQL based on the vital type (platform, username, etc.).
    /// This Rust implementation executes the same pattern: for each host vitals label,
    /// it inserts matching hosts and removes non-matching ones.
    pub async fn update_host_vitals_label_membership(&self) -> Result<u32> {
        // label_membership_type = 2 is LabelMembershipTypeHostVitals
        let labels: Vec<LabelRow> = sqlx::query_as::<_, LabelRow>(
            "SELECT * FROM labels WHERE label_membership_type = 2 ORDER BY id",
        )
        .fetch_all(self.pool())
        .await?;

        let mut updated = 0u32;

        for label in &labels {
            // Parse the host vitals criteria from the label's query field.
            // The query field contains JSON like {"vital":"platform","value":"darwin"}
            // or {"vital":"username","value":"admin"}.
            let criteria: serde_json::Value = match serde_json::from_str(&label.query) {
                Ok(v) => v,
                Err(e) => {
                    tracing::warn!(label_id = label.id, error = %e, "failed to parse host vitals criteria");
                    continue;
                }
            };

            let vital = criteria.get("vital").and_then(|v| v.as_str()).unwrap_or("");
            let value = criteria.get("value").and_then(|v| v.as_str()).unwrap_or("");

            if vital.is_empty() || value.is_empty() {
                tracing::warn!(label_id = label.id, "empty vital or value in host vitals criteria");
                continue;
            }

            // Build the candidate query based on the vital type.
            // Matches Go's CalculateHostVitalsQuery patterns.
            let (candidate_sql, team_clause) = match vital {
                "platform" => (
                    format!(
                        "SELECT {} AS label_id, hosts.id AS host_id FROM {{hosts}} WHERE hosts.platform = ?",
                        label.id
                    ),
                    true,
                ),
                "username" => (
                    format!(
                        "SELECT {} AS label_id, hosts.id AS host_id FROM {{hosts}} \
                         JOIN host_users hu ON hu.host_id = hosts.id WHERE hu.username = ?",
                        label.id
                    ),
                    true,
                ),
                _ => {
                    tracing::warn!(label_id = label.id, vital = vital, "unsupported host vital type");
                    continue;
                }
            };

            // Apply team scoping if the label has a team_id
            let candidate_sql = if team_clause {
                if let Some(tid) = label.team_id {
                    candidate_sql.replace(
                        "{hosts}",
                        &format!(
                            "hosts JOIN (SELECT {} team_id) label_team ON label_team.team_id = hosts.team_id",
                            tid
                        ),
                    )
                } else {
                    candidate_sql.replace("{hosts}", "hosts")
                }
            } else {
                candidate_sql.replace("{hosts}", "hosts")
            };

            // Insert new members
            let insert_sql = format!(
                "INSERT INTO label_membership (label_id, host_id) \
                 SELECT candidate.label_id, candidate.host_id FROM ({}) AS candidate \
                 ON DUPLICATE KEY UPDATE host_id = label_membership.host_id",
                candidate_sql
            );
            sqlx::query(&insert_sql)
                .bind(value)
                .execute(self.pool())
                .await?;

            // Remove stale members
            let delete_sql = format!(
                "DELETE FROM label_membership WHERE label_id = {} \
                 AND NOT EXISTS (SELECT 1 FROM ({}) AS candidate \
                 WHERE candidate.host_id = label_membership.host_id)",
                label.id, candidate_sql
            );
            sqlx::query(&delete_sql)
                .bind(value)
                .execute(self.pool())
                .await?;

            updated += 1;
        }

        Ok(updated)
    }
}
