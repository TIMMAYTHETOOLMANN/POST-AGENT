use anyhow::Result;
use config::{Config, File};
use log::info;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Configuration for the TikTok cross-posting application
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    /// TikTok API configuration
    pub tiktok: TikTokConfig,

    /// Facebook API configuration
    pub facebook: FacebookConfig,

    /// Instagram API configuration
    pub instagram: InstagramConfig,

    /// YouTube API configuration
    pub youtube: YouTubeConfig,

    /// Content processing configuration
    pub content: ContentConfig,

    /// Scheduling configuration
    pub scheduler: SchedulerConfig,

    /// Browser automation configuration
    pub browser: BrowserConfig,

    /// System configuration
    pub system: SystemConfig,
}

/// TikTok browser automation configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TikTokConfig {
    /// TikTok username to fetch content from
    pub username: String,

    /// TikTok login email for browser automation
    pub email: String,

    /// TikTok login password for browser automation
    pub password: String,
}

/// Facebook browser automation configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FacebookConfig {
    /// Facebook profile ID (for personal accounts) or page ID (for business)
    pub profile_id: String,

    /// Facebook login email for browser automation
    pub email: String,

    /// Facebook login password for browser automation
    pub password: String,
}

/// Instagram browser automation configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InstagramConfig {
    /// Instagram account ID
    pub account_id: String,

    /// Instagram login email for browser automation
    pub email: String,

    /// Instagram login password for browser automation
    pub password: String,
}

/// YouTube browser automation configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct YouTubeConfig {
    /// YouTube channel ID
    pub channel_id: String,

    /// YouTube login email for browser automation
    pub email: String,

    /// YouTube login password for browser automation
    pub password: String,
}

/// Content processing configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ContentConfig {
    /// Number of hashtags to include
    pub hashtag_count: usize,

    /// List of custom hashtags to always include
    pub custom_hashtags: Vec<String>,

    /// List of tagline templates
    pub tagline_templates: Vec<String>,

    /// Whether to include trending hashtags
    pub use_trending_hashtags: bool,
}

/// Scheduler configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SchedulerConfig {
    /// Interval in minutes to check for new TikTok content
    pub check_interval_minutes: u64,

    /// Interval in minutes between posting legacy content
    pub legacy_post_interval_minutes: u64,

    /// Maximum number of posts per day
    pub max_posts_per_day: usize,
}

/// Browser automation configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct BrowserConfig {
    /// Whether to run browser in headless mode
    pub headless: bool,

    /// Timeout for page loads in seconds
    pub page_load_timeout: u64,

    /// Timeout for element waits in seconds
    pub element_wait_timeout: u64,

    /// Number of retry attempts for failed operations
    pub max_retries: usize,

    /// Delay between retry attempts in milliseconds
    pub retry_delay_ms: u64,

    /// Browser window width
    pub window_width: u32,

    /// Browser window height
    pub window_height: u32,

    /// User agent string to use
    pub user_agent: Option<String>,
}

/// System configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SystemConfig {
    /// Automation mode: 'playwright' or 'api'
    pub automation_mode: String,

    /// Debug mode: true for verbose logging, false for production
    pub debug: bool,
}

/// Default configuration values
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tiktok: TikTokConfig {
                username: "".to_string(),
                email: "".to_string(),
                password: "".to_string(),
            },
            facebook: FacebookConfig {
                profile_id: "".to_string(),
                email: "".to_string(),
                password: "".to_string(),
            },
            instagram: InstagramConfig {
                account_id: "".to_string(),
                email: "".to_string(),
                password: "".to_string(),
            },
            youtube: YouTubeConfig {
                channel_id: "".to_string(),
                email: "".to_string(),
                password: "".to_string(),
            },
            content: ContentConfig {
                hashtag_count: 10,
                custom_hashtags: vec!["fyp".to_string(), "viral".to_string(), "trending".to_string()],
                tagline_templates: vec![
                    "You won't believe what happens next! 😱".to_string(),
                    "This changed everything for me! 🔥".to_string(),
                    "Wait for it... 👀".to_string(),
                    "Mind blown! 🤯".to_string(),
                    "This is insane! 🚀".to_string(),
                ],
                use_trending_hashtags: true,
            },
            scheduler: SchedulerConfig {
                check_interval_minutes: 10,
                legacy_post_interval_minutes: 10,
                max_posts_per_day: 144, // 24 hours * 60 minutes / 10 minutes
            },
            browser: BrowserConfig {
                headless: true,
                page_load_timeout: 30,
                element_wait_timeout: 10,
                max_retries: 3,
                retry_delay_ms: 2000,
                window_width: 1920,
                window_height: 1080,
                user_agent: None,
            },
            system: SystemConfig {
                automation_mode: "playwright".to_string(),
                debug: false,
            },
        }
    }
}

/// Global configuration instance
static CONFIG: Lazy<std::sync::Mutex<Option<AppConfig>>> =
    Lazy::new(|| std::sync::Mutex::new(None));

/// Load configuration from file and environment variables
pub fn load_config() -> Result<AppConfig> {
    // Load from .env file if it exists
    dotenv::dotenv().ok();

    // Default config path
    let config_path = std::env::var("CONFIG_PATH").unwrap_or_else(|_| "config.toml".to_string());

    let mut config_builder = Config::builder();

    // Add default config
    let default_config = AppConfig::default();
    config_builder = config_builder.add_source(config::Config::try_from(&default_config)?);

    // Add config from file if it exists
    if Path::new(&config_path).exists() {
        config_builder = config_builder.add_source(File::with_name(&config_path));
        info!("Loaded configuration from {}", config_path);
    } else {
        info!(
            "No configuration file found at {}, using defaults",
            config_path
        );

        // Create default config file
        let toml = toml::to_string_pretty(&default_config)?;
        std::fs::write(&config_path, toml)?;
        info!("Created default configuration file at {}", config_path);
    }

    // Add environment variables with prefix APP_
    config_builder =
        config_builder.add_source(config::Environment::with_prefix("APP").separator("__"));

    // Build config
    let config = config_builder.build()?.try_deserialize::<AppConfig>()?;

    // Store config globally
    let mut config_lock = CONFIG.lock().unwrap();
    *config_lock = Some(config.clone());

    Ok(config)
}

/// Get the global configuration
pub fn get_config() -> AppConfig {
    let config_lock = CONFIG.lock().unwrap();
    config_lock
        .clone()
        .expect("Configuration not loaded. Call load_config() first.")
}

/// Validate configuration settings
pub fn validate_config(config: &AppConfig) -> Result<()> {
    let mut errors = Vec::new();

    // TikTok validation
    if config.tiktok.username.is_empty() {
        errors.push("TikTok username cannot be empty");
    }
    if config.tiktok.email.is_empty() {
        errors.push("TikTok email cannot be empty");
    }
    if config.tiktok.password.is_empty() {
        errors.push("TikTok password cannot be empty");
    }

    // Scheduler validation
    if config.scheduler.check_interval_minutes == 0 {
        errors.push("Check interval must be greater than 0");
    }
    if config.scheduler.legacy_post_interval_minutes == 0 {
        errors.push("Legacy post interval must be greater than 0");
    }
    if config.scheduler.max_posts_per_day == 0 {
        errors.push("Max posts per day must be greater than 0");
    }

    // Content validation
    if config.content.hashtag_count == 0 {
        errors.push("Hashtag count must be greater than 0");
    }

    if !errors.is_empty() {
        return Err(anyhow::anyhow!(
            "Configuration validation failed: {}",
            errors.join(", ")
        ));
    }

    Ok(())
}

/// Test configuration connectivity (placeholder for actual API tests)
pub fn test_config_connectivity(config: &AppConfig) -> Result<()> {
    info!("Testing configuration connectivity...");

    // Here we would normally test API connectivity
    // For now, just validate that required fields are present
    validate_config(config)?;

    info!("Configuration validation passed");
    Ok(())
}
