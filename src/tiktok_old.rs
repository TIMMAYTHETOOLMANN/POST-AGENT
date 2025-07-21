use crate::config;
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;

/// Represents a TikTok video with all its metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokVideo {
    /// Unique identifier for the video
    pub id: String,

    /// URL to the video on TikTok
    pub url: String,

    /// Description/caption of the video
    pub description: String,

    /// Creation timestamp
    pub created_at: DateTime<Utc>,

    /// Number of likes
    pub likes: u64,

    /// Number of comments
    pub comments: u64,

    /// Number of shares
    pub shares: u64,

    /// Number of views
    pub views: u64,

    /// Duration of the video in seconds
    pub duration: f64,

    /// URL to the video file
    pub video_url: String,

    /// URL to the cover image
    pub cover_url: String,

    /// Original hashtags used in the video
    pub hashtags: Vec<String>,

    /// Path to the downloaded video file (if downloaded)
    pub local_video_path: Option<PathBuf>,

    /// Path to the downloaded cover image (if downloaded)
    pub local_cover_path: Option<PathBuf>,

    /// Whether this video has been processed and posted
    pub is_posted: bool,
}

/// Fetches all TikTok videos from the configured account
pub async fn fetch_all_videos() -> AppResult<Vec<TikTokVideo>> {
    info!("Fetching all TikTok videos");

    let config = config::get_config();
    let username = &config.tiktok.username;
    let _api_key = &config.tiktok.api_key;

    info!("Fetching videos for TikTok user: {}", username);

    let client = Client::new();
    
    // Use TikTok API to fetch all videos
    let url = format!("https://open.tiktokapis.com/v2/video/list/");
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.tiktok.access_token.as_ref().unwrap_or(&"".to_string())))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "max_count": 100,
            "fields": ["id", "title", "video_description", "duration", "cover_image_url", "embed_link", "create_time", "like_count", "comment_count", "share_count", "view_count"]
        }))
        .send()
        .await
        .map_err(|e| AppError::ApiError(format!("Failed to fetch videos: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::ApiError(format!("TikTok API error: {}", response.status())));
    }

    let json: serde_json::Value = response.json().await
        .map_err(|e| AppError::ApiError(format!("Failed to parse API response: {}", e)))?;

    let mut videos = Vec::new();
    
    if let Some(data) = json.get("data").and_then(|d| d.get("videos")).and_then(|v| v.as_array()) {
        for video_data in data {
            if let Some(video) = parse_tiktok_video(video_data, username)? {
                videos.push(video);
            }
        }
    }

    // Sort by creation date (oldest first)
    videos.sort_by(|a, b| a.created_at.cmp(&b.created_at));

    info!("Fetched {} TikTok videos", videos.len());

    Ok(videos)
}

/// Fetches only new TikTok videos that haven't been processed yet
pub async fn fetch_new_videos() -> AppResult<Vec<TikTokVideo>> {
    info!("Fetching new TikTok videos");

    let config = config::get_config();
    let username = &config.tiktok.username;

    info!("Checking for new videos for TikTok user: {}", username);

    let client = Client::new();
    
    // Use TikTok API to fetch recent videos
    let url = format!("https://open.tiktokapis.com/v2/video/list/");
    
    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.tiktok.access_token.as_ref().unwrap_or(&"".to_string())))
        .header("Content-Type", "application/json")
        .json(&serde_json::json!({
            "max_count": 20,
            "fields": ["id", "title", "video_description", "duration", "cover_image_url", "embed_link", "create_time", "like_count", "comment_count", "share_count", "view_count"]
        }))
        .send()
        .await
        .map_err(|e| AppError::ApiError(format!("Failed to fetch videos: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::ApiError(format!("TikTok API error: {}", response.status())));
    }

    let json: serde_json::Value = response.json().await
        .map_err(|e| AppError::ApiError(format!("Failed to parse API response: {}", e)))?;

    let mut videos = Vec::new();
    
    if let Some(data) = json.get("data").and_then(|d| d.get("videos")).and_then(|v| v.as_array()) {
        for video_data in data {
            if let Some(video) = parse_tiktok_video(video_data, username)? {
                videos.push(video);
            }
        }
    }

    // Sort by creation date (newest first for new videos)
    videos.sort_by(|a, b| b.created_at.cmp(&a.created_at));

    info!("Fetched {} new TikTok videos", videos.len());

    Ok(videos)
}

/// Downloads the video and cover image for a TikTok video
pub async fn download_video_content(video: &mut TikTokVideo) -> AppResult<()> {
    info!("Downloading content for TikTok video: {}", video.id);

    // Create directory for downloads if it doesn't exist
    let download_dir = PathBuf::from("downloads");
    if !download_dir.exists() {
        fs::create_dir_all(&download_dir).await?;
    }

    let client = Client::new();

    // Download video
    let video_path = download_dir.join(format!("{}.mp4", video.id));
    let video_bytes = client
        .get(&video.video_url)
        .timeout(Duration::from_secs(60))
        .send()
        .await
        .map_err(|e| AppError::MediaDownloadError(format!("Failed to download video: {}", e)))?
        .bytes()
        .await
        .map_err(|e| AppError::MediaDownloadError(format!("Failed to read video bytes: {}", e)))?;

    fs::write(&video_path, video_bytes).await?;
    video.local_video_path = Some(video_path);

    // Download cover image
    let cover_path = download_dir.join(format!("{}.jpg", video.id));
    let cover_bytes = client
        .get(&video.cover_url)
        .timeout(Duration::from_secs(30))
        .send()
        .await
        .map_err(|e| AppError::MediaDownloadError(format!("Failed to download cover: {}", e)))?
        .bytes()
        .await
        .map_err(|e| AppError::MediaDownloadError(format!("Failed to read cover bytes: {}", e)))?;

    fs::write(&cover_path, cover_bytes).await?;
    video.local_cover_path = Some(cover_path);

    info!(
        "Successfully downloaded content for TikTok video: {}",
        video.id
    );

    Ok(())
}

/// Parse TikTok video data from API response
fn parse_tiktok_video(video_data: &serde_json::Value, username: &str) -> AppResult<Option<TikTokVideo>> {
    let id = video_data.get("id").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let _title = video_data.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let description = video_data.get("video_description").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let duration = video_data.get("duration").and_then(|v| v.as_f64()).unwrap_or(0.0);
    let cover_url = video_data.get("cover_image_url").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let embed_link = video_data.get("embed_link").and_then(|v| v.as_str()).unwrap_or("").to_string();
    let create_time = video_data.get("create_time").and_then(|v| v.as_i64()).unwrap_or(0);
    let like_count = video_data.get("like_count").and_then(|v| v.as_u64()).unwrap_or(0);
    let comment_count = video_data.get("comment_count").and_then(|v| v.as_u64()).unwrap_or(0);
    let share_count = video_data.get("share_count").and_then(|v| v.as_u64()).unwrap_or(0);
    let view_count = video_data.get("view_count").and_then(|v| v.as_u64()).unwrap_or(0);

    if id.is_empty() {
        return Ok(None);
    }

    // Parse hashtags from description
    let hashtags = extract_hashtags(&description);

    // Convert timestamp to DateTime
    let created_at = DateTime::from_timestamp(create_time, 0)
        .unwrap_or_else(|| Utc::now());

    let video = TikTokVideo {
        id,
        url: embed_link,
        description,
        created_at,
        likes: like_count,
        comments: comment_count,
        shares: share_count,
        views: view_count,
        duration,
        video_url: format!("https://www.tiktok.com/@{}/video/{}", username, video_data.get("id").and_then(|v| v.as_str()).unwrap_or("")),
        cover_url,
        hashtags,
        local_video_path: None,
        local_cover_path: None,
        is_posted: false,
    };

    Ok(Some(video))
}

/// Extract hashtags from description text
fn extract_hashtags(description: &str) -> Vec<String> {
    let mut hashtags = Vec::new();
    let words: Vec<&str> = description.split_whitespace().collect();
    
    for word in words {
        if word.starts_with('#') && word.len() > 1 {
            hashtags.push(word[1..].to_string());
        }
    }
    
    hashtags
}

/// Authenticates with the TikTok API
pub async fn authenticate() -> AppResult<()> {
    info!("Authenticating with TikTok API");

    let config = config::get_config();
    let client = Client::new();

    // Use TikTok API for authentication
    let url = "https://open.tiktokapis.com/v2/oauth/token/";
    
    let response = client
        .post(url)
        .header("Content-Type", "application/x-www-form-urlencoded")
        .form(&[
            ("client_key", &config.tiktok.api_key),
            ("client_secret", &config.tiktok.api_secret),
            ("grant_type", &"client_credentials".to_string()),
        ])
        .send()
        .await
        .map_err(|e| AppError::AuthenticationError(format!("Failed to authenticate: {}", e)))?;

    if !response.status().is_success() {
        return Err(AppError::AuthenticationError(format!("TikTok authentication failed: {}", response.status())));
    }

    let _json: serde_json::Value = response.json().await
        .map_err(|e| AppError::AuthenticationError(format!("Failed to parse auth response: {}", e)))?;

    info!("Successfully authenticated with TikTok API");

    Ok(())
}
