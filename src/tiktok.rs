use crate::config;
use crate::error::{AppError, AppResult};
use chrono::{DateTime, Utc};
use log::info;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

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

    info!("Fetching videos for TikTok user: {}", username);

    // In a real implementation, this would use the TikTok API
    // For now, we'll simulate fetching videos
    let videos = simulate_fetch_videos(username).await?;

    info!("Fetched {} TikTok videos", videos.len());

    Ok(videos)
}

/// Fetches only new TikTok videos that haven't been processed yet
pub async fn fetch_new_videos() -> AppResult<Vec<TikTokVideo>> {
    info!("Fetching new TikTok videos");

    let config = config::get_config();
    let username = &config.tiktok.username;

    // In a real implementation, this would use the TikTok API and check against
    // a database of already processed videos
    // For now, we'll simulate fetching new videos
    let videos = simulate_fetch_new_videos(username).await?;

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

    // For simulation, create mock video files instead of downloading from non-existent URLs
    if video.video_url.contains("tiktok.com") {
        info!("Simulating download for video: {}", video.id);
        
        // Create mock video file
        let video_path = download_dir.join(format!("{}.mp4", video.id));
        let mock_video_data = create_mock_video_data();
        fs::write(&video_path, mock_video_data).await?;
        video.local_video_path = Some(video_path);

        // Create mock cover image
        let cover_path = download_dir.join(format!("{}.jpg", video.id));
        let mock_cover_data = create_mock_cover_data();
        fs::write(&cover_path, mock_cover_data).await?;
        video.local_cover_path = Some(cover_path);

        info!("Successfully created mock files for TikTok video: {}", video.id);
        return Ok(());
    }

    // Real download logic for actual URLs
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

/// Creates mock video data for simulation
fn create_mock_video_data() -> Vec<u8> {
    // Create a minimal MP4 header for simulation
    // This is just a placeholder - in real implementation, you'd use actual video data
    vec![
        0x00, 0x00, 0x00, 0x18, 0x66, 0x74, 0x79, 0x70, 0x6D, 0x70, 0x34, 0x31,
        0x00, 0x00, 0x00, 0x00, 0x6D, 0x70, 0x34, 0x31, 0x69, 0x73, 0x6F, 0x6D,
        // Add some dummy data to make it look like a real file
        0x00, 0x00, 0x00, 0x10, 0x6D, 0x64, 0x61, 0x74, 0x48, 0x65, 0x6C, 0x6C,
        0x6F, 0x20, 0x54, 0x69, 0x6B, 0x54, 0x6F, 0x6B, 0x21, 0x00, 0x00, 0x00,
    ]
}

/// Creates mock cover image data for simulation
fn create_mock_cover_data() -> Vec<u8> {
    // Create a minimal JPEG header for simulation
    vec![
        0xFF, 0xD8, 0xFF, 0xE0, 0x00, 0x10, 0x4A, 0x46, 0x49, 0x46, 0x00, 0x01,
        0x01, 0x01, 0x00, 0x48, 0x00, 0x48, 0x00, 0x00, 0xFF, 0xD9,
    ]
}

/// Simulates fetching videos from TikTok (for development/testing)
async fn simulate_fetch_videos(username: &str) -> AppResult<Vec<TikTokVideo>> {
    info!("Simulating fetch of all videos for user: {}", username);

    // Simulate API delay
    sleep(Duration::from_millis(500)).await;

    // Create some sample videos
    let mut videos = Vec::new();

    for i in 1..6 {
        let video = TikTokVideo {
            id: format!("video_{}", i),
            url: format!("https://tiktok.com/@{}/video/{}", username, i),
            description: format!("This is video #{} #fyp #viral", i),
            created_at: Utc::now() - chrono::Duration::days(i),
            likes: 1000 * i as u64,
            comments: 100 * i as u64,
            shares: 50 * i as u64,
            views: 5000 * i as u64,
            duration: 15.0,
            video_url: format!("https://tiktok.com/video/{}.mp4", i),
            cover_url: format!("https://tiktok.com/cover/{}.jpg", i),
            hashtags: vec!["fyp".to_string(), "viral".to_string()],
            local_video_path: None,
            local_cover_path: None,
            is_posted: false,
        };

        videos.push(video);
    }

    Ok(videos)
}

/// Simulates fetching new videos from TikTok (for development/testing)
async fn simulate_fetch_new_videos(username: &str) -> AppResult<Vec<TikTokVideo>> {
    info!("Simulating fetch of new videos for user: {}", username);

    // Simulate API delay
    sleep(Duration::from_millis(500)).await;

    // Create a deterministic new video based on current time, but only return it occasionally
    // This prevents the infinite loop of always returning the same "new" video
    let now = Utc::now();
    let minutes_since_midnight = (now.timestamp() % 86400) / 60;
    
    // Only return a "new" video every 30 minutes (to simulate realistic posting frequency)
    if minutes_since_midnight % 30 == 0 {
        let video = TikTokVideo {
            id: format!("video_new_{}", now.timestamp() / 1800), // Group by 30-minute periods
            url: format!("https://tiktok.com/@{}/video/new", username),
            description: "This is a new video #fyp #viral #trending".to_string(),
            created_at: now,
            likes: 0,
            comments: 0,
            shares: 0,
            views: 0,
            duration: 20.0,
            video_url: "https://tiktok.com/video/new.mp4".to_string(),
            cover_url: "https://tiktok.com/cover/new.jpg".to_string(),
            hashtags: vec![
                "fyp".to_string(),
                "viral".to_string(),
                "trending".to_string(),
            ],
            local_video_path: None,
            local_cover_path: None,
            is_posted: false,
        };

        info!("Returning simulated new video: {}", video.id);
        Ok(vec![video])
    } else {
        info!("No new videos available at this time");
        Ok(vec![])
    }
}

/// Authenticates with the TikTok API
pub async fn authenticate() -> AppResult<()> {
    info!("Authenticating with TikTok API");

    let _config = config::get_config();

    // In a real implementation, this would use the TikTok API for authentication
    // For now, we'll just simulate authentication

    info!("Successfully authenticated with TikTok API");

    Ok(())
}
