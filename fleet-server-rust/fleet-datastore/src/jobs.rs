//! Job queue operations for the worker/integrations system.
//!
//! SQL queries match the Go code in `server/datastore/mysql/jobs.go`.

use chrono::{DateTime, Utc};

use crate::error::Result;
use crate::mysql::MysqlDatastore;

/// Job states matching Go's `fleet.JobState*` constants.
pub const JOB_STATE_QUEUED: &str = "queued";
pub const JOB_STATE_SUCCESS: &str = "success";
pub const JOB_STATE_FAILURE: &str = "failure";

/// Retry delays matching Go's worker retry strategy.
const RETRY_DELAYS: [std::time::Duration; 5] = [
    std::time::Duration::from_secs(0),
    std::time::Duration::from_secs(300),    // 5 min
    std::time::Duration::from_secs(600),    // 10 min
    std::time::Duration::from_secs(3600),   // 1 hour
    std::time::Duration::from_secs(7200),   // 2 hours
];

/// Maximum number of retries before marking a job as failed.
pub const MAX_RETRIES: i32 = 5;

/// Row type for the jobs table.
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct JobRow {
    pub id: u32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub name: String,
    pub args: Option<serde_json::Value>,
    pub state: String,
    pub retries: i32,
    pub error: Option<String>,
    pub not_before: DateTime<Utc>,
}

impl MysqlDatastore {
    /// Creates a new job in the queue. Matches Go's `NewJob`.
    pub async fn new_job(
        &self,
        name: &str,
        args: Option<&serde_json::Value>,
        state: &str,
        not_before: Option<DateTime<Utc>>,
    ) -> Result<u32> {
        let result = sqlx::query(
            r#"
            INSERT INTO jobs (name, args, state, retries, error, not_before)
            VALUES (?, ?, ?, 0, NULL, COALESCE(?, NOW()))
            "#,
        )
        .bind(name)
        .bind(args)
        .bind(state)
        .bind(not_before)
        .execute(self.pool())
        .await?;

        Ok(result.last_insert_id() as u32)
    }

    /// Gets queued jobs filtered by name, ready to run (not_before <= now).
    /// Matches Go's `GetFilteredQueuedJobs`.
    pub async fn get_filtered_queued_jobs(
        &self,
        max_jobs: u32,
        job_names: &[&str],
    ) -> Result<Vec<JobRow>> {
        let now = Utc::now();

        if job_names.is_empty() {
            return Ok(sqlx::query_as::<_, JobRow>(
                r#"
                SELECT id, created_at, updated_at, name, args, state,
                    retries, error, not_before
                FROM jobs
                WHERE state = ? AND not_before <= ?
                ORDER BY updated_at ASC
                LIMIT ?
                "#,
            )
            .bind(JOB_STATE_QUEUED)
            .bind(now)
            .bind(max_jobs)
            .fetch_all(self.pool())
            .await?);
        }

        let placeholders: Vec<&str> = job_names.iter().map(|_| "?").collect();
        let sql = format!(
            r#"
            SELECT id, created_at, updated_at, name, args, state,
                retries, error, not_before
            FROM jobs
            WHERE state = ? AND not_before <= ?
                AND name IN ({})
            ORDER BY updated_at ASC
            LIMIT ?
            "#,
            placeholders.join(",")
        );

        let mut query = sqlx::query_as::<_, JobRow>(&sql)
            .bind(JOB_STATE_QUEUED)
            .bind(now);

        for name in job_names {
            query = query.bind(*name);
        }

        Ok(query.bind(max_jobs).fetch_all(self.pool()).await?)
    }

    /// Updates a job's state after processing. Matches Go's `UpdateJob`.
    pub async fn update_job(
        &self,
        id: u32,
        state: &str,
        retries: i32,
        error: Option<&str>,
        not_before: Option<DateTime<Utc>>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE jobs
            SET state = ?, retries = ?, error = ?, not_before = COALESCE(?, NOW())
            WHERE id = ?
            "#,
        )
        .bind(state)
        .bind(retries)
        .bind(error)
        .bind(not_before)
        .bind(id)
        .execute(self.pool())
        .await?;

        Ok(())
    }

    /// Gets a single job by ID. Matches Go's `GetJob`.
    pub async fn get_job(&self, id: u32) -> Result<JobRow> {
        Ok(sqlx::query_as::<_, JobRow>(
            r#"
            SELECT id, created_at, updated_at, name, args, state,
                retries, error, not_before
            FROM jobs
            WHERE id = ?
            "#,
        )
        .bind(id)
        .fetch_one(self.pool())
        .await?)
    }

    /// Processes queued jobs for the integrations worker.
    /// Matches Go's `worker.ProcessJobs` loop.
    ///
    /// Since the actual job handlers (Jira, Zendesk, Apple MDM) require
    /// external service clients, this processes the job queue and marks
    /// unhandled jobs for retry. Specific job handlers can be registered
    /// by the caller.
    pub async fn process_worker_jobs(
        &self,
        job_names: &[&str],
        handler: &dyn Fn(&JobRow) -> std::pin::Pin<Box<dyn std::future::Future<Output = std::result::Result<(), String>> + Send + '_>>,
    ) -> Result<u32> {
        let mut total_processed = 0u32;

        loop {
            let jobs = self.get_filtered_queued_jobs(100, job_names).await?;
            if jobs.is_empty() {
                break;
            }

            for job in &jobs {
                let result = handler(job).await;

                match result {
                    Ok(()) => {
                        self.update_job(job.id, JOB_STATE_SUCCESS, job.retries, None, None)
                            .await?;
                    }
                    Err(e) => {
                        let new_retries = job.retries + 1;
                        if new_retries >= MAX_RETRIES as i32 {
                            self.update_job(
                                job.id,
                                JOB_STATE_FAILURE,
                                new_retries,
                                Some(&e),
                                None,
                            )
                            .await?;
                        } else {
                            let delay = RETRY_DELAYS
                                .get(new_retries as usize)
                                .copied()
                                .unwrap_or(RETRY_DELAYS[4]);
                            let not_before = Utc::now()
                                + chrono::Duration::seconds(delay.as_secs() as i64);
                            self.update_job(
                                job.id,
                                JOB_STATE_QUEUED,
                                new_retries,
                                Some(&e),
                                Some(not_before),
                            )
                            .await?;
                        }
                    }
                }

                total_processed += 1;
            }
        }

        Ok(total_processed)
    }

    /// Gets a batch activity by execution ID. Matches Go's `GetBatchActivity`.
    pub async fn get_batch_activity_status(&self, execution_id: &str) -> Result<Option<String>> {
        let result: Option<(String,)> = sqlx::query_as(
            "SELECT status FROM batch_activities WHERE execution_id = ?",
        )
        .bind(execution_id)
        .fetch_optional(self.pool())
        .await?;

        Ok(result.map(|(s,)| s))
    }

    /// Runs a scheduled batch activity. Matches Go's `RunScheduledBatchActivity`.
    pub async fn run_scheduled_batch_activity(&self, execution_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE batch_activities SET status = 'started' WHERE execution_id = ? AND status = 'scheduled'",
        )
        .bind(execution_id)
        .execute(self.pool())
        .await?;

        Ok(())
    }
}
