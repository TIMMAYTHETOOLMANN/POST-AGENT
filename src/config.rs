use anyhow::Result;
use config::{Config, File};
use serde::{Deserialize, Serialize};
use std::path::Path;
use log::info;
use once_cell::sync::Lazy;

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
}

/// TikTok API configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TikTokConfig {
    /// TikTok username to fetch content from
    pub username: String,

    /// TikTok API key
    pub api_key: String,

    /// TikTok API secret
    pub api_secret: String,

    /// TikTok access token
    pub access_token: Option<String>,
}

/// Facebook API configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct FacebookConfig {
    /// Facebook page ID
    pub page_id: String,

    /// Facebook API key
    pub api_key: String,

    /// Facebook API secret
    pub api_secret: String,

    /// Facebook access token
    pub access_token: String,
}

/// Instagram API configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct InstagramConfig {
    /// Instagram account ID
    pub account_id: String,

    /// Instagram API key
    pub api_key: String,

    /// Instagram API secret
    pub api_secret: String,

    /// Instagram access token
    pub access_token: String,
}

/// YouTube API configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct YouTubeConfig {
    /// YouTube channel ID
    pub channel_id: String,

    /// YouTube API key
    pub api_key: String,

    /// YouTube client ID
    pub client_id: String,

    /// YouTube client secret
    pub client_secret: String,

    /// YouTube refresh token
    pub refresh_token: String,
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

/// Default configuration values
impl Default for AppConfig {
    fn default() -> Self {
        Self {
            tiktok: TikTokConfig {
                username: "your_tiktok_username".to_string(),
                api_key: "your_tiktok_api_key".to_string(),
                api_secret: "your_tiktok_api_secret".to_string(),
                access_token: None,
            },
            facebook: FacebookConfig {
                page_id: "your_facebook_page_id".to_string(),
                api_key: "your_facebook_api_key".to_string(),
                api_secret: "your_facebook_api_secret".to_string(),
                access_token: "your_facebook_access_token".to_string(),
            },
            instagram: InstagramConfig {
                account_id: "your_instagram_account_id".to_string(),
                api_key: "your_instagram_api_key".to_string(),
                api_secret: "your_instagram_api_secret".to_string(),
                access_token: "your_instagram_access_token".to_string(),
            },
            youtube: YouTubeConfig {
                channel_id: "your_youtube_channel_id".to_string(),
                api_key: "your_youtube_api_key".to_string(),
                client_id: "your_youtube_client_id".to_string(),
                client_secret: "your_youtube_client_secret".to_string(),
                refresh_token: "your_youtube_refresh_token".to_string(),
            },
            content: ContentConfig {
                hashtag_count: 5,
                custom_hashtags: vec!["fyp".to_string(), "viral".to_string()],
                tagline_templates: vec![
                    "You won't believe what happens next! 😱".to_string(),
                    "This changed everything for me! 🔥".to_string(),
                    "Wait for it... 👀".to_string(),
                ],
                use_trending_hashtags: true,
            },
            scheduler: SchedulerConfig {
                check_interval_minutes: 5,
                legacy_post_interval_minutes: 10,
                max_posts_per_day: 24,
            },
        }
    }
}

/// Global configuration instance
static CONFIG: Lazy<std::sync::Mutex<Option<AppConfig>>> = Lazy::new(|| {
    std::sync::Mutex::new(None)
});

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
        info!("No configuration file found at {}, using defaults", config_path);

        // Create default config file
        let toml = toml::to_string_pretty(&default_config)?;
        std::fs::write(&config_path, toml)?;
        info!("Created default configuration file at {}", config_path);
    }

    // Add environment variables with prefix APP_
    config_builder = config_builder.add_source(config::Environment::with_prefix("APP").separator("__"));

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
    config_lock.clone().expect("Configuration not loaded. Call load_config() first.")
}

/// Validate configuration settings
pub fn validate_config(config: &AppConfig) -> Result<()> {
    let mut errors = Vec::new();

    // TikTok validation
    if config.tiktok.username.is_empty() {
        errors.push("TikTok username cannot be empty");
    }
    if config.tiktok.api_key.is_empty() {
        errors.push("TikTok API key cannot be empty");
    }
    if config.tiktok.api_secret.is_empty() {
        errors.push("TikTok API secret cannot be empty");
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
        return Err(anyhow::anyhow!("Configuration validation failed: {}", errors.join(", ")));
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
