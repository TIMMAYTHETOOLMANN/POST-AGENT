use crate::config::AppConfig;
use crate::error::AppResult;
use dialoguer::{Confirm, Input, Password};
use std::fs;

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

    config.tiktok.email = Input::new()
        .with_prompt("TikTok login email")
        .default(config.tiktok.email)
        .interact()?;

    config.tiktok.password = Password::new()
        .with_prompt("TikTok login password")
        .interact()?;

    // Facebook configuration
    println!("\n=== Facebook Personal Account Configuration ===");
    config.facebook.profile_id = Input::new()
        .with_prompt("Facebook profile ID (numeric user ID)")
        .default(config.facebook.profile_id)
        .interact()?;

    config.facebook.email = Input::new()
        .with_prompt("Facebook login email")
        .default(config.facebook.email)
        .interact()?;

    config.facebook.password = Password::new()
        .with_prompt("Facebook login password")
        .interact()?;

    // Instagram configuration
    println!("\n=== Instagram Personal Account Configuration ===");
    config.instagram.account_id = Input::new()
        .with_prompt("Instagram account ID (numeric user ID)")
        .default(config.instagram.account_id)
        .interact()?;

    config.instagram.email = Input::new()
        .with_prompt("Instagram login email or username")
        .default(config.instagram.email)
        .interact()?;

    config.instagram.password = Password::new()
        .with_prompt("Instagram login password")
        .interact()?;

    // YouTube configuration
    println!("\n=== YouTube Personal Account Configuration ===");
    config.youtube.channel_id = Input::new()
        .with_prompt("YouTube channel ID (starts with UC)")
        .default(config.youtube.channel_id)
        .interact()?;

    config.youtube.email = Input::new()
        .with_prompt("YouTube/Google login email")
        .default(config.youtube.email)
        .interact()?;

    config.youtube.password = Password::new()
        .with_prompt("YouTube/Google login password")
        .interact()?;

    // System configuration
    println!("\n=== System Configuration ===");
    config.system.automation_mode = Input::new()
        .with_prompt("Automation mode (playwright or api)")
        .default(config.system.automation_mode)
        .interact()?;

    config.system.debug = Confirm::new()
        .with_prompt("Enable debug mode?")
        .default(config.system.debug)
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

    // Validate configuration before saving
    println!("\n=== Configuration Validation ===");
    match crate::config::validate_config(&config) {
        Ok(_) => println!("✓ Configuration validation passed"),
        Err(e) => {
            println!("⚠ Configuration validation failed: {}", e);
            let continue_anyway = Confirm::new()
                .with_prompt("Do you want to continue with this configuration anyway?")
                .default(false)
                .interact()?;
            if !continue_anyway {
                println!("Configuration cancelled. Please run the wizard again to fix the issues.");
                return Err(crate::error::AppError::GenericError(
                    "Configuration validation failed".to_string(),
                ));
            }
        },
    }

    // Test connectivity (optional)
    let test_connectivity = Confirm::new()
        .with_prompt("Do you want to test the configuration connectivity?")
        .default(true)
        .interact()?;

    if test_connectivity {
        match crate::config::test_config_connectivity(&config) {
            Ok(_) => println!("✓ Configuration connectivity test passed"),
            Err(e) => println!("⚠ Configuration connectivity test failed: {}", e),
        }
    }

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
    env_content.push_str(&format!(
        "APP__TIKTOK__USERNAME={}\n",
        config.tiktok.username
    ));
    env_content.push_str(&format!(
        "APP__TIKTOK__EMAIL={}\n", 
        config.tiktok.email
    ));
    env_content.push_str(&format!(
        "APP__TIKTOK__PASSWORD={}\n", 
        config.tiktok.password
    ));

    // Facebook
    env_content.push_str(&format!(
        "APP__FACEBOOK__PROFILE_ID={}\n",
        config.facebook.profile_id
    ));
    env_content.push_str(&format!(
        "APP__FACEBOOK__EMAIL={}\n",
        config.facebook.email
    ));
    env_content.push_str(&format!(
        "APP__FACEBOOK__PASSWORD={}\n",
        config.facebook.password
    ));

    // Instagram
    env_content.push_str(&format!(
        "APP__INSTAGRAM__ACCOUNT_ID={}\n",
        config.instagram.account_id
    ));
    // Instagram
    env_content.push_str(&format!(
        "APP__INSTAGRAM__ACCOUNT_ID={}\n",
        config.instagram.account_id
    ));
    env_content.push_str(&format!(
        "APP__INSTAGRAM__EMAIL={}\n",
        config.instagram.email
    ));
    env_content.push_str(&format!(
        "APP__INSTAGRAM__PASSWORD={}\n",
        config.instagram.password
    ));

    // YouTube
    env_content.push_str(&format!(
        "APP__YOUTUBE__CHANNEL_ID={}\n",
        config.youtube.channel_id
    ));
    env_content.push_str(&format!(
        "APP__YOUTUBE__EMAIL={}\n",
        config.youtube.email
    ));
    env_content.push_str(&format!(
        "APP__YOUTUBE__PASSWORD={}\n",
        config.youtube.password
    ));

    // Content
    env_content.push_str(&format!(
        "APP__CONTENT__HASHTAG_COUNT={}\n",
        config.content.hashtag_count
    ));
    env_content.push_str(&format!(
        "APP__CONTENT__USE_TRENDING_HASHTAGS={}\n",
        config.content.use_trending_hashtags
    ));

    // Scheduler
    env_content.push_str(&format!(
        "APP__SCHEDULER__CHECK_INTERVAL_MINUTES={}\n",
        config.scheduler.check_interval_minutes
    ));
    env_content.push_str(&format!(
        "APP__SCHEDULER__LEGACY_POST_INTERVAL_MINUTES={}\n",
        config.scheduler.legacy_post_interval_minutes
    ));
    env_content.push_str(&format!(
        "APP__SCHEDULER__MAX_POSTS_PER_DAY={}\n",
        config.scheduler.max_posts_per_day
    ));

    // System
    env_content.push_str(&format!(
        "APP__SYSTEM__AUTOMATION_MODE={}\n",
        config.system.automation_mode
    ));
    env_content.push_str(&format!(
        "APP__SYSTEM__DEBUG={}\n",
        config.system.debug
    ));

    // Write to file
    fs::write(env_path, env_content)?;

    Ok(())
}
