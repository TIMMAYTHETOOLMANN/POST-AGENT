use crate::config;
use crate::error::{AppError, AppResult};
use crate::tiktok::TikTokVideo;
use chrono::{DateTime, Utc};
use log::{debug, info};
use once_cell::sync::Lazy;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::Mutex;

/// Database of posted content
#[derive(Debug, Serialize, Deserialize)]
struct PostDatabase {
    /// Map of video IDs to posting status
    videos: FxHashMap<String, PostStatus>,

    /// Last time the database was updated
    last_updated: DateTime<Utc>,
}

/// Status of a posted video
#[derive(Debug, Clone, Serialize, Deserialize)]
struct PostStatus {
    /// Video ID
    video_id: String,

    /// Whether the video has been posted to Facebook
    posted_to_facebook: bool,

    /// Whether the video has been posted to Instagram
    posted_to_instagram: bool,

    /// Whether the video has been posted to YouTube
    posted_to_youtube: bool,

    /// When the video was first posted
    first_posted_at: Option<DateTime<Utc>>,

    /// When the video was last posted
    last_posted_at: Option<DateTime<Utc>>,
}

/// Global database instance
static DB: Lazy<Arc<Mutex<Option<PostDatabase>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

/// Initialize the scheduler
pub async fn init() -> AppResult<()> {
    info!("Initializing scheduler");

    // Load or create the database
    let db = load_database().await?;

    // Store the database in the global instance
    let mut db_lock = DB.lock().await;
    *db_lock = Some(db);

    info!("Scheduler initialized");

    Ok(())
}

/// Mark a video as posted
pub async fn mark_as_posted(video_id: &str) -> AppResult<()> {
    debug!("Marking video as posted: {}", video_id);

    let mut db_lock = DB.lock().await;
    let db = db_lock
        .as_mut()
        .ok_or_else(|| AppError::SchedulerError("Scheduler not initialized".to_string()))?;

    // Update or create the post status
    let now = Utc::now();
    let status = db
        .videos
        .entry(video_id.to_string())
        .or_insert_with(|| PostStatus {
            video_id: video_id.to_string(),
            posted_to_facebook: false,
            posted_to_instagram: false,
            posted_to_youtube: false,
            first_posted_at: None,
            last_posted_at: None,
        });

    // Update the status
    status.posted_to_facebook = true;
    status.posted_to_instagram = true;
    status.posted_to_youtube = true;

    if status.first_posted_at.is_none() {
        status.first_posted_at = Some(now);
    }

    status.last_posted_at = Some(now);

    // Update the last updated timestamp
    db.last_updated = now;

    // Save the database
    save_database(db).await?;

    debug!("Video marked as posted: {}", video_id);

    Ok(())
}

/// Get the next legacy video to post
pub async fn get_next_legacy_video() -> AppResult<Option<TikTokVideo>> {
    debug!("Getting next legacy video to post");

    // Get all videos and find the next unposted one (oldest first)
    let videos = crate::tiktok::fetch_all_videos().await?;
    
    for video in videos {
        if !is_video_posted(&video.id).await? {
            debug!("Next legacy video: {}", video.id);
            return Ok(Some(video));
        }
    }

    debug!("No more legacy videos to post");
    Ok(None)
}

/// Check if a video has been posted
pub async fn is_video_posted(video_id: &str) -> AppResult<bool> {
    let db_lock = DB.lock().await;
    let db = db_lock
        .as_ref()
        .ok_or_else(|| AppError::SchedulerError("Scheduler not initialized".to_string()))?;

    Ok(db.videos.contains_key(video_id))
}

/// Load the database from disk
async fn load_database() -> AppResult<PostDatabase> {
    let db_path = get_database_path()?;

    if Path::new(&db_path).exists() {
        // Load the database from disk
        let db_json = fs::read_to_string(&db_path)
            .map_err(|e| AppError::DatabaseError(format!("Failed to read database: {}", e)))?;

        let db: PostDatabase = serde_json::from_str(&db_json)
            .map_err(|e| AppError::DatabaseError(format!("Failed to parse database: {}", e)))?;

        info!("Loaded database with {} videos", db.videos.len());

        Ok(db)
    } else {
        // Create a new database
        let db = PostDatabase {
            videos: FxHashMap::default(),
            last_updated: Utc::now(),
        };

        // Create the directory if it doesn't exist
        if let Some(parent) = Path::new(&db_path).parent() {
            fs::create_dir_all(parent).map_err(|e| {
                AppError::DatabaseError(format!("Failed to create database directory: {}", e))
            })?;
        }

        // Save the new database
        let db_json = serde_json::to_string_pretty(&db)
            .map_err(|e| AppError::DatabaseError(format!("Failed to serialize database: {}", e)))?;

        fs::write(&db_path, db_json)
            .map_err(|e| AppError::DatabaseError(format!("Failed to write database: {}", e)))?;

        info!("Created new database");

        Ok(db)
    }
}

/// Save the database to disk
async fn save_database(db: &PostDatabase) -> AppResult<()> {
    let db_path = get_database_path()?;

    // Serialize the database
    let db_json = serde_json::to_string_pretty(db)
        .map_err(|e| AppError::DatabaseError(format!("Failed to serialize database: {}", e)))?;

    // Save to disk
    fs::write(&db_path, db_json)
        .map_err(|e| AppError::DatabaseError(format!("Failed to write database: {}", e)))?;

    debug!("Saved database with {} videos", db.videos.len());

    Ok(())
}

/// Get the path to the database file
fn get_database_path() -> AppResult<PathBuf> {
    let _config = config::get_config();

    // In a real implementation, this might be configurable
    // For now, we'll just use a fixed path
    let db_dir = PathBuf::from("data");
    let db_path = db_dir.join("posts.json");

    Ok(db_path)
}
