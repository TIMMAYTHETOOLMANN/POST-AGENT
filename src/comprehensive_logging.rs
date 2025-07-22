use crate::error::{AppError, AppResult};
use log::{debug, error, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncWriteExt, BufWriter};
use tokio::sync::RwLock;

/// Log level for custom logging
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum LogLevel {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Critical,
}

/// Log entry structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogEntry {
    pub timestamp: u64,
    pub level: LogLevel,
    pub component: String,
    pub message: String,
    pub metadata: HashMap<String, String>,
    pub session_id: Option<String>,
    pub request_id: Option<String>,
}

impl LogEntry {
    pub fn new(level: LogLevel, component: &str, message: &str) -> Self {
        Self {
            timestamp: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            level,
            component: component.to_string(),
            message: message.to_string(),
            metadata: HashMap::new(),
            session_id: None,
            request_id: None,
        }
    }

    pub fn with_metadata(mut self, key: &str, value: &str) -> Self {
        self.metadata.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_session_id(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn with_request_id(mut self, request_id: &str) -> Self {
        self.request_id = Some(request_id.to_string());
        self
    }
}

/// Debug session for tracking related operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DebugSession {
    pub id: String,
    pub name: String,
    pub start_time: u64,
    pub end_time: Option<u64>,
    pub logs: Vec<LogEntry>,
    pub metrics: HashMap<String, f64>,
    pub screenshots: Vec<String>,
    pub artifacts: Vec<String>,
}

impl DebugSession {
    pub fn new(name: &str) -> Self {
        let id = format!("session_{}_{}", 
            name.replace(" ", "_").to_lowercase(),
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );

        Self {
            id: id.clone(),
            name: name.to_string(),
            start_time: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            end_time: None,
            logs: Vec::new(),
            metrics: HashMap::new(),
            screenshots: Vec::new(),
            artifacts: Vec::new(),
        }
    }

    pub fn add_log(&mut self, mut entry: LogEntry) {
        entry.session_id = Some(self.id.clone());
        self.logs.push(entry);
    }

    pub fn add_metric(&mut self, name: &str, value: f64) {
        self.metrics.insert(name.to_string(), value);
    }

    pub fn add_screenshot(&mut self, path: &str) {
        self.screenshots.push(path.to_string());
    }

    pub fn add_artifact(&mut self, path: &str) {
        self.artifacts.push(path.to_string());
    }

    pub fn end_session(&mut self) {
        self.end_time = Some(SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs());
    }

    pub fn duration_seconds(&self) -> u64 {
        let end_time = self.end_time.unwrap_or_else(|| 
            SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs()
        );
        end_time - self.start_time
    }
}

/// Comprehensive logging and debugging system
pub struct LoggingSystem {
    log_directory: PathBuf,
    max_log_files: usize,
    max_log_size_mb: u64,
    current_log_file: Option<BufWriter<File>>,
    sessions: Arc<RwLock<HashMap<String, DebugSession>>>,
    metrics: Arc<RwLock<HashMap<String, f64>>>,
    debug_enabled: bool,
}

impl LoggingSystem {
    /// Create a new logging system
    pub fn new(log_directory: &str, max_files: usize, max_size_mb: u64, debug_enabled: bool) -> Self {
        Self {
            log_directory: PathBuf::from(log_directory),
            max_log_files: max_files,
            max_log_size_mb: max_size_mb,
            current_log_file: None,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(HashMap::new())),
            debug_enabled,
        }
    }

    /// Initialize the logging system
    pub async fn initialize(&mut self) -> AppResult<()> {
        // Create log directory
        tokio::fs::create_dir_all(&self.log_directory).await.map_err(|e| {
            AppError::GenericError(format!("Failed to create log directory: {}", e))
        })?;

        // Create initial log file
        self.rotate_log_file().await?;

        info!("Logging system initialized at {:?}", self.log_directory);
        Ok(())
    }

    /// Log an entry
    pub async fn log(&mut self, entry: LogEntry) -> AppResult<()> {
        // Write to structured log file
        if let Some(ref mut writer) = self.current_log_file {
            let log_line = serde_json::to_string(&entry).map_err(|e| {
                AppError::GenericError(format!("Failed to serialize log entry: {}", e))
            })?;

            writer.write_all(log_line.as_bytes()).await.map_err(|e| {
                AppError::GenericError(format!("Failed to write log entry: {}", e))
            })?;
            writer.write_all(b"\n").await.map_err(|e| {
                AppError::GenericError(format!("Failed to write newline: {}", e))
            })?;
            writer.flush().await.map_err(|e| {
                AppError::GenericError(format!("Failed to flush log: {}", e))
            })?;
        }

        // Also log to standard logger based on level
        match entry.level {
            LogLevel::Trace => debug!("[{}] {}", entry.component, entry.message),
            LogLevel::Debug => debug!("[{}] {}", entry.component, entry.message),
            LogLevel::Info => info!("[{}] {}", entry.component, entry.message),
            LogLevel::Warn => warn!("[{}] {}", entry.component, entry.message),
            LogLevel::Error => error!("[{}] {}", entry.component, entry.message),
            LogLevel::Critical => error!("[CRITICAL] [{}] {}", entry.component, entry.message),
        }

        // Check if log rotation is needed
        self.check_log_rotation().await?;

        Ok(())
    }

    /// Create a new debug session
    pub async fn create_debug_session(&self, name: &str) -> String {
        let session = DebugSession::new(name);
        let session_id = session.id.clone();
        
        let mut sessions = self.sessions.write().await;
        sessions.insert(session_id.clone(), session);
        
        info!("Created debug session: {}", session_id);
        session_id
    }

    /// Add log to a debug session
    pub async fn log_to_session(&self, session_id: &str, entry: LogEntry) -> AppResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.add_log(entry);
        }
        Ok(())
    }

    /// Add metric to a debug session
    pub async fn add_session_metric(&self, session_id: &str, name: &str, value: f64) -> AppResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.add_metric(name, value);
        }
        Ok(())
    }

    /// Add screenshot to a debug session
    pub async fn add_session_screenshot(&self, session_id: &str, screenshot_path: &str) -> AppResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.add_screenshot(screenshot_path);
        }
        Ok(())
    }

    /// End a debug session and save it
    pub async fn end_debug_session(&self, session_id: &str) -> AppResult<()> {
        let mut sessions = self.sessions.write().await;
        if let Some(session) = sessions.get_mut(session_id) {
            session.end_session();
            
            // Save session to file
            let session_file = self.log_directory.join(format!("{}.json", session_id));
            let session_json = serde_json::to_string_pretty(session).map_err(|e| {
                AppError::GenericError(format!("Failed to serialize debug session: {}", e))
            })?;
            
            tokio::fs::write(session_file, session_json).await.map_err(|e| {
                AppError::GenericError(format!("Failed to save debug session: {}", e))
            })?;
            
            info!("Debug session {} ended and saved (duration: {}s)", 
                  session_id, session.duration_seconds());
        }
        Ok(())
    }

    /// Record a global metric
    pub async fn record_metric(&self, name: &str, value: f64) {
        let mut metrics = self.metrics.write().await;
        metrics.insert(name.to_string(), value);
        debug!("Recorded metric: {} = {}", name, value);
    }

    /// Get all recorded metrics
    pub async fn get_metrics(&self) -> HashMap<String, f64> {
        self.metrics.read().await.clone()
    }

    /// Generate debug report for a time period
    pub async fn generate_debug_report(&self, hours: u64) -> AppResult<DebugReport> {
        let cutoff_time = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs() - (hours * 3600);
        
        let sessions = self.sessions.read().await;
        let recent_sessions: Vec<DebugSession> = sessions.values()
            .filter(|s| s.start_time >= cutoff_time)
            .cloned()
            .collect();

        let metrics = self.get_metrics().await;

        let report = DebugReport {
            generated_at: SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs(),
            time_period_hours: hours,
            total_sessions: recent_sessions.len(),
            completed_sessions: recent_sessions.iter().filter(|s| s.end_time.is_some()).count(),
            failed_sessions: recent_sessions.iter()
                .filter(|s| s.logs.iter().any(|l| matches!(l.level, LogLevel::Error | LogLevel::Critical)))
                .count(),
            average_session_duration: if recent_sessions.is_empty() { 0.0 } else {
                recent_sessions.iter().map(|s| s.duration_seconds() as f64).sum::<f64>() / recent_sessions.len() as f64
            },
            total_errors: recent_sessions.iter()
                .map(|s| s.logs.iter().filter(|l| matches!(l.level, LogLevel::Error | LogLevel::Critical)).count())
                .sum(),
            metrics: metrics.clone(),
            recent_sessions,
        };

        Ok(report)
    }

    /// Rotate log file if needed
    async fn check_log_rotation(&mut self) -> AppResult<()> {
        if let Some(ref mut writer) = self.current_log_file {
            // Check file size
            let current_file_path = self.get_current_log_file_path();
            if let Ok(metadata) = tokio::fs::metadata(&current_file_path).await {
                if metadata.len() > self.max_log_size_mb * 1024 * 1024 {
                    info!("Log file size limit reached, rotating");
                    self.rotate_log_file().await?;
                }
            }
        }
        Ok(())
    }

    /// Rotate to a new log file
    async fn rotate_log_file(&mut self) -> AppResult<()> {
        // Close current file
        if let Some(_writer) = self.current_log_file.take() {
            drop(_writer);
        }

        // Clean up old log files
        self.cleanup_old_log_files().await?;

        // Create new log file
        let log_file_path = self.get_current_log_file_path();
        let file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&log_file_path)
            .await
            .map_err(|e| AppError::GenericError(format!("Failed to create log file: {}", e)))?;

        self.current_log_file = Some(BufWriter::new(file));
        debug!("Created new log file: {:?}", log_file_path);

        Ok(())
    }

    /// Get current log file path
    fn get_current_log_file_path(&self) -> PathBuf {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_secs();
        self.log_directory.join(format!("app_{}.jsonl", timestamp))
    }

    /// Clean up old log files
    async fn cleanup_old_log_files(&self) -> AppResult<()> {
        let mut entries = tokio::fs::read_dir(&self.log_directory).await.map_err(|e| {
            AppError::GenericError(format!("Failed to read log directory: {}", e))
        })?;

        let mut log_files = Vec::new();
        while let Ok(Some(entry)) = entries.next_entry().await {
            let path = entry.path();
            if path.extension().and_then(|ext| ext.to_str()) == Some("jsonl") {
                if let Ok(metadata) = entry.metadata().await {
                    if let Ok(modified) = metadata.modified() {
                        log_files.push((path, modified));
                    }
                }
            }
        }

        // Sort by modification time (newest first)
        log_files.sort_by(|a, b| b.1.cmp(&a.1));

        // Remove excess files
        for (path, _) in log_files.into_iter().skip(self.max_log_files) {
            if let Err(e) = tokio::fs::remove_file(&path).await {
                warn!("Failed to remove old log file {:?}: {}", path, e);
            } else {
                debug!("Removed old log file: {:?}", path);
            }
        }

        Ok(())
    }

    /// Check if debug mode is enabled
    pub fn is_debug_enabled(&self) -> bool {
        self.debug_enabled
    }
}

/// Debug report structure
#[derive(Debug, Serialize, Deserialize)]
pub struct DebugReport {
    pub generated_at: u64,
    pub time_period_hours: u64,
    pub total_sessions: usize,
    pub completed_sessions: usize,
    pub failed_sessions: usize,
    pub average_session_duration: f64,
    pub total_errors: usize,
    pub metrics: HashMap<String, f64>,
    pub recent_sessions: Vec<DebugSession>,
}

/// Global logging system
static mut LOGGING_SYSTEM: Option<LoggingSystem> = None;
static LOGGING_SYSTEM_INIT: std::sync::Once = std::sync::Once::new();

/// Get the global logging system
pub async fn get_logging_system() -> &'static mut LoggingSystem {
    unsafe {
        LOGGING_SYSTEM_INIT.call_once(|| {
            LOGGING_SYSTEM = Some(LoggingSystem::new("logs", 100, 50, true)); // 100 files, 50MB each
        });
        LOGGING_SYSTEM.as_mut().unwrap()
    }
}

/// Initialize comprehensive logging
pub async fn init_comprehensive_logging() -> AppResult<()> {
    let system = get_logging_system().await;
    system.initialize().await?;
    info!("Comprehensive logging system initialized");
    Ok(())
}

/// Log a message with the comprehensive logging system
pub async fn log_message(level: LogLevel, component: &str, message: &str) -> AppResult<()> {
    let entry = LogEntry::new(level, component, message);
    let system = get_logging_system().await;
    system.log(entry).await
}

/// Create a debug session
pub async fn create_debug_session(name: &str) -> String {
    let system = get_logging_system().await;
    system.create_debug_session(name).await
}

/// End a debug session
pub async fn end_debug_session(session_id: &str) -> AppResult<()> {
    let system = get_logging_system().await;
    system.end_debug_session(session_id).await
}

/// Record a metric
pub async fn record_metric(name: &str, value: f64) {
    let system = get_logging_system().await;
    system.record_metric(name, value).await
}

/// Generate a debug report
pub async fn generate_debug_report(hours: u64) -> AppResult<DebugReport> {
    let system = get_logging_system().await;
    system.generate_debug_report(hours).await
}