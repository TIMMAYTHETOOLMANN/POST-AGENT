use crate::config::{
    AppConfig, ContentConfig, FacebookConfig, InstagramConfig, SchedulerConfig, TikTokConfig,
    YouTubeConfig,
};
use console::style;
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use std::error::Error;
use std::fs;
use toml;

/// Run the configuration wizard to set up the application
pub fn run_wizard() -> Result<AppConfig, Box<dyn Error>> {
    println!(
        "{}",
        style("TikTok Cross-posting Configuration Wizard")
            .bold()
            .cyan()
    );
    println!(
        "This wizard will help you set up your configuration for the TikTok cross-posting application."
    );
    println!("Press Ctrl+C at any time to cancel.\n");

    // Start with default config
    let mut config = AppConfig::default();

    // TikTok Configuration
    println!("\n{}", style("TikTok Configuration").bold().green());
    config.tiktok = configure_tiktok()?;

    // Facebook Configuration
    println!("\n{}", style("Facebook Configuration").bold().green());
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to configure Facebook integration?")
        .default(true)
        .interact()?
    {
        config.facebook = configure_facebook()?;
    }

    // Instagram Configuration
    println!("\n{}", style("Instagram Configuration").bold().green());
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to configure Instagram integration?")
        .default(true)
        .interact()?
    {
        config.instagram = configure_instagram()?;
    }

    // YouTube Configuration
    println!("\n{}", style("YouTube Configuration").bold().green());
    if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to configure YouTube integration?")
        .default(true)
        .interact()?
    {
        config.youtube = configure_youtube()?;
    }

    // Content Configuration
    println!("\n{}", style("Content Configuration").bold().green());
    config.content = configure_content()?;

    // Scheduler Configuration
    println!("\n{}", style("Scheduler Configuration").bold().green());
    config.scheduler = configure_scheduler()?;

    // Save configuration
    let config_path = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Where do you want to save the configuration?")
        .default("config.toml".to_string())
        .interact()?;

    save_config(&config, &config_path)?;

    println!("\n{}", style("Configuration complete!").bold().green());
    println!("Your configuration has been saved to {}", config_path);
    println!("You can now run the application with this configuration.");

    Ok(config)
}

/// Configure TikTok settings
fn configure_tiktok() -> Result<TikTokConfig, Box<dyn Error>> {
    let username = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("TikTok username")
        .default("your_tiktok_username".to_string())
        .interact()?;

    let api_key = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("TikTok API key")
        .default("your_tiktok_api_key".to_string())
        .interact()?;

    let api_secret = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("TikTok API secret")
        .default("your_tiktok_api_secret".to_string())
        .interact()?;

    let has_access_token = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you have a TikTok access token?")
        .default(false)
        .interact()?;

    let access_token = if has_access_token {
        let token = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("TikTok access token")
            .interact()?;
        Some(token)
    } else {
        None
    };

    Ok(TikTokConfig {
        username,
        api_key,
        api_secret,
        access_token,
    })
}

/// Configure Facebook settings
fn configure_facebook() -> Result<FacebookConfig, Box<dyn Error>> {
    let page_id = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Facebook page ID")
        .default("your_facebook_page_id".to_string())
        .interact()?;

    let api_key = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Facebook API key")
        .default("your_facebook_api_key".to_string())
        .interact()?;

    let api_secret = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Facebook API secret")
        .default("your_facebook_api_secret".to_string())
        .interact()?;

    let access_token = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Facebook access token")
        .default("your_facebook_access_token".to_string())
        .interact()?;

    Ok(FacebookConfig {
        page_id,
        api_key,
        api_secret,
        access_token,
    })
}

/// Configure Instagram settings
fn configure_instagram() -> Result<InstagramConfig, Box<dyn Error>> {
    let account_id = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Instagram account ID")
        .default("your_instagram_account_id".to_string())
        .interact()?;

    let api_key = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Instagram API key")
        .default("your_instagram_api_key".to_string())
        .interact()?;

    let api_secret = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Instagram API secret")
        .default("your_instagram_api_secret".to_string())
        .interact()?;

    let access_token = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Instagram access token")
        .default("your_instagram_access_token".to_string())
        .interact()?;

    Ok(InstagramConfig {
        account_id,
        api_key,
        api_secret,
        access_token,
    })
}

/// Configure YouTube settings
fn configure_youtube() -> Result<YouTubeConfig, Box<dyn Error>> {
    let channel_id = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("YouTube channel ID")
        .default("your_youtube_channel_id".to_string())
        .interact()?;

    let api_key = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("YouTube API key")
        .default("your_youtube_api_key".to_string())
        .interact()?;

    let client_id = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("YouTube client ID")
        .default("your_youtube_client_id".to_string())
        .interact()?;

    let client_secret = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("YouTube client secret")
        .default("your_youtube_client_secret".to_string())
        .interact()?;

    let refresh_token = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("YouTube refresh token")
        .default("your_youtube_refresh_token".to_string())
        .interact()?;

    Ok(YouTubeConfig {
        channel_id,
        api_key,
        client_id,
        client_secret,
        refresh_token,
    })
}

/// Configure content settings
fn configure_content() -> Result<ContentConfig, Box<dyn Error>> {
    let hashtag_count = Input::<usize>::with_theme(&ColorfulTheme::default())
        .with_prompt("Number of hashtags to include")
        .default(5)
        .interact()?;

    println!("Enter custom hashtags (comma-separated, no #):");
    let custom_hashtags_input = Input::<String>::with_theme(&ColorfulTheme::default())
        .with_prompt("Custom hashtags")
        .default("fyp,viral".to_string())
        .interact()?;

    let custom_hashtags: Vec<String> = custom_hashtags_input
        .split(',')
        .map(|s| s.trim().to_string())
        .collect();

    println!("Enter tagline templates (one per line, empty line to finish):");
    let mut tagline_templates = Vec::new();
    loop {
        let template = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Tagline template (empty to finish)")
            .allow_empty(true)
            .interact()?;

        if template.is_empty() {
            break;
        }

        tagline_templates.push(template);
    }

    if tagline_templates.is_empty() {
        tagline_templates = vec![
            "You won't believe what happens next! 😱".to_string(),
            "This changed everything for me! 🔥".to_string(),
            "Wait for it... 👀".to_string(),
        ];
    }

    let use_trending_hashtags = Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Use trending hashtags?")
        .default(true)
        .interact()?;

    Ok(ContentConfig {
        hashtag_count,
        custom_hashtags,
        tagline_templates,
        use_trending_hashtags,
    })
}

/// Configure scheduler settings
fn configure_scheduler() -> Result<SchedulerConfig, Box<dyn Error>> {
    let check_interval_minutes = Input::<u64>::with_theme(&ColorfulTheme::default())
        .with_prompt("Interval in minutes to check for new TikTok content")
        .default(5)
        .interact()?;

    let legacy_post_interval_minutes = Input::<u64>::with_theme(&ColorfulTheme::default())
        .with_prompt("Interval in minutes between posting legacy content")
        .default(10)
        .interact()?;

    let max_posts_per_day = Input::<usize>::with_theme(&ColorfulTheme::default())
        .with_prompt("Maximum number of posts per day")
        .default(24)
        .interact()?;

    Ok(SchedulerConfig {
        check_interval_minutes,
        legacy_post_interval_minutes,
        max_posts_per_day,
    })
}

/// Save configuration to a file
fn save_config(config: &AppConfig, path: &str) -> Result<(), Box<dyn Error>> {
    let toml_string = toml::to_string_pretty(config)?;
    fs::write(path, toml_string)?;
    Ok(())
}

/// Export configuration as environment variables
pub fn export_env_vars(config: &AppConfig) -> Vec<String> {
    let mut env_vars = Vec::new();

    // TikTok
    env_vars.push(format!("APP__TIKTOK__USERNAME={}", config.tiktok.username));
    env_vars.push(format!("APP__TIKTOK__API_KEY={}", config.tiktok.api_key));
    env_vars.push(format!(
        "APP__TIKTOK__API_SECRET={}",
        config.tiktok.api_secret
    ));
    if let Some(token) = &config.tiktok.access_token {
        env_vars.push(format!("APP__TIKTOK__ACCESS_TOKEN={}", token));
    }

    // Facebook
    env_vars.push(format!(
        "APP__FACEBOOK__PAGE_ID={}",
        config.facebook.page_id
    ));
    env_vars.push(format!(
        "APP__FACEBOOK__API_KEY={}",
        config.facebook.api_key
    ));
    env_vars.push(format!(
        "APP__FACEBOOK__API_SECRET={}",
        config.facebook.api_secret
    ));
    env_vars.push(format!(
        "APP__FACEBOOK__ACCESS_TOKEN={}",
        config.facebook.access_token
    ));

    // Instagram
    env_vars.push(format!(
        "APP__INSTAGRAM__ACCOUNT_ID={}",
        config.instagram.account_id
    ));
    env_vars.push(format!(
        "APP__INSTAGRAM__API_KEY={}",
        config.instagram.api_key
    ));
    env_vars.push(format!(
        "APP__INSTAGRAM__API_SECRET={}",
        config.instagram.api_secret
    ));
    env_vars.push(format!(
        "APP__INSTAGRAM__ACCESS_TOKEN={}",
        config.instagram.access_token
    ));

    // YouTube
    env_vars.push(format!(
        "APP__YOUTUBE__CHANNEL_ID={}",
        config.youtube.channel_id
    ));
    env_vars.push(format!("APP__YOUTUBE__API_KEY={}", config.youtube.api_key));
    env_vars.push(format!(
        "APP__YOUTUBE__CLIENT_ID={}",
        config.youtube.client_id
    ));
    env_vars.push(format!(
        "APP__YOUTUBE__CLIENT_SECRET={}",
        config.youtube.client_secret
    ));
    env_vars.push(format!(
        "APP__YOUTUBE__REFRESH_TOKEN={}",
        config.youtube.refresh_token
    ));

    // Content
    env_vars.push(format!(
        "APP__CONTENT__HASHTAG_COUNT={}",
        config.content.hashtag_count
    ));
    env_vars.push(format!(
        "APP__CONTENT__CUSTOM_HASHTAGS={}",
        config.content.custom_hashtags.join(",")
    ));
    env_vars.push(format!(
        "APP__CONTENT__TAGLINE_TEMPLATES={}",
        config.content.tagline_templates.join("||")
    ));
    env_vars.push(format!(
        "APP__CONTENT__USE_TRENDING_HASHTAGS={}",
        config.content.use_trending_hashtags
    ));

    // Scheduler
    env_vars.push(format!(
        "APP__SCHEDULER__CHECK_INTERVAL_MINUTES={}",
        config.scheduler.check_interval_minutes
    ));
    env_vars.push(format!(
        "APP__SCHEDULER__LEGACY_POST_INTERVAL_MINUTES={}",
        config.scheduler.legacy_post_interval_minutes
    ));
    env_vars.push(format!(
        "APP__SCHEDULER__MAX_POSTS_PER_DAY={}",
        config.scheduler.max_posts_per_day
    ));

    env_vars
}

/// Generate a .env file from configuration
pub fn generate_env_file(config: &AppConfig, path: &str) -> Result<(), Box<dyn Error>> {
    let env_vars = export_env_vars(config);
    let env_content = env_vars.join("\n");
    fs::write(path, env_content)?;
    Ok(())
}
