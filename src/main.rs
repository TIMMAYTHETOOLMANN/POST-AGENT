use clap::{Parser, Subcommand};
use log::{error, info, warn};
use std::env;
use tiktok_crosspost::config_wizard;
use tiktok_crosspost::error::{AppError, AppResult};
use tiktok_crosspost::{app, config, job_scheduler, platforms, scheduler, tiktok};
use tokio::time::Duration;

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
        #[arg(short = 'p', long, default_value = ".env")]
        env_path: String,
    },
}

#[tokio::main]
async fn main() -> AppResult<()> {
    // Set up environment variables for logging
    if env::var("RUST_LOG").is_err() {
        unsafe {
            env::set_var("RUST_LOG", "info");
        }
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
        return handle_config_command(output, env, env_path).await;
    }

    // Main application flow
    run_main_application().await
}

/// Handle the configuration wizard command
async fn handle_config_command(output: String, env: bool, env_path: String) -> AppResult<()> {
    info!("Starting configuration wizard");

    let config = config_wizard::run_wizard()?;

    // Save configuration to the specified path
    let toml_string = toml::to_string_pretty(&config)
        .map_err(|e| AppError::ConfigError(format!("Failed to serialize configuration: {}", e)))?;

    std::fs::write(&output, toml_string)
        .map_err(|e| AppError::GenericError(format!("Failed to save configuration: {}", e)))?;

    info!("Configuration saved to {}", output);

    // Generate .env file if requested
    if env {
        config_wizard::generate_env_file(&config, &env_path).map_err(|e| {
            AppError::GenericError(format!("Failed to export environment variables: {}", e))
        })?;
        info!("Environment variables exported to {}", env_path);
    }

    info!("Configuration completed successfully!");
    Ok(())
}

/// Run the main application
async fn run_main_application() -> AppResult<()> {
    info!("Starting TikTok Cross-posting Application");

    // Load and validate configuration
    let config = initialize_configuration().await?;

    // Initialize components
    initialize_components().await?;

    // Authenticate with services
    authenticate_services().await?;

    // Set up and start job scheduler
    let scheduler = setup_job_scheduler(&config).await?;
    scheduler.start().await?;
    info!("Job scheduler started");

    // Run initial sync
    run_initial_sync().await?;

    // Keep the application running
    run_main_loop().await
}

/// Initialize and validate configuration
async fn initialize_configuration() -> AppResult<config::AppConfig> {
    let config = config::load_config()?;
    info!("Configuration loaded successfully");

    config::validate_config(&config)
        .map_err(|e| AppError::ConfigError(format!("Configuration validation failed: {}", e)))?;
    info!("Configuration validation passed");

    Ok(config)
}

/// Initialize application components
async fn initialize_components() -> AppResult<()> {
    scheduler::init().await?;
    info!("Scheduler database initialized");
    
    Ok(())
}

/// Authenticate with all services
async fn authenticate_services() -> AppResult<()> {
    // Authenticate with TikTok
    tiktok::authenticate().await.map_err(|_| {
        AppError::AuthenticationError("Failed to authenticate with TikTok".to_string())
    })?;
    info!("Successfully authenticated with TikTok");

    // Authenticate with all platforms
    match platforms::authenticate_all_platforms().await {
        Ok(_) => info!("Successfully authenticated with all platforms"),
        Err(e) => {
            warn!("Some platform authentications failed: {}", e);
            // Continue anyway, as we might still be able to post to some platforms
        },
    }

    Ok(())
}

/// Set up the job scheduler with configured intervals
async fn setup_job_scheduler(config: &config::AppConfig) -> AppResult<job_scheduler::JobScheduler> {
    let mut scheduler = job_scheduler::JobScheduler::new().await?;
    info!("Job scheduler initialized");

    // Set up continuous monitoring job (every N minutes)
    let monitor_cron = format!("0 */{} * * * *", config.scheduler.check_interval_minutes);
    scheduler
        .add(&monitor_cron, || {
            Box::pin(async {
                if let Err(e) = app::monitor_and_post_new_content().await {
                    error!("Error checking for new TikTok content: {}", e);
                } else {
                    info!("Successfully checked for new TikTok content");
                }
            })
        })
        .await?;
    info!("Added monitoring job with schedule: {}", monitor_cron);

    // Set up job for posting legacy content (every N minutes)
    let legacy_cron = format!(
        "0 */{} * * * *",
        config.scheduler.legacy_post_interval_minutes
    );
    scheduler
        .add(&legacy_cron, || {
            Box::pin(async {
                if let Err(e) = app::post_legacy_content().await {
                    error!("Error posting legacy TikTok content: {}", e);
                } else {
                    info!("Successfully posted legacy TikTok content");
                }
            })
        })
        .await?;
    info!("Added legacy content job with schedule: {}", legacy_cron);

    Ok(scheduler)
}

/// Run initial content synchronization
async fn run_initial_sync() -> AppResult<()> {
    info!("Starting initial content synchronization");
    match app::initial_sync_all_content().await {
        Ok(_) => info!("Initial content sync completed successfully"),
        Err(e) => {
            error!("Error during initial content sync: {}", e);
            // Continue anyway, as we can still post legacy content and monitor for new content
        },
    }
    Ok(())
}

/// Main application loop
async fn run_main_loop() -> AppResult<()> {
    info!("TikTok Cross-posting Application is now running");
    info!("Press Ctrl+C to stop the application");

    // Keep the application running efficiently
    loop {
        tokio::time::sleep(Duration::from_secs(60)).await;
    }
}
