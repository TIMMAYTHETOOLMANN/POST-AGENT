use crate::error::{AppError, AppResult};
use log::{error, info, warn};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::fs;
use tokio::process::Command;
use tokio::time::sleep;

/// Deployment configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentConfig {
    pub repository_url: String,
    pub branch: String,
    pub health_check_url: String,
    pub health_check_timeout_seconds: u64,
    pub rollback_timeout_seconds: u64,
    pub pre_deployment_commands: Vec<String>,
    pub post_deployment_commands: Vec<String>,
    pub backup_enabled: bool,
    pub auto_restart: bool,
}

impl Default for DeploymentConfig {
    fn default() -> Self {
        Self {
            repository_url: "https://github.com/TIMMAYTHETOOLMANN/POST-AGENT.git".to_string(),
            branch: "main".to_string(),
            health_check_url: "http://localhost:8080/health".to_string(),
            health_check_timeout_seconds: 60,
            rollback_timeout_seconds: 300,
            pre_deployment_commands: vec!["cargo test".to_string()],
            post_deployment_commands: vec![],
            backup_enabled: true,
            auto_restart: true,
        }
    }
}

/// Deployment status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DeploymentStatus {
    Idle,
    Preparing,
    Downloading,
    Building,
    Testing,
    Backing,
    Deploying,
    HealthChecking,
    Success,
    Failed,
    RollingBack,
    RolledBack,
}

/// Deployment result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentResult {
    pub id: String,
    pub status: DeploymentStatus,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub commit_hash: Option<String>,
    pub old_commit_hash: Option<String>,
    pub error_message: Option<String>,
    pub logs: Vec<String>,
    pub health_check_passed: bool,
    pub rollback_performed: bool,
}

/// Blue-green deployment manager
pub struct BlueGreenDeployment {
    config: DeploymentConfig,
    current_deployment: String, // "blue" or "green"
    blue_path: PathBuf,
    green_path: PathBuf,
    deployment_root: PathBuf,
    status: DeploymentStatus,
    current_result: Option<DeploymentResult>,
}

impl BlueGreenDeployment {
    /// Create a new blue-green deployment manager
    pub fn new(config: DeploymentConfig, deployment_root: &str) -> Self {
        let deployment_root = PathBuf::from(deployment_root);
        let blue_path = deployment_root.join("blue");
        let green_path = deployment_root.join("green");

        Self {
            config,
            current_deployment: "blue".to_string(),
            blue_path,
            green_path,
            deployment_root,
            status: DeploymentStatus::Idle,
            current_result: None,
        }
    }

    /// Initialize deployment directories
    pub async fn initialize(&mut self) -> AppResult<()> {
        // Create deployment directories
        fs::create_dir_all(&self.blue_path).await.map_err(|e| {
            AppError::GenericError(format!("Failed to create blue directory: {}", e))
        })?;

        fs::create_dir_all(&self.green_path).await.map_err(|e| {
            AppError::GenericError(format!("Failed to create green directory: {}", e))
        })?;

        // Initialize with current codebase if directories are empty
        if self.is_directory_empty(&self.blue_path).await? {
            self.setup_initial_deployment(&self.blue_path).await?;
        }

        info!("Blue-green deployment manager initialized");
        Ok(())
    }

    /// Check if directory is empty
    async fn is_directory_empty(&self, path: &PathBuf) -> AppResult<bool> {
        let mut entries = fs::read_dir(path).await.map_err(|e| {
            AppError::GenericError(format!("Failed to read directory: {}", e))
        })?;

        Ok(entries.next_entry().await.map_err(|e| {
            AppError::GenericError(format!("Failed to check directory contents: {}", e))
        })?.is_none())
    }

    /// Setup initial deployment from current working directory
    async fn setup_initial_deployment(&self, target_path: &PathBuf) -> AppResult<()> {
        info!("Setting up initial deployment in {:?}", target_path);

        // Copy current application to deployment directory
        let output = Command::new("cp")
            .args(&["-r", ".", &target_path.to_string_lossy()])
            .output()
            .await
            .map_err(|e| AppError::GenericError(format!("Failed to copy files: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::GenericError(format!(
                "Failed to copy files: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        info!("Initial deployment setup completed");
        Ok(())
    }

    /// Perform zero-downtime deployment
    pub async fn deploy(&mut self) -> AppResult<DeploymentResult> {
        let deployment_id = format!("deploy_{}", SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        info!("Starting deployment: {}", deployment_id);

        let mut result = DeploymentResult {
            id: deployment_id.clone(),
            status: DeploymentStatus::Preparing,
            start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            end_time: None,
            commit_hash: None,
            old_commit_hash: None,
            error_message: None,
            logs: Vec::new(),
            health_check_passed: false,
            rollback_performed: false,
        };

        self.status = DeploymentStatus::Preparing;
        self.current_result = Some(result.clone());

        // Get current commit hash for rollback
        result.old_commit_hash = self.get_current_commit_hash().await.ok();

        // Determine target deployment slot (opposite of current)
        let target_deployment_name = if self.current_deployment == "blue" {
            "green"
        } else {
            "blue"
        };

        let target_path = if target_deployment_name == "green" {
            self.green_path.clone()
        } else {
            self.blue_path.clone()
        };

        result.logs.push(format!("Target deployment path: {:?}", target_path));

        // Step 1: Backup current deployment
        if self.config.backup_enabled {
            self.status = DeploymentStatus::Backing;
            result.status = DeploymentStatus::Backing;
            
            if let Err(e) = self.backup_current_deployment().await {
                result.status = DeploymentStatus::Failed;
                result.error_message = Some(format!("Backup failed: {}", e));
                result.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
                return Ok(result);
            }
            
            result.logs.push("Current deployment backed up".to_string());
        }

        // Step 2: Download new code
        self.status = DeploymentStatus::Downloading;
        result.status = DeploymentStatus::Downloading;
        
        match self.download_new_code(&target_path).await {
            Ok(commit_hash) => {
                result.commit_hash = Some(commit_hash);
                result.logs.push("New code downloaded successfully".to_string());
            }
            Err(e) => {
                result.status = DeploymentStatus::Failed;
                result.error_message = Some(format!("Download failed: {}", e));
                result.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
                return Ok(result);
            }
        }

        // Step 3: Build new code
        self.status = DeploymentStatus::Building;
        result.status = DeploymentStatus::Building;
        
        if let Err(e) = self.build_new_code(&target_path).await {
            result.status = DeploymentStatus::Failed;
            result.error_message = Some(format!("Build failed: {}", e));
            result.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
            return Ok(result);
        }
        
        result.logs.push("New code built successfully".to_string());

        // Step 4: Run tests
        self.status = DeploymentStatus::Testing;
        result.status = DeploymentStatus::Testing;
        
        if let Err(e) = self.run_pre_deployment_tests(&target_path).await {
            result.status = DeploymentStatus::Failed;
            result.error_message = Some(format!("Tests failed: {}", e));
            result.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
            return Ok(result);
        }
        
        result.logs.push("Pre-deployment tests passed".to_string());

        // Step 5: Deploy (switch symlinks or ports)
        self.status = DeploymentStatus::Deploying;
        result.status = DeploymentStatus::Deploying;
        
        if let Err(e) = self.switch_deployment(&target_path).await {
            result.status = DeploymentStatus::Failed;
            result.error_message = Some(format!("Deployment switch failed: {}", e));
            result.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
            return Ok(result);
        }
        
        result.logs.push("Deployment switched successfully".to_string());

        // Step 6: Health check
        self.status = DeploymentStatus::HealthChecking;
        result.status = DeploymentStatus::HealthChecking;
        
        let health_passed = self.perform_health_check().await.unwrap_or(false);
        result.health_check_passed = health_passed;
        
        if health_passed {
            result.logs.push("Health check passed".to_string());
        } else {
            result.logs.push("Health check failed".to_string());
            
            // Step 7: Rollback if health check failed
            self.status = DeploymentStatus::RollingBack;
            result.status = DeploymentStatus::RollingBack;
            
            if let Err(e) = self.rollback_deployment().await {
                result.error_message = Some(format!("Rollback failed: {}", e));
                result.logs.push(format!("Rollback failed: {}", e));
            } else {
                result.rollback_performed = true;
                result.logs.push("Deployment rolled back successfully".to_string());
            }
            
            result.status = DeploymentStatus::RolledBack;
            result.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
            return Ok(result);
        }

        // Step 8: Run post-deployment commands
        if let Err(e) = self.run_post_deployment_commands(&target_path).await {
            warn!("Post-deployment commands failed: {}", e);
            result.logs.push(format!("Post-deployment commands failed: {}", e));
        } else {
            result.logs.push("Post-deployment commands completed".to_string());
        }

        // Success!
        self.status = DeploymentStatus::Success;
        result.status = DeploymentStatus::Success;
        result.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
        result.logs.push("Deployment completed successfully".to_string());

        info!("Deployment {} completed successfully", deployment_id);
        Ok(result)
    }

    /// Get current commit hash
    async fn get_current_commit_hash(&self) -> AppResult<String> {
        let output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .output()
            .await
            .map_err(|e| AppError::GenericError(format!("Failed to get commit hash: {}", e)))?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
        } else {
            Err(AppError::GenericError("Failed to get commit hash".to_string()))
        }
    }

    /// Backup current deployment
    async fn backup_current_deployment(&self) -> AppResult<()> {
        info!("Backing up current deployment");
        
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        let backup_path = self.deployment_root.join(format!("backup_{}", timestamp));

        let current_path = if self.current_deployment == "blue" {
            &self.blue_path
        } else {
            &self.green_path
        };

        let output = Command::new("cp")
            .args(&["-r", &current_path.to_string_lossy(), &backup_path.to_string_lossy()])
            .output()
            .await
            .map_err(|e| AppError::GenericError(format!("Failed to backup deployment: {}", e)))?;

        if output.status.success() {
            info!("Backup created at {:?}", backup_path);
            Ok(())
        } else {
            Err(AppError::GenericError(format!(
                "Backup failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    /// Download new code to target directory
    async fn download_new_code(&self, target_path: &PathBuf) -> AppResult<String> {
        info!("Downloading new code to {:?}", target_path);

        // Clean target directory
        if target_path.exists() {
            fs::remove_dir_all(target_path).await.map_err(|e| {
                AppError::GenericError(format!("Failed to clean target directory: {}", e))
            })?;
        }

        // Clone repository
        let output = Command::new("git")
            .args(&[
                "clone",
                "--branch", &self.config.branch,
                "--depth", "1",
                &self.config.repository_url,
                &target_path.to_string_lossy(),
            ])
            .output()
            .await
            .map_err(|e| AppError::GenericError(format!("Failed to clone repository: {}", e)))?;

        if !output.status.success() {
            return Err(AppError::GenericError(format!(
                "Git clone failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        // Get commit hash
        let commit_output = Command::new("git")
            .args(&["rev-parse", "HEAD"])
            .current_dir(target_path)
            .output()
            .await
            .map_err(|e| AppError::GenericError(format!("Failed to get commit hash: {}", e)))?;

        if commit_output.status.success() {
            let commit_hash = String::from_utf8_lossy(&commit_output.stdout).trim().to_string();
            info!("Downloaded code with commit hash: {}", commit_hash);
            Ok(commit_hash)
        } else {
            Err(AppError::GenericError("Failed to get commit hash".to_string()))
        }
    }

    /// Build new code
    async fn build_new_code(&self, target_path: &PathBuf) -> AppResult<()> {
        info!("Building new code");

        let output = Command::new("cargo")
            .args(&["build", "--release"])
            .current_dir(target_path)
            .output()
            .await
            .map_err(|e| AppError::GenericError(format!("Failed to build code: {}", e)))?;

        if output.status.success() {
            info!("Build completed successfully");
            Ok(())
        } else {
            Err(AppError::GenericError(format!(
                "Build failed: {}",
                String::from_utf8_lossy(&output.stderr)
            )))
        }
    }

    /// Run pre-deployment tests
    async fn run_pre_deployment_tests(&self, target_path: &PathBuf) -> AppResult<()> {
        info!("Running pre-deployment tests");

        for command in &self.config.pre_deployment_commands {
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let mut cmd = Command::new(parts[0]);
            if parts.len() > 1 {
                cmd.args(&parts[1..]);
            }

            let output = cmd
                .current_dir(target_path)
                .output()
                .await
                .map_err(|e| AppError::GenericError(format!("Failed to run command '{}': {}", command, e)))?;

            if !output.status.success() {
                return Err(AppError::GenericError(format!(
                    "Pre-deployment command '{}' failed: {}",
                    command,
                    String::from_utf8_lossy(&output.stderr)
                )));
            }
        }

        info!("All pre-deployment tests passed");
        Ok(())
    }

    /// Switch deployment (blue/green)
    async fn switch_deployment(&mut self, target_path: &PathBuf) -> AppResult<()> {
        info!("Switching deployment");

        // Create symlink to current deployment
        let current_link = self.deployment_root.join("current");
        
        // Remove existing symlink
        if current_link.exists() {
            fs::remove_file(&current_link).await.map_err(|e| {
                AppError::GenericError(format!("Failed to remove current symlink: {}", e))
            })?;
        }

        // Create new symlink
        #[cfg(unix)]
        {
            use std::os::unix::fs::symlink;
            let target_path_clone = target_path.clone();
            let current_link_clone = current_link.clone();
            tokio::task::spawn_blocking(move || {
                symlink(target_path_clone, current_link_clone)
            }).await.map_err(|e| {
                AppError::GenericError(format!("Failed to create symlink: {}", e))
            })?.map_err(|e| {
                AppError::GenericError(format!("Failed to create symlink: {}", e))
            })?;
        }

        // Update current deployment tracking
        self.current_deployment = if self.current_deployment == "blue" {
            "green".to_string()
        } else {
            "blue".to_string()
        };

        info!("Deployment switched to {}", self.current_deployment);
        Ok(())
    }

    /// Perform health check
    async fn perform_health_check(&self) -> AppResult<bool> {
        info!("Performing health check");

        for attempt in 1..=3 {
            // Simple health check using curl or built-in health endpoint
            let output = Command::new("curl")
                .args(&[
                    "-f",
                    "-s",
                    "--max-time", &self.config.health_check_timeout_seconds.to_string(),
                    &self.config.health_check_url,
                ])
                .output()
                .await;

            match output {
                Ok(output) if output.status.success() => {
                    info!("Health check passed on attempt {}", attempt);
                    return Ok(true);
                }
                Ok(_) => {
                    warn!("Health check failed on attempt {}", attempt);
                }
                Err(e) => {
                    warn!("Health check error on attempt {}: {}", attempt, e);
                }
            }

            if attempt < 3 {
                sleep(Duration::from_secs(5)).await;
            }
        }

        error!("Health check failed after 3 attempts");
        Ok(false)
    }

    /// Rollback deployment
    async fn rollback_deployment(&mut self) -> AppResult<()> {
        info!("Rolling back deployment");

        // Switch back to previous deployment
        let previous_path = if self.current_deployment == "blue" {
            self.green_path.clone()
        } else {
            self.blue_path.clone()
        };

        // Switch deployment back
        self.switch_deployment(&previous_path).await?;

        // Perform health check on rollback
        if !self.perform_health_check().await.unwrap_or(false) {
            warn!("Health check failed even after rollback");
        }

        info!("Rollback completed");
        Ok(())
    }

    /// Run post-deployment commands
    async fn run_post_deployment_commands(&self, target_path: &PathBuf) -> AppResult<()> {
        info!("Running post-deployment commands");

        for command in &self.config.post_deployment_commands {
            let parts: Vec<&str> = command.split_whitespace().collect();
            if parts.is_empty() {
                continue;
            }

            let mut cmd = Command::new(parts[0]);
            if parts.len() > 1 {
                cmd.args(&parts[1..]);
            }

            let output = cmd
                .current_dir(target_path)
                .output()
                .await
                .map_err(|e| AppError::GenericError(format!("Failed to run command '{}': {}", command, e)))?;

            if !output.status.success() {
                warn!("Post-deployment command '{}' failed: {}", command, String::from_utf8_lossy(&output.stderr));
            }
        }

        info!("Post-deployment commands completed");
        Ok(())
    }

    /// Get current deployment status
    pub fn get_status(&self) -> DeploymentStatus {
        self.status.clone()
    }

    /// Get current deployment result
    pub fn get_current_result(&self) -> Option<DeploymentResult> {
        self.current_result.clone()
    }

    /// Check if deployment is in progress
    pub fn is_deploying(&self) -> bool {
        matches!(self.status, 
            DeploymentStatus::Preparing |
            DeploymentStatus::Downloading |
            DeploymentStatus::Building |
            DeploymentStatus::Testing |
            DeploymentStatus::Backing |
            DeploymentStatus::Deploying |
            DeploymentStatus::HealthChecking |
            DeploymentStatus::RollingBack
        )
    }
}

/// Global deployment manager
static mut DEPLOYMENT_MANAGER: Option<BlueGreenDeployment> = None;
static DEPLOYMENT_MANAGER_INIT: std::sync::Once = std::sync::Once::new();

/// Get the global deployment manager
pub fn get_deployment_manager() -> &'static mut BlueGreenDeployment {
    unsafe {
        DEPLOYMENT_MANAGER_INIT.call_once(|| {
            let config = DeploymentConfig::default();
            DEPLOYMENT_MANAGER = Some(BlueGreenDeployment::new(config, "deployments"));
        });
        DEPLOYMENT_MANAGER.as_mut().unwrap()
    }
}

/// Initialize zero-downtime deployment
pub async fn init_zero_downtime_deployment() -> AppResult<()> {
    let manager = get_deployment_manager();
    manager.initialize().await?;
    info!("Zero-downtime deployment system initialized");
    Ok(())
}

/// Trigger a new deployment
pub async fn trigger_deployment() -> AppResult<DeploymentResult> {
    let manager = get_deployment_manager();
    
    if manager.is_deploying() {
        return Err(AppError::GenericError("Deployment already in progress".to_string()));
    }
    
    manager.deploy().await
}