mod config;
pub mod config_wizard;
mod content;
mod dispatcher;
mod error;
mod facebook_client;
mod instagram_client;
mod media_processor;
mod platforms;
mod scheduler;
mod state_tracker;
mod tiktok;
mod youtube_uploader;

use crate::error::{AppError, AppResult};
use chrono::Utc;
use clap::{Parser, Subcommand};
use log::{error, info, warn};
use std::env;
use std::time::Duration;
use tokio_cron_scheduler::{Job, JobScheduler};

/// TikTok Cross-posting Application
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Command to execute
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the configuration wizard
    Config {
        /// Path to save the configuration file
        #[arg(short, long, default_value = "config.toml")]
        output: String,

        /// Generate a .env file with environment variables
        #[arg(short, long)]
        env: bool,

        /// Path to save the .env file
        #[arg(short, long, default_value = ".env")]
        env_path: String,
    },
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Set up environment variables for logging
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }

    // Initialize logging
    env_logger::init();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Handle commands
    if let Some(Commands::Config {
        output,
        env,
        env_path,
    }) = cli.command
    {
        info!("Starting configuration wizard");

        match config_wizard::run_wizard() {
            Ok(config) => {
                // Save configuration to the specified path
                match toml::to_string_pretty(&config) {
                    Ok(toml_string) => match std::fs::write(&output, toml_string) {
                        Ok(_) => info!("Configuration saved to {}", output),
                        Err(e) => {
                            error!("Failed to save configuration: {}", e);
                            return Err(AppError::IoError(format!(
                                "Failed to save configuration: {}",
                                e
                            )));
                        }
                    },
                    Err(e) => {
                        error!("Failed to serialize configuration: {}", e);
                        return Err(AppError::ConfigError(format!(
                            "Failed to serialize configuration: {}",
                            e
                        )));
                    }
                }

                // Generate .env file if requested
                if env {
                    match config_wizard::generate_env_file(&config, &env_path) {
                        Ok(_) => info!("Environment variables exported to {}", env_path),
                        Err(e) => {
                            error!("Failed to export environment variables: {}", e);
                            return Err(AppError::IoError(format!(
                                "Failed to export environment variables: {}",
                                e
                            )));
                        }
                    }
                }

                info!("Configuration completed successfully!");
                return Ok(());
            }
            Err(e) => {
                error!("Error during configuration: {}", e);
                return Err(AppError::ConfigError(format!(
                    "Error during configuration: {}",
                    e
                )));
            }
        }
    }

    info!("Starting TikTok Cross-posting Application");

    // Load configuration
    let config = config::load_config()?;
    info!("Configuration loaded successfully");

    // Initialize the scheduler database
    scheduler::init().await?;
    info!("Scheduler database initialized");

    // Authenticate with TikTok
    match tiktok::authenticate().await {
        Ok(_) => info!("Successfully authenticated with TikTok"),
        Err(e) => {
            error!("Failed to authenticate with TikTok: {}", e);
            return Err(AppError::AuthenticationError(
                "Failed to authenticate with TikTok".to_string(),
            ));
        }
    }

    // Authenticate with all platforms
    match platforms::authenticate_all_platforms().await {
        Ok(_) => info!("Successfully authenticated with all platforms"),
        Err(e) => {
            warn!("Some platform authentications failed: {}", e);
            // Continue anyway, as we might still be able to post to some platforms
        }
    }

    // Start the enhanced scheduler
    match scheduler::start_scheduler().await {
        Ok(_) => info!("Enhanced scheduler started successfully"),
        Err(e) => {
            error!("Failed to start enhanced scheduler: {}", e);
            return Err(e);
        }
    }

    // For backward compatibility, we'll also set up the legacy jobs
    // Initialize the job scheduler
    let mut legacy_scheduler = JobScheduler::new();
    info!("Legacy job scheduler initialized");

    // Set up continuous monitoring job (every 5 minutes)
    let monitor_cron = format!("0 */{} * * * *", config.scheduler.check_interval_minutes);
    let monitor_job = Job::new_async(&monitor_cron, |_uuid, _l| {
        Box::pin(async {
            match monitor_and_post_new_content().await {
                Ok(_) => info!("Successfully checked for new TikTok content"),
                Err(e) => error!("Error checking for new TikTok content: {}", e),
            }
        })
    })?;
    legacy_scheduler.add(monitor_job)?;
    info!("Added monitoring job with schedule: {}", monitor_cron);

    // Set up job for posting legacy content (every 10 minutes)
    let legacy_cron = format!(
        "0 */{} * * * *",
        config.scheduler.legacy_post_interval_minutes
    );
    let legacy_job = Job::new_async(&legacy_cron, |_uuid, _l| {
        Box::pin(async {
            match post_legacy_content().await {
                Ok(_) => info!("Successfully posted legacy TikTok content"),
                Err(e) => error!("Error posting legacy TikTok content: {}", e),
            }
        })
    })?;
    legacy_scheduler.add(legacy_job)?;
    info!("Added legacy content job with schedule: {}", legacy_cron);

    // Start the legacy scheduler
    legacy_scheduler.start().await?;
    info!("Legacy job scheduler started");

    // Initial sync of all content
    info!("Starting initial content synchronization");
    match initial_sync_all_content().await {
        Ok(_) => info!("Initial content sync completed successfully"),
        Err(e) => {
            error!("Error during initial content sync: {}", e);
            // Continue anyway, as we can still post legacy content and monitor for new content
        }
    }

    info!("TikTok Cross-posting Application is now running");
    info!("Press Ctrl+C to stop the application");

    // Keep the application running
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}

/// Performs the initial synchronization of all TikTok content to all platforms
async fn initial_sync_all_content() -> AppResult<()> {
    info!("Starting initial sync of all TikTok content");

    // Fetch all TikTok videos
    let mut videos = tiktok::fetch_all_videos().await?;
    info!("Fetched {} TikTok videos", videos.len());

    if videos.is_empty() {
        info!("No TikTok videos found to sync");
        return Ok(());
    }

    // Sort videos by creation date (oldest first)
    videos.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    // Process and post each video to all platforms
    let mut success_count = 0;
    for mut video in videos {
        // Check if this video has already been posted
        if scheduler::is_video_posted(&video.id).await? {
            info!("Skipping already posted video: {}", video.id);
            continue;
        }

        // Download the video content
        match tiktok::download_video_content(&mut video).await {
            Ok(_) => info!("Downloaded content for video: {}", video.id),
            Err(e) => {
                error!("Failed to download content for video {}: {}", video.id, e);
                continue;
            }
        }

        // Process content (add hashtags and taglines)
        let processed_content = match content::process_content(video).await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to process content: {}", e);
                continue;
            }
        };

        // Post to all platforms
        match platforms::post_to_all_platforms(processed_content.clone()).await {
            Ok(_) => {
                info!(
                    "Successfully posted video to all platforms: {}",
                    processed_content.original_video.id
                );
                success_count += 1;

                // Record that this content has been posted
                if let Err(e) =
                    scheduler::mark_as_posted(&processed_content.original_video.id).await
                {
                    error!("Failed to mark video as posted: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to post video to all platforms: {}", e);
                // Continue with next video
            }
        }

        // Add a small delay between posts to avoid rate limiting
        tokio::time::sleep(Duration::from_secs(2)).await;
    }

    info!(
        "Initial sync completed. Successfully posted {} videos",
        success_count
    );
    Ok(())
}

/// Monitors TikTok for new content and posts it to all platforms
async fn monitor_and_post_new_content() -> AppResult<()> {
    info!("Checking for new TikTok content");

    // Fetch new TikTok videos
    let new_videos = tiktok::fetch_new_videos().await?;
    info!("Found {} new TikTok videos", new_videos.len());

    if new_videos.is_empty() {
        return Ok(());
    }

    // Process and post each new video to all platforms
    for mut video in new_videos {
        // Check if this video has already been posted
        if scheduler::is_video_posted(&video.id).await? {
            info!("Skipping already posted video: {}", video.id);
            continue;
        }

        // Download the video content
        match tiktok::download_video_content(&mut video).await {
            Ok(_) => info!("Downloaded content for video: {}", video.id),
            Err(e) => {
                error!("Failed to download content for video {}: {}", video.id, e);
                continue;
            }
        }

        // Process content (add hashtags and taglines)
        let processed_content = match content::process_content(video).await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to process content: {}", e);
                continue;
            }
        };

        // Post to all platforms
        match platforms::post_to_all_platforms(processed_content.clone()).await {
            Ok(_) => {
                info!(
                    "Successfully posted new video to all platforms: {}",
                    processed_content.original_video.id
                );

                // Record that this content has been posted
                if let Err(e) =
                    scheduler::mark_as_posted(&processed_content.original_video.id).await
                {
                    error!("Failed to mark video as posted: {}", e);
                }
            }
            Err(e) => {
                error!("Failed to post new video to all platforms: {}", e);
                // Continue with next video
            }
        }
    }

    Ok(())
}

/// Posts legacy TikTok content according to the schedule
async fn post_legacy_content() -> AppResult<()> {
    info!("Posting next legacy TikTok content");

    // Get the next legacy video to post
    if let Some(mut video) = scheduler::get_next_legacy_video().await? {
        info!("Found legacy video to post: {}", video.id);

        // Check if this video has already been posted
        if scheduler::is_video_posted(&video.id).await? {
            info!("Skipping already posted legacy video: {}", video.id);
            return Ok(());
        }

        // Download the video content if needed
        if video.local_video_path.is_none() || video.local_cover_path.is_none() {
            match tiktok::download_video_content(&mut video).await {
                Ok(_) => info!("Downloaded content for legacy video: {}", video.id),
                Err(e) => {
                    error!(
                        "Failed to download content for legacy video {}: {}",
                        video.id, e
                    );
                    return Err(e);
                }
            }
        }

        // Process content (add hashtags and taglines)
        let processed_content = content::process_content(video).await?;

        // Post to all platforms
        match platforms::post_to_all_platforms(processed_content.clone()).await {
            Ok(_) => {
                info!(
                    "Successfully posted legacy video to all platforms: {}",
                    processed_content.original_video.id
                );

                // Record that this content has been posted
                scheduler::mark_as_posted(&processed_content.original_video.id).await?;

                info!("Successfully posted legacy content");
            }
            Err(e) => {
                error!("Failed to post legacy video to all platforms: {}", e);
                return Err(e);
            }
        }
    } else {
        info!("No more legacy content to post");
    }

    Ok(())
}
