// Enhanced Scheduler Module
// Handles scheduling and triggering of tasks

use crate::dispatcher::run_dispatch;
use crate::error::{AppError, AppResult};
use log::{debug, error, info, warn};
use tokio_cron_scheduler::{Job, JobScheduler};

// Re-export existing scheduler functions
pub use crate::scheduler::legacy::*;

/// Start the scheduler
pub async fn start_scheduler() -> Result<(), AppError> {
    info!("Starting enhanced scheduler");

    // Create a new job scheduler
    let sched = JobScheduler::new()
        .await
        .map_err(|e| AppError::SchedulerError(format!("Failed to create job scheduler: {}", e)))?;

    // Add a job to run the dispatcher every minute
    sched
        .add(
            Job::new_async("0 * * * * *", |_uuid, _l| {
                Box::pin(async {
                    match run_dispatch().await {
                        Ok(_) => info!("Dispatch completed successfully"),
                        Err(e) => error!("Dispatch failed: {}", e),
                    }
                })
            })
            .map_err(|e| {
                AppError::SchedulerError(format!("Failed to create dispatch job: {}", e))
            })?,
        )
        .await
        .map_err(|e| AppError::SchedulerError(format!("Failed to add dispatch job: {}", e)))?;

    info!("Added dispatch job to scheduler");

    // Start the scheduler
    sched
        .start()
        .await
        .map_err(|e| AppError::SchedulerError(format!("Failed to start scheduler: {}", e)))?;

    info!("Scheduler started successfully");

    Ok(())
}

// Create a submodule for the legacy scheduler functions
pub mod legacy {
    // This will be populated by the existing scheduler functions
}
