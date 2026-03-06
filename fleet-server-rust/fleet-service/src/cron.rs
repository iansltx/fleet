//! Background cron scheduler for periodic tasks.
//!
//! Provides a `CronScheduler` that manages named periodic jobs. Each job
//! runs on a configurable interval and can be triggered ad-hoc via the
//! `/api/v1/fleet/trigger` endpoint.
//!
//! Mirrors Go's `server/fleet/cron_schedules.go` and `server/cron/` package.

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{info, error};

/// Status of a cron job run.
#[derive(Debug, Clone, PartialEq)]
pub enum CronStatus {
    Pending,
    Completed,
    Expired,
    Canceled,
}

/// Stats for a cron job run.
#[derive(Debug, Clone)]
pub struct CronStats {
    pub name: String,
    pub status: CronStatus,
    pub started_at: chrono::DateTime<chrono::Utc>,
}

/// A named periodic job.
pub struct CronJob {
    /// Name of the schedule (e.g. "cleanups_then_aggregation").
    pub name: String,
    /// How often the job runs.
    pub interval: Duration,
    /// The job function. Takes the schedule name and returns a result.
    pub func: Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>,
}

/// Internal state for a running schedule.
struct RunningSchedule {
    handle: JoinHandle<()>,
    trigger_tx: tokio::sync::mpsc::Sender<()>,
    current_stats: Option<CronStats>,
}

/// The cron scheduler manages all background periodic jobs.
pub struct CronScheduler {
    schedules: Arc<Mutex<HashMap<String, RunningSchedule>>>,
}

impl CronScheduler {
    /// Create a new empty scheduler.
    pub fn new() -> Self {
        Self {
            schedules: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register and start a periodic job.
    pub async fn register(&self, job: CronJob) {
        let name = job.name.clone();
        let interval = job.interval;
        let func = Arc::new(job.func);
        let schedules = self.schedules.clone();

        let (trigger_tx, mut trigger_rx) = tokio::sync::mpsc::channel::<()>(1);

        let sched_name = name.clone();
        let handle = tokio::spawn(async move {
            let mut ticker = tokio::time::interval(interval);
            // Skip the first immediate tick
            ticker.tick().await;

            loop {
                tokio::select! {
                    _ = ticker.tick() => {
                        run_job(&sched_name, &func, &schedules).await;
                    }
                    Some(()) = trigger_rx.recv() => {
                        info!(schedule = %sched_name, "ad-hoc trigger received");
                        run_job(&sched_name, &func, &schedules).await;
                    }
                    else => break,
                }
            }
        });

        let mut map = self.schedules.lock().await;
        map.insert(name.clone(), RunningSchedule {
            handle,
            trigger_tx,
            current_stats: None,
        });
    }

    /// Trigger an ad-hoc run of the named schedule.
    ///
    /// Returns an error if the schedule doesn't exist or is already running.
    pub async fn trigger(&self, name: &str) -> Result<(), CronTriggerError> {
        let map = self.schedules.lock().await;
        let sched = map.get(name).ok_or_else(|| {
            let names: Vec<String> = map.keys().cloned().collect();
            CronTriggerError::NotFound {
                name: name.to_string(),
                available: names,
            }
        })?;

        // Check if currently running
        if let Some(stats) = &sched.current_stats {
            if stats.status == CronStatus::Pending {
                return Err(CronTriggerError::Conflict {
                    name: name.to_string(),
                    started_at: stats.started_at,
                });
            }
        }

        sched.trigger_tx.send(()).await.map_err(|_| CronTriggerError::ScheduleStopped)?;
        Ok(())
    }

    /// Return the names of all registered schedules.
    pub async fn schedule_names(&self) -> Vec<String> {
        let map = self.schedules.lock().await;
        let mut names: Vec<String> = map.keys().cloned().collect();
        names.sort();
        names
    }

    /// Shut down all schedules.
    pub async fn shutdown(&self) {
        let mut map = self.schedules.lock().await;
        for (name, sched) in map.drain() {
            info!(schedule = %name, "stopping cron schedule");
            sched.handle.abort();
        }
    }
}

impl Default for CronScheduler {
    fn default() -> Self {
        Self::new()
    }
}

/// Run a single job execution, updating stats.
async fn run_job(
    name: &str,
    func: &Arc<Box<dyn Fn() -> Pin<Box<dyn Future<Output = Result<(), String>> + Send>> + Send + Sync>>,
    schedules: &Arc<Mutex<HashMap<String, RunningSchedule>>>,
) {
    // Mark as pending
    {
        let mut map = schedules.lock().await;
        if let Some(sched) = map.get_mut(name) {
            sched.current_stats = Some(CronStats {
                name: name.to_string(),
                status: CronStatus::Pending,
                started_at: chrono::Utc::now(),
            });
        }
    }

    let start = std::time::Instant::now();
    let result = (func)().await;
    let elapsed = start.elapsed();

    match &result {
        Ok(()) => {
            info!(schedule = %name, elapsed_ms = elapsed.as_millis(), "cron job completed");
        }
        Err(e) => {
            error!(schedule = %name, error = %e, elapsed_ms = elapsed.as_millis(), "cron job failed");
        }
    }

    // Mark as completed
    {
        let mut map = schedules.lock().await;
        if let Some(sched) = map.get_mut(name) {
            if let Some(stats) = &mut sched.current_stats {
                stats.status = CronStatus::Completed;
            }
        }
    }
}

/// Errors from trigger requests.
#[derive(Debug, thiserror::Error)]
pub enum CronTriggerError {
    #[error("unknown schedule '{name}'; available: {}", available.join(", "))]
    NotFound { name: String, available: Vec<String> },

    #[error("conflicts with current status of {name} schedule: run started at {started_at}")]
    Conflict {
        name: String,
        started_at: chrono::DateTime<chrono::Utc>,
    },

    #[error("schedule has stopped")]
    ScheduleStopped,
}

/// Well-known cron schedule names matching Go constants in
/// `server/fleet/cron_schedules.go`.
pub mod schedule_names {
    pub const CLEANUPS_THEN_AGGREGATION: &str = "cleanups_then_aggregation";
    pub const FREQUENT_CLEANUPS: &str = "frequent_cleanups";
    pub const USAGE_STATISTICS: &str = "usage_statistics";
    pub const VULNERABILITIES: &str = "vulnerabilities";
    pub const AUTOMATIONS: &str = "automations";
    pub const INTEGRATIONS: &str = "integrations";
    pub const ACTIVITIES_STREAMING: &str = "activities_streaming";
    pub const QUERY_RESULTS_CLEANUP: &str = "query_results_cleanup";
    pub const UPCOMING_ACTIVITIES_MAINTENANCE: &str = "upcoming_activities_maintenance";
    pub const HOST_VITALS_LABEL_MEMBERSHIP: &str = "host_vitals_label_membership";
    pub const BATCH_ACTIVITY_COMPLETION_CHECKER: &str = "batch_activity_completion_checker";
    pub const SCHEDULED_BATCH_ACTIVITIES: &str = "scheduled_batch_activities";
}
