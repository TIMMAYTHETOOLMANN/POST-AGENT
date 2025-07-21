use crate::error::{AppError, AppResult};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::fs;

/// Session data for a platform
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PlatformSession {
    /// Platform name (facebook, instagram, youtube, tiktok)
    pub platform: String,
    /// Session cookies as key-value pairs
    pub cookies: HashMap<String, String>,
    /// Local storage data
    pub local_storage: HashMap<String, String>,
    /// Session storage data
    pub session_storage: HashMap<String, String>,
    /// User agent used for this session
    pub user_agent: String,
    /// Timestamp when session was created
    pub created_at: u64,
    /// Timestamp when session was last used
    pub last_used: u64,
    /// Whether the session is currently valid
    pub is_valid: bool,
}

impl PlatformSession {
    /// Create a new platform session
    pub fn new(platform: &str, user_agent: &str) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        Self {
            platform: platform.to_string(),
            cookies: HashMap::new(),
            local_storage: HashMap::new(),
            session_storage: HashMap::new(),
            user_agent: user_agent.to_string(),
            created_at: now,
            last_used: now,
            is_valid: true,
        }
    }

    /// Update the last used timestamp
    pub fn touch(&mut self) {
        self.last_used = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Check if the session is expired (older than 24 hours)
    pub fn is_expired(&self) -> bool {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();
        now - self.last_used > 86400 // 24 hours
    }

    /// Mark session as invalid
    pub fn invalidate(&mut self) {
        self.is_valid = false;
    }
}

/// Session manager for persistent browser sessions
pub struct SessionManager {
    sessions: HashMap<String, PlatformSession>,
    session_file: String,
}

impl SessionManager {
    /// Create a new session manager
    pub fn new() -> Self {
        Self {
            sessions: HashMap::new(),
            session_file: "data/sessions.json".to_string(),
        }
    }

    /// Load sessions from disk
    pub async fn load_sessions(&mut self) -> AppResult<()> {
        // Create data directory if it doesn't exist
        if let Some(parent) = Path::new(&self.session_file).parent() {
            fs::create_dir_all(parent).await.map_err(|e| {
                AppError::GenericError(format!("Failed to create sessions directory: {}", e))
            })?;
        }

        if Path::new(&self.session_file).exists() {
            let content = fs::read_to_string(&self.session_file).await.map_err(|e| {
                AppError::GenericError(format!("Failed to read sessions file: {}", e))
            })?;

            self.sessions = serde_json::from_str(&content).unwrap_or_else(|e| {
                warn!("Failed to parse sessions file, starting fresh: {}", e);
                HashMap::new()
            });

            // Clean up expired sessions
            self.sessions.retain(|_, session| !session.is_expired());

            info!("Loaded {} sessions from disk", self.sessions.len());
        } else {
            info!("No existing sessions file found, starting fresh");
        }

        Ok(())
    }

    /// Save sessions to disk
    pub async fn save_sessions(&self) -> AppResult<()> {
        let content = serde_json::to_string_pretty(&self.sessions).map_err(|e| {
            AppError::GenericError(format!("Failed to serialize sessions: {}", e))
        })?;

        fs::write(&self.session_file, content).await.map_err(|e| {
            AppError::GenericError(format!("Failed to save sessions file: {}", e))
        })?;

        debug!("Saved {} sessions to disk", self.sessions.len());
        Ok(())
    }

    /// Get a session for a platform
    pub fn get_session(&mut self, platform: &str) -> Option<&mut PlatformSession> {
        let should_remove = if let Some(session) = self.sessions.get(platform) {
            !session.is_valid || session.is_expired()
        } else {
            false
        };

        if should_remove {
            self.sessions.remove(platform);
            return None;
        }

        if let Some(session) = self.sessions.get_mut(platform) {
            session.touch();
            Some(session)
        } else {
            None
        }
    }

    /// Create or update a session for a platform
    pub fn set_session(&mut self, platform: &str, session: PlatformSession) {
        info!("Storing session for platform: {}", platform);
        self.sessions.insert(platform.to_string(), session);
    }

    /// Remove a session for a platform
    pub fn remove_session(&mut self, platform: &str) {
        if self.sessions.remove(platform).is_some() {
            info!("Removed session for platform: {}", platform);
        }
    }

    /// Invalidate all sessions
    pub fn clear_all_sessions(&mut self) {
        self.sessions.clear();
        info!("Cleared all sessions");
    }

    /// Get all platform names with valid sessions
    pub fn get_valid_platforms(&self) -> Vec<String> {
        self.sessions
            .iter()
            .filter(|(_, session)| session.is_valid && !session.is_expired())
            .map(|(platform, _)| platform.clone())
            .collect()
    }

    /// Check if a platform has a valid session
    pub fn has_valid_session(&self, platform: &str) -> bool {
        if let Some(session) = self.sessions.get(platform) {
            session.is_valid && !session.is_expired()
        } else {
            false
        }
    }

    /// Get session statistics
    pub fn get_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        
        let total = self.sessions.len() as u64;
        let valid = self.sessions.values().filter(|s| s.is_valid && !s.is_expired()).count() as u64;
        let expired = self.sessions.values().filter(|s| s.is_expired()).count() as u64;
        let invalid = self.sessions.values().filter(|s| !s.is_valid).count() as u64;

        stats.insert("total".to_string(), total);
        stats.insert("valid".to_string(), valid);
        stats.insert("expired".to_string(), expired);
        stats.insert("invalid".to_string(), invalid);

        stats
    }
}

/// Global session manager instance
static mut SESSION_MANAGER: Option<SessionManager> = None;
static SESSION_MANAGER_INIT: std::sync::Once = std::sync::Once::new();

/// Get the global session manager instance
pub async fn get_session_manager() -> &'static mut SessionManager {
    unsafe {
        SESSION_MANAGER_INIT.call_once(|| {
            SESSION_MANAGER = Some(SessionManager::new());
        });
        SESSION_MANAGER.as_mut().unwrap()
    }
}

/// Initialize the session manager and load existing sessions
pub async fn init_session_manager() -> AppResult<()> {
    let manager = get_session_manager().await;
    manager.load_sessions().await?;
    info!("Session manager initialized successfully");
    Ok(())
}

/// Save all sessions to disk
pub async fn save_all_sessions() -> AppResult<()> {
    let manager = get_session_manager().await;
    manager.save_sessions().await?;
    Ok(())
}