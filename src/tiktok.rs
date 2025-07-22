use crate::browser::BrowserManager;
use crate::config;
use crate::error::{AppError, AppResult};
use headless_chrome::Tab;
use log::{info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::fs;
use tokio::time::sleep;

/// Represents a TikTok video
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TikTokVideo {
    pub id: String,
    pub title: String,
    pub description: String,
    pub video_url: String,
    pub cover_image_url: String,
    pub duration: u64,
    pub view_count: u64,
    pub like_count: u64,
    pub share_count: u64,
    pub comment_count: u64,
    pub create_time: u64,
    pub hashtags: Vec<String>,
    pub local_video_path: Option<String>,
    pub local_cover_path: Option<String>,
}

/// Browser automation for TikTok
pub struct TikTokBrowser {
    manager: BrowserManager,
    tab: Arc<Tab>,
}

impl TikTokBrowser {
    pub async fn new() -> AppResult<Self> {
        info!("Initializing TikTok browser automation");
        
        let manager = BrowserManager::new()?;
        let tab = manager.new_tab()?;
        
        Ok(Self { manager, tab })
    }

    pub async fn login(&mut self, email: &str, password: &str) -> AppResult<()> {
        info!("Logging into TikTok with email: {}", email);
        
        // Navigate to TikTok login page
        self.manager.navigate_with_retry(&self.tab, "https://www.tiktok.com/login/phone-or-email/email").await?;
        
        // Fill in email
        self.manager.wait_for_element_with_retry(&self.tab, "input[name='username']").await?;
        self.manager.type_text_with_retry(&self.tab, "input[name='username']", email).await?;
        
        // Fill in password
        self.manager.type_text_with_retry(&self.tab, "input[type='password']", password).await?;
        
        // Click login button
        self.manager.click_with_retry(&self.tab, "button[data-e2e='login-button']").await?;
        
        // Wait for successful login
        self.manager.wait(Duration::from_secs(5)).await;
        
        info!("Successfully logged into TikTok");
        Ok(())
    }

    pub async fn navigate_to_profile(&mut self, username: &str) -> AppResult<()> {
        info!("Navigating to TikTok profile: @{}", username);
        
        let profile_url = format!("https://www.tiktok.com/@{}", username);
        self.manager.navigate_with_retry(&self.tab, &profile_url).await?;
        
        // Wait for profile to load
        self.manager.wait_for_element_with_retry(&self.tab, "div[data-e2e='user-page']").await?;
        
        Ok(())
    }

    pub async fn extract_video_urls(&mut self) -> AppResult<Vec<String>> {
        info!("Extracting video URLs from profile page");
        
        // Scroll down to load more videos
        for _ in 0..5 {
            self.tab.evaluate("window.scrollTo(0, document.body.scrollHeight)", false)
                .map_err(|e| AppError::GenericError(format!("Failed to scroll: {}", e)))?;
            self.manager.wait(Duration::from_secs(2)).await;
        }

        // Extract video links
        let video_urls_result = self.tab.evaluate(r#"
            Array.from(document.querySelectorAll('a[href*="/video/"]')).map(a => a.href)
        "#, false).map_err(|e| AppError::GenericError(format!("Failed to extract video URLs: {}", e)))?;

        // Parse the result
        let video_urls: Vec<String> = if let Some(value) = video_urls_result.value {
            serde_json::from_value(value)
                .map_err(|e| AppError::GenericError(format!("Failed to parse video URLs: {}", e)))?
        } else {
            Vec::new()
        };

        info!("Found {} video URLs", video_urls.len());
        Ok(video_urls)
    }

    pub async fn extract_video_metadata(&mut self, video_url: &str) -> AppResult<TikTokVideo> {
        info!("Extracting metadata for video: {}", video_url);
        
        self.manager.navigate_with_retry(&self.tab, video_url).await?;
        
        // Wait for video to load
        self.manager.wait_for_element_with_retry(&self.tab, "div[data-e2e='video-player']").await?;
        
        // Extract video metadata using JavaScript
        let metadata_result = self.tab.evaluate(r#"
            (function() {
                try {
                    // Extract video ID from URL
                    const videoId = window.location.pathname.split('/').pop();
                    
                    // Extract title and description
                    const titleElement = document.querySelector('h1[data-e2e="browse-video-desc"]') || 
                                        document.querySelector('[data-e2e="video-desc"]');
                    const title = titleElement ? titleElement.textContent.trim() : '';
                    
                    // Extract stats
                    const likesElement = document.querySelector('[data-e2e="like-count"]');
                    const commentsElement = document.querySelector('[data-e2e="comment-count"]');
                    const sharesElement = document.querySelector('[data-e2e="share-count"]');
                    
                    const likes = likesElement ? parseInt(likesElement.textContent.replace(/\D/g, '')) || 0 : 0;
                    const comments = commentsElement ? parseInt(commentsElement.textContent.replace(/\D/g, '')) || 0 : 0;
                    const shares = sharesElement ? parseInt(sharesElement.textContent.replace(/\D/g, '')) || 0 : 0;
                    
                    // Extract video source URL
                    const videoElement = document.querySelector('video');
                    const videoSrc = videoElement ? videoElement.src : '';
                    
                    // Extract cover image
                    const coverElement = videoElement ? videoElement.poster : '';
                    
                    // Extract hashtags
                    const hashtagElements = document.querySelectorAll('a[href*="/tag/"]');
                    const hashtags = Array.from(hashtagElements).map(el => el.textContent.trim());
                    
                    return {
                        id: videoId,
                        title: title,
                        description: title,
                        video_url: videoSrc,
                        cover_image_url: coverElement,
                        duration: videoElement ? Math.floor(videoElement.duration) || 0 : 0,
                        view_count: 0, // Views not easily accessible
                        like_count: likes,
                        share_count: shares,
                        comment_count: comments,
                        create_time: Math.floor(Date.now() / 1000),
                        hashtags: hashtags
                    };
                } catch (error) {
                    return { error: error.message };
                }
            })()
        "#, false).map_err(|e| AppError::GenericError(format!("Failed to extract metadata: {}", e)))?;

        let metadata: HashMap<String, serde_json::Value> = if let Some(value) = metadata_result.value {
            serde_json::from_value(value)
                .map_err(|e| AppError::GenericError(format!("Failed to parse metadata: {}", e)))?
        } else {
            return Err(AppError::GenericError("No metadata returned from JavaScript".to_string()));
        };

        if let Some(error) = metadata.get("error") {
            return Err(AppError::GenericError(format!("JavaScript error: {}", error)));
        }

        let video = TikTokVideo {
            id: metadata.get("id").and_then(|v| v.as_str()).unwrap_or("unknown").to_string(),
            title: metadata.get("title").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            description: metadata.get("description").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            video_url: metadata.get("video_url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            cover_image_url: metadata.get("cover_image_url").and_then(|v| v.as_str()).unwrap_or("").to_string(),
            duration: metadata.get("duration").and_then(|v| v.as_u64()).unwrap_or(0),
            view_count: metadata.get("view_count").and_then(|v| v.as_u64()).unwrap_or(0),
            like_count: metadata.get("like_count").and_then(|v| v.as_u64()).unwrap_or(0),
            share_count: metadata.get("share_count").and_then(|v| v.as_u64()).unwrap_or(0),
            comment_count: metadata.get("comment_count").and_then(|v| v.as_u64()).unwrap_or(0),
            create_time: metadata.get("create_time").and_then(|v| v.as_u64()).unwrap_or(0),
            hashtags: metadata.get("hashtags")
                .and_then(|v| v.as_array())
                .map(|arr| arr.iter().filter_map(|v| v.as_str().map(|s| s.to_string())).collect())
                .unwrap_or_default(),
            local_video_path: None,
            local_cover_path: None,
        };

        info!("Extracted metadata for video: {}", video.id);
        Ok(video)
    }

    pub async fn verify_authentication(&mut self) -> AppResult<()> {
        info!("Verifying TikTok authentication");
        
        // Check if we're still logged in by looking for profile elements
        self.manager.navigate_with_retry(&self.tab, "https://www.tiktok.com/foryou").await?;
        self.manager.wait_for_element_with_retry(&self.tab, "div[data-e2e='recommend-list-item']").await?;
        
        Ok(())
    }
}

/// Authenticate with TikTok using browser automation
pub async fn authenticate() -> AppResult<()> {
    info!("Authenticating with TikTok via browser automation");

    let config = config::get_config();
    let tiktok_config = &config.tiktok;

    let mut browser = TikTokBrowser::new().await?;
    
    // Test login
    browser.login(&tiktok_config.email, &tiktok_config.password).await?;
    
    // Verify authentication was successful
    browser.verify_authentication().await?;
    
    info!("TikTok authentication completed successfully");
    Ok(())
}

/// Fetches all TikTok videos from the configured account using browser automation
pub async fn fetch_all_videos() -> AppResult<Vec<TikTokVideo>> {
    info!("Fetching all TikTok videos using browser automation");

    let config = config::get_config();
    let username = &config.tiktok.username;

    info!("Fetching videos for TikTok user: {}", username);

    let mut browser = TikTokBrowser::new().await?;
    
    // Login to TikTok
    browser.login(&config.tiktok.email, &config.tiktok.password).await?;
    
    // Navigate to user profile
    browser.navigate_to_profile(username).await?;
    
    // Extract video URLs
    let video_urls = browser.extract_video_urls().await?;
    
    // Extract metadata for each video
    let mut videos = Vec::new();
    for video_url in video_urls.iter().take(20) { // Limit to 20 videos for performance
        match browser.extract_video_metadata(video_url).await {
            Ok(video) => videos.push(video),
            Err(e) => {
                warn!("Failed to extract metadata for {}: {}", video_url, e);
                continue;
            }
        }
        
        // Small delay between requests
        sleep(Duration::from_secs(1)).await;
    }

    info!("Successfully fetched {} TikTok videos", videos.len());
    Ok(videos)
}

/// Fetches new TikTok videos that haven't been processed yet using browser automation
pub async fn fetch_new_videos() -> AppResult<Vec<TikTokVideo>> {
    info!("Fetching new TikTok videos using browser automation");

    // For simplicity, we'll fetch all videos and let the scheduler filter out already processed ones
    fetch_all_videos().await
}

/// Downloads video content for a TikTok video
pub async fn download_video_content(video: &mut TikTokVideo) -> AppResult<()> {
    info!("Downloading content for video: {}", video.id);

    // Create downloads directory if it doesn't exist
    fs::create_dir_all("downloads").await.map_err(|e| {
        AppError::GenericError(format!("Failed to create downloads directory: {}", e))
    })?;

    let client = Client::new();

    // Download video file
    if !video.video_url.is_empty() {
        let video_filename = format!("downloads/{}_video.mp4", video.id);
        
        match client.get(&video.video_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let content = response.bytes().await.map_err(|e| {
                        AppError::GenericError(format!("Failed to read video content: {}", e))
                    })?;
                    
                    fs::write(&video_filename, content).await.map_err(|e| {
                        AppError::GenericError(format!("Failed to write video file: {}", e))
                    })?;
                    
                    video.local_video_path = Some(video_filename);
                    info!("Downloaded video file for: {}", video.id);
                } else {
                    warn!("Failed to download video for {}: HTTP {}", video.id, response.status());
                }
            }
            Err(e) => {
                warn!("Failed to download video for {}: {}", video.id, e);
            }
        }
    }

    // Download cover image
    if !video.cover_image_url.is_empty() {
        let cover_filename = format!("downloads/{}_cover.jpg", video.id);
        
        match client.get(&video.cover_image_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let content = response.bytes().await.map_err(|e| {
                        AppError::GenericError(format!("Failed to read cover content: {}", e))
                    })?;
                    
                    fs::write(&cover_filename, content).await.map_err(|e| {
                        AppError::GenericError(format!("Failed to write cover file: {}", e))
                    })?;
                    
                    video.local_cover_path = Some(cover_filename);
                    info!("Downloaded cover image for: {}", video.id);
                } else {
                    warn!("Failed to download cover for {}: HTTP {}", video.id, response.status());
                }
            }
            Err(e) => {
                warn!("Failed to download cover for {}: {}", video.id, e);
            }
        }
    }

    // Add small delay to avoid rate limiting
    sleep(Duration::from_millis(500)).await;

    Ok(())
}