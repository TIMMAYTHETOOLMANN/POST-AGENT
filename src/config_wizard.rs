use crate::config::AppConfig;
use crate::error::AppResult;
use dialoguer::{Confirm, Input, Select};
use std::fs;
use std::path::Path;

/// Run the interactive configuration wizard
pub fn run_wizard() -> AppResult<AppConfig> {
    println!("Welcome to the TikTok Cross-posting Configuration Wizard!");
    println!("This wizard will help you set up your configuration.");

    // Create a default config to start with
    let mut config = AppConfig::default();

    // TikTok configuration
    println!("\n=== TikTok Configuration ===");
    config.tiktok.username = Input::new()
        .with_prompt("TikTok username")
        .default(config.tiktok.username)
        .interact()?;

    config.tiktok.api_key = Input::new()
        .with_prompt("TikTok API key")
        .default(config.tiktok.api_key)
        .interact()?;

    config.tiktok.api_secret = Input::new()
        .with_prompt("TikTok API secret")
        .default(config.tiktok.api_secret)
        .interact()?;

    let has_access_token = Confirm::new()
        .with_prompt("Do you have a TikTok access token?")
        .default(false)
        .interact()?;

    if has_access_token {
        config.tiktok.access_token = Some(
            Input::new()
                .with_prompt("TikTok access token")
                .interact()?
        );
    }

    // Facebook configuration
    println!("\n=== Facebook Configuration ===");
    config.facebook.page_id = Input::new()
        .with_prompt("Facebook page ID")
        .default(config.facebook.page_id)
        .interact()?;

    config.facebook.api_key = Input::new()
        .with_prompt("Facebook API key")
        .default(config.facebook.api_key)
        .interact()?;

    config.facebook.api_secret = Input::new()
        .with_prompt("Facebook API secret")
        .default(config.facebook.api_secret)
        .interact()?;

    config.facebook.access_token = Input::new()
        .with_prompt("Facebook access token")
        .default(config.facebook.access_token)
        .interact()?;

    // Instagram configuration
    println!("\n=== Instagram Configuration ===");
    config.instagram.account_id = Input::new()
        .with_prompt("Instagram account ID")
        .default(config.instagram.account_id)
        .interact()?;

    config.instagram.api_key = Input::new()
        .with_prompt("Instagram API key")
        .default(config.instagram.api_key)
        .interact()?;

    config.instagram.api_secret = Input::new()
        .with_prompt("Instagram API secret")
        .default(config.instagram.api_secret)
        .interact()?;

    config.instagram.access_token = Input::new()
        .with_prompt("Instagram access token")
        .default(config.instagram.access_token)
        .interact()?;

    // YouTube configuration
    println!("\n=== YouTube Configuration ===");
    config.youtube.channel_id = Input::new()
        .with_prompt("YouTube channel ID")
        .default(config.youtube.channel_id)
        .interact()?;

    config.youtube.api_key = Input::new()
        .with_prompt("YouTube API key")
        .default(config.youtube.api_key)
        .interact()?;

    config.youtube.client_id = Input::new()
        .with_prompt("YouTube client ID")
        .default(config.youtube.client_id)
        .interact()?;

    config.youtube.client_secret = Input::new()
        .with_prompt("YouTube client secret")
        .default(config.youtube.client_secret)
        .interact()?;

    config.youtube.refresh_token = Input::new()
        .with_prompt("YouTube refresh token")
        .default(config.youtube.refresh_token)
        .interact()?;

    // Content configuration
    println!("\n=== Content Configuration ===");
    config.content.hashtag_count = Input::new()
        .with_prompt("Number of hashtags to include")
        .default(config.content.hashtag_count)
        .interact()?;

    // Custom hashtags
    let mut custom_hashtags = config.content.custom_hashtags.join(", ");
    custom_hashtags = Input::new()
        .with_prompt("Custom hashtags (comma separated)")
        .default(custom_hashtags)
        .interact()?;

    config.content.custom_hashtags = custom_hashtags
        .split(',')
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .collect();

    config.content.use_trending_hashtags = Confirm::new()
        .with_prompt("Use trending hashtags?")
        .default(config.content.use_trending_hashtags)
        .interact()?;

    // Scheduler configuration
    println!("\n=== Scheduler Configuration ===");
    config.scheduler.check_interval_minutes = Input::new()
        .with_prompt("Check interval (minutes)")
        .default(config.scheduler.check_interval_minutes)
        .interact()?;

    config.scheduler.legacy_post_interval_minutes = Input::new()
        .with_prompt("Legacy post interval (minutes)")
        .default(config.scheduler.legacy_post_interval_minutes)
        .interact()?;

    config.scheduler.max_posts_per_day = Input::new()
        .with_prompt("Maximum posts per day")
        .default(config.scheduler.max_posts_per_day)
        .interact()?;

    // Save configuration
    let save_config = Confirm::new()
        .with_prompt("Do you want to save this configuration?")
        .default(true)
        .interact()?;

    if save_config {
        let config_path = Input::new()
            .with_prompt("Where do you want to save the configuration?")
            .default("config.toml".to_string())
            .interact()?;

        let toml = toml::to_string_pretty(&config)?;
        fs::write(&config_path, toml)?;
        println!("Configuration saved to {}", config_path);
    }

    Ok(config)
}

/// Generate a .env file with environment variables from the configuration
pub fn generate_env_file(config: &AppConfig, env_path: &str) -> AppResult<()> {
    let mut env_content = String::new();

    // TikTok
    env_content.push_str(&format!("APP__TIKTOK__USERNAME={}\n", config.tiktok.username));
    env_content.push_str(&format!("APP__TIKTOK__API_KEY={}\n", config.tiktok.api_key));
    env_content.push_str(&format!("APP__TIKTOK__API_SECRET={}\n", config.tiktok.api_secret));
    if let Some(token) = &config.tiktok.access_token {
        env_content.push_str(&format!("APP__TIKTOK__ACCESS_TOKEN={}\n", token));
    }

    // Facebook
    env_content.push_str(&format!("APP__FACEBOOK__PAGE_ID={}\n", config.facebook.page_id));
    env_content.push_str(&format!("APP__FACEBOOK__API_KEY={}\n", config.facebook.api_key));
    env_content.push_str(&format!("APP__FACEBOOK__API_SECRET={}\n", config.facebook.api_secret));
    env_content.push_str(&format!("APP__FACEBOOK__ACCESS_TOKEN={}\n", config.facebook.access_token));

    // Instagram
    env_content.push_str(&format!("APP__INSTAGRAM__ACCOUNT_ID={}\n", config.instagram.account_id));
    env_content.push_str(&format!("APP__INSTAGRAM__API_KEY={}\n", config.instagram.api_key));
    env_content.push_str(&format!("APP__INSTAGRAM__API_SECRET={}\n", config.instagram.api_secret));
    env_content.push_str(&format!("APP__INSTAGRAM__ACCESS_TOKEN={}\n", config.instagram.access_token));

    // YouTube
    env_content.push_str(&format!("APP__YOUTUBE__CHANNEL_ID={}\n", config.youtube.channel_id));
    env_content.push_str(&format!("APP__YOUTUBE__API_KEY={}\n", config.youtube.api_key));
    env_content.push_str(&format!("APP__YOUTUBE__CLIENT_ID={}\n", config.youtube.client_id));
    env_content.push_str(&format!("APP__YOUTUBE__CLIENT_SECRET={}\n", config.youtube.client_secret));
    env_content.push_str(&format!("APP__YOUTUBE__REFRESH_TOKEN={}\n", config.youtube.refresh_token));

    // Content
    env_content.push_str(&format!("APP__CONTENT__HASHTAG_COUNT={}\n", config.content.hashtag_count));
    env_content.push_str(&format!("APP__CONTENT__USE_TRENDING_HASHTAGS={}\n", config.content.use_trending_hashtags));

    // Scheduler
    env_content.push_str(&format!("APP__SCHEDULER__CHECK_INTERVAL_MINUTES={}\n", config.scheduler.check_interval_minutes));
    env_content.push_str(&format!("APP__SCHEDULER__LEGACY_POST_INTERVAL_MINUTES={}\n", config.scheduler.legacy_post_interval_minutes));
    env_content.push_str(&format!("APP__SCHEDULER__MAX_POSTS_PER_DAY={}\n", config.scheduler.max_posts_per_day));

    // Write to file
    fs::write(env_path, env_content)?;

    Ok(())
}
