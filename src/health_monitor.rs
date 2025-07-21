use crate::error::{AppError, AppResult};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::Mutex;
use tokio::time::sleep;

/// Health status of a component
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Critical,
    Unknown,
}

/// Health check result for a component
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheck {
    pub component: String,
    pub status: HealthStatus,
    pub message: String,
    pub timestamp: u64,
    pub response_time_ms: u64,
    pub metadata: HashMap<String, String>,
}

impl HealthCheck {
    pub fn new(component: &str, status: HealthStatus, message: &str) -> Self {
        Self {
            component: component.to_string(),
            status,
            message: message.to_string(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            response_time_ms: 0,
            metadata: HashMap::new(),
        }
    }

    pub fn with_response_time(mut self, response_time_ms: u64) -> Self {
        self.response_time_ms = response_time_ms;
        self
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }
}

/// Health monitoring configuration
#[derive(Debug, Clone)]
pub struct HealthConfig {
    pub check_interval_seconds: u64,
    pub failure_threshold: u32,
    pub warning_threshold: u32,
    pub restart_cooldown_minutes: u64,
    pub max_restart_attempts: u32,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            check_interval_seconds: 30,
            failure_threshold: 3,
            warning_threshold: 2,
            restart_cooldown_minutes: 5,
            max_restart_attempts: 5,
        }
    }
}

/// Component restart statistics
#[derive(Debug, Clone, Default)]
pub struct RestartStats {
    pub total_restarts: u32,
    pub last_restart_time: Option<u64>,
    pub consecutive_failures: u32,
    pub last_success_time: Option<u64>,
}

/// Health monitoring system
pub struct HealthMonitor {
    config: HealthConfig,
    health_checks: Arc<Mutex<HashMap<String, HealthCheck>>>,
    restart_stats: Arc<Mutex<HashMap<String, RestartStats>>>,
    is_running: Arc<Mutex<bool>>,
}

impl HealthMonitor {
    /// Create a new health monitor
    pub fn new(config: HealthConfig) -> Self {
        Self {
            config,
            health_checks: Arc::new(Mutex::new(HashMap::new())),
            restart_stats: Arc::new(Mutex::new(HashMap::new())),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Start the health monitoring loop
    pub async fn start(&self) -> AppResult<()> {
        let mut is_running = self.is_running.lock().await;
        if *is_running {
            return Ok(());
        }
        *is_running = true;
        drop(is_running);

        info!("Starting health monitor with {} second intervals", self.config.check_interval_seconds);

        let health_checks = Arc::clone(&self.health_checks);
        let restart_stats = Arc::clone(&self.restart_stats);
        let is_running = Arc::clone(&self.is_running);
        let config = self.config.clone();

        tokio::spawn(async move {
            while *is_running.lock().await {
                // Perform health checks
                if let Err(e) = Self::perform_health_checks(&health_checks, &restart_stats, &config).await {
                    error!("Health check failed: {}", e);
                }

                sleep(Duration::from_secs(config.check_interval_seconds)).await;
            }
            info!("Health monitor stopped");
        });

        Ok(())
    }

    /// Stop the health monitoring
    pub async fn stop(&self) {
        let mut is_running = self.is_running.lock().await;
        *is_running = false;
        info!("Health monitor stopping");
    }

    /// Perform health checks for all components
    async fn perform_health_checks(
        health_checks: &Arc<Mutex<HashMap<String, HealthCheck>>>,
        restart_stats: &Arc<Mutex<HashMap<String, RestartStats>>>,
        config: &HealthConfig,
    ) -> AppResult<()> {
        let start_time = SystemTime::now();

        // Check browser automation health
        let browser_health = Self::check_browser_health().await;
        Self::update_health_check(health_checks, browser_health.clone()).await;

        // Check session health
        let session_health = Self::check_session_health().await;
        Self::update_health_check(health_checks, session_health.clone()).await;

        // Check TikTok connectivity
        let tiktok_health = Self::check_tiktok_health().await;
        Self::update_health_check(health_checks, tiktok_health.clone()).await;

        // Check platform connectivity
        let platform_health = Self::check_platform_health().await;
        Self::update_health_check(health_checks, platform_health.clone()).await;

        // Check system resources
        let system_health = Self::check_system_health().await;
        Self::update_health_check(health_checks, system_health.clone()).await;

        // Handle component failures and restarts
        Self::handle_component_failures(health_checks, restart_stats, config).await?;

        let duration = start_time.elapsed().unwrap().as_millis();
        debug!("Health check cycle completed in {}ms", duration);

        Ok(())
    }

    /// Check browser automation health
    async fn check_browser_health() -> HealthCheck {
        // Test basic browser functionality
        match crate::browser::BrowserManager::new() {
            Ok(_) => HealthCheck::new("browser", HealthStatus::Healthy, "Browser automation available"),
            Err(e) => HealthCheck::new("browser", HealthStatus::Critical, &format!("Browser automation failed: {}", e)),
        }
    }

    /// Check session health
    async fn check_session_health() -> HealthCheck {
        match crate::session_manager::get_session_manager().await.get_stats() {
            stats => {
                let valid = stats.get("valid").unwrap_or(&0);
                let total = stats.get("total").unwrap_or(&0);
                
                if *total == 0 {
                    HealthCheck::new("sessions", HealthStatus::Warning, "No sessions found")
                } else if *valid == 0 {
                    HealthCheck::new("sessions", HealthStatus::Critical, "No valid sessions")
                } else {
                    HealthCheck::new("sessions", HealthStatus::Healthy, &format!("{}/{} sessions valid", valid, total))
                        .with_metadata("valid_sessions", &valid.to_string())
                        .with_metadata("total_sessions", &total.to_string())
                }
            }
        }
    }

    /// Check TikTok connectivity
    async fn check_tiktok_health() -> HealthCheck {
        // Simple connectivity test
        match tokio::process::Command::new("ping")
            .args(&["-c", "1", "-W", "5", "www.tiktok.com"])
            .output()
            .await
        {
            Ok(output) => {
                if output.status.success() {
                    HealthCheck::new("tiktok", HealthStatus::Healthy, "TikTok connectivity OK")
                } else {
                    HealthCheck::new("tiktok", HealthStatus::Warning, "TikTok connectivity issues")
                }
            }
            Err(e) => HealthCheck::new("tiktok", HealthStatus::Critical, &format!("TikTok connectivity test failed: {}", e)),
        }
    }

    /// Check platform connectivity
    async fn check_platform_health() -> HealthCheck {
        let platforms = vec!["facebook.com", "instagram.com", "youtube.com"];
        let mut healthy_count = 0;

        for platform in &platforms {
            match tokio::process::Command::new("ping")
                .args(&["-c", "1", "-W", "3", platform])
                .output()
                .await
            {
                Ok(output) if output.status.success() => healthy_count += 1,
                _ => {}
            }
        }

        match healthy_count {
            3 => HealthCheck::new("platforms", HealthStatus::Healthy, "All platforms reachable"),
            2 => HealthCheck::new("platforms", HealthStatus::Warning, "Some platform connectivity issues"),
            1 => HealthCheck::new("platforms", HealthStatus::Warning, "Limited platform connectivity"),
            0 => HealthCheck::new("platforms", HealthStatus::Critical, "No platform connectivity"),
            _ => HealthCheck::new("platforms", HealthStatus::Unknown, "Unexpected connectivity state"),
        }.with_metadata("healthy_platforms", &healthy_count.to_string())
    }

    /// Check system resource health
    async fn check_system_health() -> HealthCheck {
        // Check disk space
        match std::fs::metadata(".") {
            Ok(_) => {
                // Check if we can write to the current directory
                match tokio::fs::write("health_check_temp", "test").await {
                    Ok(_) => {
                        let _ = tokio::fs::remove_file("health_check_temp").await;
                        HealthCheck::new("system", HealthStatus::Healthy, "System resources OK")
                    }
                    Err(e) => HealthCheck::new("system", HealthStatus::Critical, &format!("Filesystem write failed: {}", e)),
                }
            }
            Err(e) => HealthCheck::new("system", HealthStatus::Critical, &format!("Filesystem access failed: {}", e)),
        }
    }

    /// Update health check results
    async fn update_health_check(
        health_checks: &Arc<Mutex<HashMap<String, HealthCheck>>>,
        health_check: HealthCheck,
    ) {
        let mut checks = health_checks.lock().await;
        debug!("Health check - {}: {:?} - {}", health_check.component, health_check.status, health_check.message);
        checks.insert(health_check.component.clone(), health_check);
    }

    /// Handle component failures and trigger restarts if needed
    async fn handle_component_failures(
        health_checks: &Arc<Mutex<HashMap<String, HealthCheck>>>,
        restart_stats: &Arc<Mutex<HashMap<String, RestartStats>>>,
        config: &HealthConfig,
    ) -> AppResult<()> {
        let checks = health_checks.lock().await;
        let mut stats = restart_stats.lock().await;

        for (component, health_check) in checks.iter() {
            let component_stats = stats.entry(component.clone()).or_default();

            match health_check.status {
                HealthStatus::Critical => {
                    component_stats.consecutive_failures += 1;
                    
                    if component_stats.consecutive_failures >= config.failure_threshold {
                        if Self::should_attempt_restart(component_stats, config) {
                            warn!("Component {} has failed {} times, attempting restart", component, component_stats.consecutive_failures);
                            
                            if let Err(e) = Self::restart_component(component).await {
                                error!("Failed to restart component {}: {}", component, e);
                            } else {
                                component_stats.total_restarts += 1;
                                component_stats.last_restart_time = Some(
                                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                                );
                                component_stats.consecutive_failures = 0;
                                info!("Component {} restarted successfully", component);
                            }
                        } else {
                            error!("Component {} has exceeded restart limits", component);
                        }
                    }
                }
                HealthStatus::Healthy => {
                    component_stats.consecutive_failures = 0;
                    component_stats.last_success_time = Some(
                        SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                    );
                }
                _ => {}
            }
        }

        Ok(())
    }

    /// Check if we should attempt to restart a component
    fn should_attempt_restart(stats: &RestartStats, config: &HealthConfig) -> bool {
        // Check restart limits
        if stats.total_restarts >= config.max_restart_attempts {
            return false;
        }

        // Check cooldown period
        if let Some(last_restart) = stats.last_restart_time {
            let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
            let cooldown_seconds = config.restart_cooldown_minutes * 60;
            if now - last_restart < cooldown_seconds {
                return false;
            }
        }

        true
    }

    /// Restart a component
    async fn restart_component(component: &str) -> AppResult<()> {
        match component {
            "browser" => {
                // Clear browser cache and restart
                info!("Restarting browser component");
                // Browser restart is handled by creating new instances
                Ok(())
            }
            "sessions" => {
                // Clear and reload sessions
                info!("Restarting session component");
                let manager = crate::session_manager::get_session_manager().await;
                manager.clear_all_sessions();
                manager.load_sessions().await?;
                Ok(())
            }
            "tiktok" => {
                info!("Restarting TikTok component");
                // TikTok restart is handled by re-authentication
                Ok(())
            }
            "platforms" => {
                info!("Restarting platform components");
                // Platform restart is handled by re-authentication
                Ok(())
            }
            "system" => {
                info!("System component restart requested - manual intervention may be required");
                Ok(())
            }
            _ => Err(AppError::GenericError(format!("Unknown component: {}", component))),
        }
    }

    /// Get current health status
    pub async fn get_health_status(&self) -> HashMap<String, HealthCheck> {
        self.health_checks.lock().await.clone()
    }

    /// Get restart statistics
    pub async fn get_restart_stats(&self) -> HashMap<String, RestartStats> {
        self.restart_stats.lock().await.clone()
    }

    /// Get overall system health
    pub async fn get_overall_health(&self) -> HealthStatus {
        let checks = self.health_checks.lock().await;
        
        let mut has_critical = false;
        let mut has_warning = false;
        let mut has_unknown = false;

        for health_check in checks.values() {
            match health_check.status {
                HealthStatus::Critical => has_critical = true,
                HealthStatus::Warning => has_warning = true,
                HealthStatus::Unknown => has_unknown = true,
                HealthStatus::Healthy => {}
            }
        }

        if has_critical {
            HealthStatus::Critical
        } else if has_warning {
            HealthStatus::Warning
        } else if has_unknown {
            HealthStatus::Unknown
        } else {
            HealthStatus::Healthy
        }
    }
}

/// Global health monitor instance
static mut HEALTH_MONITOR: Option<HealthMonitor> = None;
static HEALTH_MONITOR_INIT: std::sync::Once = std::sync::Once::new();

/// Get the global health monitor instance
pub fn get_health_monitor() -> &'static HealthMonitor {
    unsafe {
        HEALTH_MONITOR_INIT.call_once(|| {
            HEALTH_MONITOR = Some(HealthMonitor::new(HealthConfig::default()));
        });
        HEALTH_MONITOR.as_ref().unwrap()
    }
}

/// Initialize and start the health monitor
pub async fn init_health_monitor() -> AppResult<()> {
    let monitor = get_health_monitor();
    monitor.start().await?;
    info!("Health monitor initialized and started");
    Ok(())
}

/// Stop the health monitor
pub async fn stop_health_monitor() {
    let monitor = get_health_monitor();
    monitor.stop().await;
}

/// Get current system health status
pub async fn get_system_health() -> HealthStatus {
    let monitor = get_health_monitor();
    monitor.get_overall_health().await
}