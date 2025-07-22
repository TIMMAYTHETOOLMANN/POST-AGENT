use crate::error::{AppError, AppResult};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

/// Content backup entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentBackup {
    pub id: String,
    pub original_path: String,
    pub backup_path: String,
    pub content_type: ContentType,
    pub size_bytes: u64,
    pub checksum: String,
    pub created_at: u64,
    pub last_verified: Option<u64>,
    pub metadata: HashMap<String, String>,
}

/// Type of content being backed up
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentType {
    Video,
    Image,
    Metadata,
    Configuration,
    Session,
    Log,
}

/// Backup operation result
#[derive(Debug, Clone)]
pub struct BackupResult {
    pub success: bool,
    pub backup_path: Option<String>,
    pub error_message: Option<String>,
    pub size_bytes: u64,
    pub duration_ms: u64,
}

/// Recovery operation result
#[derive(Debug, Clone)]
pub struct RecoveryResult {
    pub success: bool,
    pub recovered_path: Option<String>,
    pub error_message: Option<String>,
    pub content_verified: bool,
}

/// Content backup and recovery manager
pub struct ContentBackupManager {
    backup_root: PathBuf,
    index: HashMap<String, ContentBackup>,
    index_file: PathBuf,
    max_backup_size_gb: u64,
    retention_days: u64,
}

impl ContentBackupManager {
    /// Create a new content backup manager
    pub fn new(backup_root: &str, max_size_gb: u64, retention_days: u64) -> Self {
        let backup_root = PathBuf::from(backup_root);
        let index_file = backup_root.join("backup_index.json");

        Self {
            backup_root,
            index: HashMap::new(),
            index_file,
            max_backup_size_gb: max_size_gb,
            retention_days,
        }
    }

    /// Initialize the backup manager
    pub async fn initialize(&mut self) -> AppResult<()> {
        // Create backup directory structure
        self.create_backup_directories().await?;
        
        // Load existing backup index
        self.load_index().await?;
        
        // Perform cleanup of old backups
        self.cleanup_old_backups().await?;
        
        info!("Content backup manager initialized with {} entries", self.index.len());
        Ok(())
    }

    /// Create backup directory structure
    async fn create_backup_directories(&self) -> AppResult<()> {
        let directories = [
            &self.backup_root,
            &self.backup_root.join("videos"),
            &self.backup_root.join("images"),
            &self.backup_root.join("metadata"),
            &self.backup_root.join("config"),
            &self.backup_root.join("sessions"),
            &self.backup_root.join("logs"),
        ];

        for dir in directories {
            fs::create_dir_all(dir).await.map_err(|e| {
                AppError::GenericError(format!("Failed to create backup directory {:?}: {}", dir, e))
            })?;
        }

        Ok(())
    }

    /// Load backup index from disk
    async fn load_index(&mut self) -> AppResult<()> {
        if self.index_file.exists() {
            let content = fs::read_to_string(&self.index_file).await.map_err(|e| {
                AppError::GenericError(format!("Failed to read backup index: {}", e))
            })?;

            self.index = serde_json::from_str(&content).unwrap_or_else(|e| {
                warn!("Failed to parse backup index, starting fresh: {}", e);
                HashMap::new()
            });

            info!("Loaded {} backup entries from index", self.index.len());
        } else {
            info!("No existing backup index found, starting fresh");
        }

        Ok(())
    }

    /// Save backup index to disk
    async fn save_index(&self) -> AppResult<()> {
        let content = serde_json::to_string_pretty(&self.index).map_err(|e| {
            AppError::GenericError(format!("Failed to serialize backup index: {}", e))
        })?;

        fs::write(&self.index_file, content).await.map_err(|e| {
            AppError::GenericError(format!("Failed to save backup index: {}", e))
        })?;

        debug!("Saved backup index with {} entries", self.index.len());
        Ok(())
    }

    /// Backup content to the backup directory
    pub async fn backup_content(
        &mut self,
        content_id: &str,
        source_path: &str,
        content_type: ContentType,
        metadata: Option<HashMap<String, String>>,
    ) -> AppResult<BackupResult> {
        let start_time = SystemTime::now();
        
        info!("Starting backup for content: {} from {}", content_id, source_path);

        // Check if source file exists
        let source_path_buf = PathBuf::from(source_path);
        if !source_path_buf.exists() {
            return Ok(BackupResult {
                success: false,
                backup_path: None,
                error_message: Some(format!("Source file does not exist: {}", source_path)),
                size_bytes: 0,
                duration_ms: start_time.elapsed().unwrap().as_millis() as u64,
            });
        }

        // Determine backup subdirectory
        let backup_subdir = match content_type {
            ContentType::Video => "videos",
            ContentType::Image => "images",
            ContentType::Metadata => "metadata",
            ContentType::Configuration => "config",
            ContentType::Session => "sessions",
            ContentType::Log => "logs",
        };

        // Generate backup filename with timestamp
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        
        let file_extension = source_path_buf
            .extension()
            .and_then(|ext| ext.to_str())
            .unwrap_or("");
        
        let backup_filename = if file_extension.is_empty() {
            format!("{}_{}", content_id, timestamp)
        } else {
            format!("{}_{}.{}", content_id, timestamp, file_extension)
        };

        let backup_path = self.backup_root.join(backup_subdir).join(backup_filename);

        // Check available space
        if let Err(e) = self.check_available_space(&source_path_buf).await {
            return Ok(BackupResult {
                success: false,
                backup_path: None,
                error_message: Some(e.to_string()),
                size_bytes: 0,
                duration_ms: start_time.elapsed().unwrap().as_millis() as u64,
            });
        }

        // Copy file to backup location
        match fs::copy(&source_path_buf, &backup_path).await {
            Ok(bytes_copied) => {
                // Calculate checksum
                let checksum = self.calculate_checksum(&backup_path).await?;

                // Create backup entry
                let backup_entry = ContentBackup {
                    id: content_id.to_string(),
                    original_path: source_path.to_string(),
                    backup_path: backup_path.to_string_lossy().to_string(),
                    content_type,
                    size_bytes: bytes_copied,
                    checksum,
                    created_at: timestamp,
                    last_verified: Some(timestamp),
                    metadata: metadata.unwrap_or_default(),
                };

                // Add to index
                self.index.insert(content_id.to_string(), backup_entry);

                // Save index
                self.save_index().await?;

                let duration = start_time.elapsed().unwrap().as_millis() as u64;
                info!("Successfully backed up {} ({} bytes) in {}ms", 
                      content_id, bytes_copied, duration);

                Ok(BackupResult {
                    success: true,
                    backup_path: Some(backup_path.to_string_lossy().to_string()),
                    error_message: None,
                    size_bytes: bytes_copied,
                    duration_ms: duration,
                })
            }
            Err(e) => {
                error!("Failed to backup {}: {}", content_id, e);
                Ok(BackupResult {
                    success: false,
                    backup_path: None,
                    error_message: Some(e.to_string()),
                    size_bytes: 0,
                    duration_ms: start_time.elapsed().unwrap().as_millis() as u64,
                })
            }
        }
    }

    /// Recover content from backup
    pub async fn recover_content(
        &self,
        content_id: &str,
        recovery_path: Option<&str>,
    ) -> AppResult<RecoveryResult> {
        info!("Starting recovery for content: {}", content_id);

        // Find backup entry
        let backup_entry = match self.index.get(content_id) {
            Some(entry) => entry,
            None => {
                return Ok(RecoveryResult {
                    success: false,
                    recovered_path: None,
                    error_message: Some(format!("No backup found for content: {}", content_id)),
                    content_verified: false,
                });
            }
        };

        // Check if backup file exists
        let backup_path = PathBuf::from(&backup_entry.backup_path);
        if !backup_path.exists() {
            return Ok(RecoveryResult {
                success: false,
                recovered_path: None,
                error_message: Some(format!("Backup file does not exist: {}", backup_entry.backup_path)),
                content_verified: false,
            });
        }

        // Determine recovery path
        let target_path = recovery_path
            .map(|p| PathBuf::from(p))
            .unwrap_or_else(|| PathBuf::from(&backup_entry.original_path));

        // Create parent directory if needed
        if let Some(parent) = target_path.parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AppError::GenericError(format!("Failed to create recovery directory: {}", e))
            })?;
        }

        // Copy backup to recovery location
        match fs::copy(&backup_path, &target_path).await {
            Ok(_) => {
                // Verify recovered content
                let content_verified = self.verify_content(&target_path, &backup_entry.checksum).await.unwrap_or(false);

                info!("Successfully recovered {} to {}", content_id, target_path.display());

                Ok(RecoveryResult {
                    success: true,
                    recovered_path: Some(target_path.to_string_lossy().to_string()),
                    error_message: None,
                    content_verified,
                })
            }
            Err(e) => {
                error!("Failed to recover {}: {}", content_id, e);
                Ok(RecoveryResult {
                    success: false,
                    recovered_path: None,
                    error_message: Some(e.to_string()),
                    content_verified: false,
                })
            }
        }
    }

    /// Verify backup integrity
    pub async fn verify_backup(&mut self, content_id: &str) -> AppResult<bool> {
        let (backup_path, expected_checksum) = match self.index.get(content_id) {
            Some(entry) => (PathBuf::from(&entry.backup_path), entry.checksum.clone()),
            None => return Ok(false),
        };

        if !backup_path.exists() {
            warn!("Backup file missing for {}: {}", content_id, backup_path.display());
            return Ok(false);
        }

        let is_valid = self.verify_content(&backup_path, &expected_checksum).await?;
        
        if is_valid {
            if let Some(backup_entry) = self.index.get_mut(content_id) {
                backup_entry.last_verified = Some(
                    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
                );
            }
            self.save_index().await?;
            debug!("Verified backup for {}", content_id);
        } else {
            warn!("Backup verification failed for {}", content_id);
        }

        Ok(is_valid)
    }

    /// Verify all backups
    pub async fn verify_all_backups(&mut self) -> AppResult<HashMap<String, bool>> {
        info!("Starting verification of all backups");
        let mut results = HashMap::new();

        let content_ids: Vec<String> = self.index.keys().cloned().collect();
        
        for content_id in content_ids {
            let is_valid = self.verify_backup(&content_id).await.unwrap_or(false);
            results.insert(content_id, is_valid);
        }

        let valid_count = results.values().filter(|&&valid| valid).count();
        info!("Backup verification completed: {}/{} valid", valid_count, results.len());

        Ok(results)
    }

    /// Calculate checksum for a file
    async fn calculate_checksum(&self, file_path: &Path) -> AppResult<String> {
        use sha2::{Sha256, Digest};

        let content = fs::read(file_path).await.map_err(|e| {
            AppError::GenericError(format!("Failed to read file for checksum: {}", e))
        })?;

        let mut hasher = Sha256::new();
        hasher.update(&content);
        let result = hasher.finalize();

        Ok(format!("{:x}", result))
    }

    /// Verify content against checksum
    async fn verify_content(&self, file_path: &Path, expected_checksum: &str) -> AppResult<bool> {
        let actual_checksum = self.calculate_checksum(file_path).await?;
        Ok(actual_checksum == expected_checksum)
    }

    /// Check available disk space
    async fn check_available_space(&self, source_file: &Path) -> AppResult<()> {
        // Get source file size
        let metadata = fs::metadata(source_file).await.map_err(|e| {
            AppError::GenericError(format!("Failed to get file metadata: {}", e))
        })?;

        let file_size = metadata.len();

        // Simple check - ensure we have at least 2x the file size available
        // In a real implementation, you would check actual disk space
        let _required_space = file_size * 2;
        let max_backup_bytes = self.max_backup_size_gb * 1024 * 1024 * 1024;

        let current_backup_size = self.get_total_backup_size().await.unwrap_or(0);
        
        if current_backup_size + file_size > max_backup_bytes {
            return Err(AppError::GenericError(
                format!("Insufficient backup space: {} bytes needed, {} bytes available", 
                        file_size, max_backup_bytes - current_backup_size)
            ));
        }

        Ok(())
    }

    /// Get total size of all backups
    async fn get_total_backup_size(&self) -> AppResult<u64> {
        let total_size: u64 = self.index.values().map(|entry| entry.size_bytes).sum();
        Ok(total_size)
    }

    /// Cleanup old backups based on retention policy
    async fn cleanup_old_backups(&mut self) -> AppResult<()> {
        let cutoff_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() - (self.retention_days * 24 * 60 * 60);

        let mut to_remove = Vec::new();

        for (content_id, backup_entry) in &self.index {
            if backup_entry.created_at < cutoff_time {
                to_remove.push(content_id.clone());
            }
        }

        for content_id in &to_remove {
            if let Some(backup_entry) = self.index.remove(content_id) {
                let backup_path = PathBuf::from(&backup_entry.backup_path);
                if backup_path.exists() {
                    if let Err(e) = fs::remove_file(&backup_path).await {
                        warn!("Failed to remove old backup file {}: {}", backup_path.display(), e);
                    } else {
                        info!("Removed old backup: {}", content_id);
                    }
                }
            }
        }

        if !to_remove.is_empty() {
            self.save_index().await?;
            info!("Cleaned up {} old backups", to_remove.len());
        }

        Ok(())
    }

    /// List all backups
    pub fn list_backups(&self) -> Vec<&ContentBackup> {
        self.index.values().collect()
    }

    /// Get backup statistics
    pub async fn get_statistics(&self) -> AppResult<HashMap<String, serde_json::Value>> {
        let mut stats = HashMap::new();
        
        let total_backups = self.index.len();
        let total_size = self.get_total_backup_size().await.unwrap_or(0);
        
        // Count by content type
        let mut type_counts = HashMap::new();
        for backup in self.index.values() {
            let type_name = format!("{:?}", backup.content_type);
            *type_counts.entry(type_name).or_insert(0) += 1;
        }

        stats.insert("total_backups".to_string(), serde_json::json!(total_backups));
        stats.insert("total_size_bytes".to_string(), serde_json::json!(total_size));
        stats.insert("total_size_gb".to_string(), serde_json::json!(total_size as f64 / (1024.0 * 1024.0 * 1024.0)));
        stats.insert("by_content_type".to_string(), serde_json::to_value(type_counts).unwrap_or_default());
        stats.insert("backup_root".to_string(), serde_json::json!(self.backup_root.display().to_string()));
        stats.insert("max_size_gb".to_string(), serde_json::json!(self.max_backup_size_gb));
        stats.insert("retention_days".to_string(), serde_json::json!(self.retention_days));

        Ok(stats)
    }

    /// Automatic backup of critical content
    pub async fn auto_backup_critical_content(&mut self) -> AppResult<Vec<BackupResult>> {
        info!("Starting automatic backup of critical content");
        let mut results = Vec::new();

        // Backup configuration files
        let config_files = ["config.toml", ".env"];
        for config_file in &config_files {
            if Path::new(config_file).exists() {
                let result = self.backup_content(
                    &format!("config_{}", config_file.replace(".", "_")),
                    config_file,
                    ContentType::Configuration,
                    None,
                ).await?;
                results.push(result);
            }
        }

        // Backup session files
        if Path::new("data/sessions.json").exists() {
            let result = self.backup_content(
                "sessions",
                "data/sessions.json",
                ContentType::Session,
                None,
            ).await?;
            results.push(result);
        }

        // Backup downloaded content
        if let Ok(mut downloads_dir) = fs::read_dir("downloads").await {
            while let Ok(Some(entry)) = downloads_dir.next_entry().await {
                let path = entry.path();
                if path.is_file() {
                    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
                        let content_type = if filename.ends_with(".mp4") || filename.ends_with(".mov") {
                            ContentType::Video
                        } else if filename.ends_with(".jpg") || filename.ends_with(".png") {
                            ContentType::Image
                        } else {
                            ContentType::Metadata
                        };

                        let result = self.backup_content(
                            &format!("download_{}", filename),
                            &path.to_string_lossy(),
                            content_type,
                            None,
                        ).await?;
                        results.push(result);
                    }
                }
            }
        }

        let success_count = results.iter().filter(|r| r.success).count();
        info!("Automatic backup completed: {}/{} successful", success_count, results.len());

        Ok(results)
    }
}

/// Global content backup manager
static mut BACKUP_MANAGER: Option<ContentBackupManager> = None;
static BACKUP_MANAGER_INIT: std::sync::Once = std::sync::Once::new();

/// Get the global content backup manager
pub async fn get_backup_manager() -> &'static mut ContentBackupManager {
    unsafe {
        BACKUP_MANAGER_INIT.call_once(|| {
            BACKUP_MANAGER = Some(ContentBackupManager::new("backups", 10, 30)); // 10GB, 30 days
        });
        BACKUP_MANAGER.as_mut().unwrap()
    }
}

/// Initialize content backup system
pub async fn init_content_backup() -> AppResult<()> {
    let manager = get_backup_manager().await;
    manager.initialize().await?;
    info!("Content backup system initialized");
    Ok(())
}

/// Perform automatic backup of critical content
pub async fn auto_backup_critical() -> AppResult<Vec<BackupResult>> {
    let manager = get_backup_manager().await;
    manager.auto_backup_critical_content().await
}