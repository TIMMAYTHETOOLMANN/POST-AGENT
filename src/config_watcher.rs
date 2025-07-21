use crate::config::{AppConfig, load_config, validate_config};
use crate::error::{AppError, AppResult};
use log::{debug, error, info, warn};
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::fs;
use tokio::sync::{Mutex, RwLock};
use tokio::time::sleep;

/// Configuration change event
#[derive(Debug, Clone)]
pub struct ConfigChangeEvent {
    pub timestamp: SystemTime,
    pub old_config: AppConfig,
    pub new_config: AppConfig,
    pub changed_sections: Vec<String>,
}

/// Configuration watcher that monitors for changes and reloads automatically
pub struct ConfigWatcher {
    config_path: String,
    current_config: Arc<RwLock<AppConfig>>,
    last_modified: Arc<Mutex<SystemTime>>,
    is_running: Arc<Mutex<bool>>,
    change_handlers: Arc<Mutex<Vec<Box<dyn Fn(ConfigChangeEvent) + Send + Sync>>>>,
}

impl ConfigWatcher {
    /// Create a new configuration watcher
    pub fn new(config_path: &str) -> Self {
        Self {
            config_path: config_path.to_string(),
            current_config: Arc::new(RwLock::new(AppConfig::default())),
            last_modified: Arc::new(Mutex::new(SystemTime::UNIX_EPOCH)),
            is_running: Arc::new(Mutex::new(false)),
            change_handlers: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Initialize the watcher with the current configuration
    pub async fn initialize(&self) -> AppResult<()> {
        let config = load_config()?;
        validate_config(&config)?;

        let mut current = self.current_config.write().await;
        *current = config;

        // Get initial file modification time
        if let Ok(metadata) = fs::metadata(&self.config_path).await {
            if let Ok(modified) = metadata.modified() {
                let mut last_mod = self.last_modified.lock().await;
                *last_mod = modified;
            }
        }

        info!("Configuration watcher initialized for: {}", self.config_path);
        Ok(())
    }

    /// Start watching for configuration changes
    pub async fn start_watching(&self, check_interval_seconds: u64) -> AppResult<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        info!("Starting configuration watcher with {} second intervals", check_interval_seconds);

        let config_path = self.config_path.clone();
        let current_config = Arc::clone(&self.current_config);
        let last_modified = Arc::clone(&self.last_modified);
        let is_running = Arc::clone(&self.is_running);
        let change_handlers = Arc::clone(&self.change_handlers);

        tokio::spawn(async move {
            while *is_running.lock().await {
                if let Err(e) = Self::check_for_changes(
                    &config_path,
                    &current_config,
                    &last_modified,
                    &change_handlers,
                ).await {
                    error!("Error checking for configuration changes: {}", e);
                }

                sleep(Duration::from_secs(check_interval_seconds)).await;
            }
            info!("Configuration watcher stopped");
        });

        Ok(())
    }

    /// Stop watching for configuration changes
    pub async fn stop_watching(&self) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        info!("Configuration watcher stopping");
    }

    /// Add a change handler function
    pub async fn add_change_handler<F>(&self, handler: F)
    where
        F: Fn(ConfigChangeEvent) + Send + Sync + 'static,
    {
        let mut handlers = self.change_handlers.lock().await;
        handlers.push(Box::new(handler));
        debug!("Added configuration change handler");
    }

    /// Get the current configuration
    pub async fn get_config(&self) -> AppConfig {
        self.current_config.read().await.clone()
    }

    /// Force reload the configuration
    pub async fn force_reload(&self) -> AppResult<bool> {
        Self::check_for_changes(
            &self.config_path,
            &self.current_config,
            &self.last_modified,
            &self.change_handlers,
        ).await
    }

    /// Check for configuration file changes
    async fn check_for_changes(
        config_path: &str,
        current_config: &Arc<RwLock<AppConfig>>,
        last_modified: &Arc<Mutex<SystemTime>>,
        change_handlers: &Arc<Mutex<Vec<Box<dyn Fn(ConfigChangeEvent) + Send + Sync>>>>,
    ) -> AppResult<bool> {
        // Check if file exists
        if !Path::new(config_path).exists() {
            warn!("Configuration file not found: {}", config_path);
            return Ok(false);
        }

        // Get file modification time
        let metadata = fs::metadata(config_path).await.map_err(|e| {
            AppError::GenericError(format!("Failed to read config file metadata: {}", e))
        })?;

        let modified = metadata.modified().map_err(|e| {
            AppError::GenericError(format!("Failed to get config file modification time: {}", e))
        })?;

        // Check if file has been modified
        let mut last_mod = last_modified.lock().await;
        if modified <= *last_mod {
            return Ok(false);
        }

        info!("Configuration file change detected, reloading...");

        // Load new configuration
        let new_config = match load_config() {
            Ok(config) => config,
            Err(e) => {
                error!("Failed to reload configuration: {}", e);
                return Err(e.into());
            }
        };

        // Validate new configuration
        if let Err(e) = validate_config(&new_config) {
            error!("New configuration is invalid: {}", e);
            return Err(AppError::ConfigError(format!("Invalid configuration: {}", e)));
        }

        // Compare configurations and detect changes
        let old_config = current_config.read().await.clone();
        let changed_sections = Self::detect_changes(&old_config, &new_config);

        if !changed_sections.is_empty() {
            info!("Configuration changes detected in sections: {:?}", changed_sections);

            // Update current configuration
            let mut current = current_config.write().await;
            *current = new_config.clone();
            drop(current);

            // Update last modified time
            *last_mod = modified;

            // Create change event
            let event = ConfigChangeEvent {
                timestamp: SystemTime::now(),
                old_config,
                new_config,
                changed_sections,
            };

            // Notify change handlers
            let handlers = change_handlers.lock().await;
            for handler in handlers.iter() {
                handler(event.clone());
            }

            info!("Configuration reloaded successfully");
            return Ok(true);
        }

        // Update last modified time even if no changes detected
        *last_mod = modified;
        Ok(false)
    }

    /// Detect changes between old and new configuration
    fn detect_changes(old_config: &AppConfig, new_config: &AppConfig) -> Vec<String> {
        let mut changes = Vec::new();

        // Check TikTok configuration changes
        if old_config.tiktok.username != new_config.tiktok.username
            || old_config.tiktok.email != new_config.tiktok.email
            || old_config.tiktok.password != new_config.tiktok.password
        {
            changes.push("tiktok".to_string());
        }

        // Check Facebook configuration changes
        if old_config.facebook.profile_id != new_config.facebook.profile_id
            || old_config.facebook.email != new_config.facebook.email
            || old_config.facebook.password != new_config.facebook.password
        {
            changes.push("facebook".to_string());
        }

        // Check Instagram configuration changes
        if old_config.instagram.account_id != new_config.instagram.account_id
            || old_config.instagram.email != new_config.instagram.email
            || old_config.instagram.password != new_config.instagram.password
        {
            changes.push("instagram".to_string());
        }

        // Check YouTube configuration changes
        if old_config.youtube.channel_id != new_config.youtube.channel_id
            || old_config.youtube.email != new_config.youtube.email
            || old_config.youtube.password != new_config.youtube.password
        {
            changes.push("youtube".to_string());
        }

        // Check content configuration changes
        if old_config.content.hashtag_count != new_config.content.hashtag_count
            || old_config.content.custom_hashtags != new_config.content.custom_hashtags
            || old_config.content.tagline_templates != new_config.content.tagline_templates
            || old_config.content.use_trending_hashtags != new_config.content.use_trending_hashtags
        {
            changes.push("content".to_string());
        }

        // Check scheduler configuration changes
        if old_config.scheduler.check_interval_minutes != new_config.scheduler.check_interval_minutes
            || old_config.scheduler.legacy_post_interval_minutes != new_config.scheduler.legacy_post_interval_minutes
            || old_config.scheduler.max_posts_per_day != new_config.scheduler.max_posts_per_day
        {
            changes.push("scheduler".to_string());
        }

        // Check browser configuration changes
        if old_config.browser.headless != new_config.browser.headless
            || old_config.browser.page_load_timeout != new_config.browser.page_load_timeout
            || old_config.browser.element_wait_timeout != new_config.browser.element_wait_timeout
            || old_config.browser.max_retries != new_config.browser.max_retries
            || old_config.browser.retry_delay_ms != new_config.browser.retry_delay_ms
            || old_config.browser.window_width != new_config.browser.window_width
            || old_config.browser.window_height != new_config.browser.window_height
            || old_config.browser.user_agent != new_config.browser.user_agent
        {
            changes.push("browser".to_string());
        }

        // Check system configuration changes
        if old_config.system.automation_mode != new_config.system.automation_mode
            || old_config.system.debug != new_config.system.debug
        {
            changes.push("system".to_string());
        }

        changes
    }

    /// Apply configuration changes to running components
    pub async fn apply_changes(event: &ConfigChangeEvent) -> AppResult<()> {
        info!("Applying configuration changes for sections: {:?}", event.changed_sections);

        for section in &event.changed_sections {
            match section.as_str() {
                "tiktok" => {
                    info!("Applying TikTok configuration changes");
                    // Clear TikTok sessions if credentials changed
                    let manager = crate::session_manager::get_session_manager().await;
                    manager.remove_session("tiktok");
                }
                "facebook" => {
                    info!("Applying Facebook configuration changes");
                    // Clear Facebook sessions if credentials changed
                    let manager = crate::session_manager::get_session_manager().await;
                    manager.remove_session("facebook");
                }
                "instagram" => {
                    info!("Applying Instagram configuration changes");
                    // Clear Instagram sessions if credentials changed
                    let manager = crate::session_manager::get_session_manager().await;
                    manager.remove_session("instagram");
                }
                "youtube" => {
                    info!("Applying YouTube configuration changes");
                    // Clear YouTube sessions if credentials changed
                    let manager = crate::session_manager::get_session_manager().await;
                    manager.remove_session("youtube");
                }
                "content" => {
                    info!("Applying content configuration changes");
                    // Content changes don't require immediate action
                }
                "scheduler" => {
                    info!("Applying scheduler configuration changes");
                    // Scheduler changes will be picked up on next cycle
                }
                "browser" => {
                    info!("Applying browser configuration changes");
                    // Browser changes will be applied to new browser instances
                }
                "system" => {
                    info!("Applying system configuration changes");
                    // System changes may require restart
                }
                _ => {
                    warn!("Unknown configuration section: {}", section);
                }
            }
        }

        // Save sessions after clearing invalid ones
        if let Err(e) = crate::session_manager::save_all_sessions().await {
            error!("Failed to save sessions after configuration change: {}", e);
        }

        info!("Configuration changes applied successfully");
        Ok(())
    }
}

/// Global configuration watcher instance
static mut CONFIG_WATCHER: Option<ConfigWatcher> = None;
static CONFIG_WATCHER_INIT: std::sync::Once = std::sync::Once::new();

/// Get the global configuration watcher instance
pub fn get_config_watcher() -> &'static ConfigWatcher {
    unsafe {
        CONFIG_WATCHER_INIT.call_once(|| {
            CONFIG_WATCHER = Some(ConfigWatcher::new("config.toml"));
        });
        CONFIG_WATCHER.as_ref().unwrap()
    }
}

/// Initialize and start the configuration watcher
pub async fn init_config_watcher() -> AppResult<()> {
    let watcher = get_config_watcher();
    watcher.initialize().await?;
    
    // Add default change handler
    watcher.add_change_handler(|event| {
        tokio::spawn(async move {
            if let Err(e) = ConfigWatcher::apply_changes(&event).await {
                error!("Failed to apply configuration changes: {}", e);
            }
        });
    }).await;

    watcher.start_watching(5).await?; // Check every 5 seconds
    info!("Configuration watcher initialized and started");
    Ok(())
}

/// Stop the configuration watcher
pub async fn stop_config_watcher() {
    let watcher = get_config_watcher();
    watcher.stop_watching().await;
}

/// Get the current configuration from the watcher
pub async fn get_watched_config() -> AppConfig {
    let watcher = get_config_watcher();
    watcher.get_config().await
}