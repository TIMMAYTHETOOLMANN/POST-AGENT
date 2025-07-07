use crate::error::{AppError, AppResult};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tokio::time::{interval, Duration};

/// A simple job scheduler using Tokio's interval functionality
pub struct JobScheduler {
    jobs: Vec<JoinHandle<()>>,
}

impl JobScheduler {
    /// Create a new JobScheduler
    pub async fn new() -> AppResult<Self> {
        Ok(Self { jobs: Vec::new() })
    }

    /// Add a new job to the scheduler
    pub async fn add<F>(&mut self, cron_expression: &str, job: F) -> AppResult<()>
    where
        F: Fn() -> Pin<Box<dyn std::future::Future<Output = ()> + Send>> + Send + Sync + 'static,
    {
        // Parse the cron expression to get the interval in minutes
        let minutes = parse_cron_minutes(cron_expression)?;
        let duration = Duration::from_secs(minutes * 60);

        // Create a new task that runs the job at the specified interval
        let job = Arc::new(Mutex::new(job));
        let handle = tokio::spawn(async move {
            let mut interval = interval(duration);
            loop {
                interval.tick().await;
                let job_guard = job.lock().await;
                (job_guard)().await;
            }
        });

        self.jobs.push(handle);
        Ok(())
    }

    /// Start the scheduler
    pub async fn start(self) -> AppResult<()> {
        // The jobs are already running, so we just need to return
        Ok(())
    }
}

/// Parse a cron expression to get the interval in minutes
fn parse_cron_minutes(cron: &str) -> AppResult<u64> {
    // Simple parser for cron expressions like "0 */5 * * * *"
    // This only handles the minute field with */n format
    let parts: Vec<&str> = cron.split_whitespace().collect();
    if parts.len() != 6 {
        return Err(AppError::SchedulerError(
            "Invalid cron expression format".to_string(),
        ));
    }

    let minute_part = parts[1];
    if minute_part.starts_with("*/") {
        let interval = minute_part
            .trim_start_matches("*/")
            .parse::<u64>()
            .map_err(|_| {
                AppError::SchedulerError("Invalid minute interval in cron expression".to_string())
            })?;
        Ok(interval)
    } else {
        Err(AppError::SchedulerError(
            "Unsupported cron expression format".to_string(),
        ))
    }
}
