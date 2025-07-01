// Dispatcher Module
// Orchestrates posting to different platforms

use crate::config;
use crate::error::{AppError, AppResult};
use crate::facebook_client;
use crate::instagram_client;
use crate::tiktok::{self, TikTokVideo};
use crate::youtube_uploader;
use log::{debug, error, info, warn};
use std::path::Path;

/// Run the dispatcher to fetch and post new videos
pub async fn run_dispatch() -> Result<(), AppError> {
    info!("Running dispatcher to check for new videos");

    // Fetch latest videos from TikTok
    let videos = tiktok::fetch_new_videos().await?;
    info!("Found {} new TikTok videos", videos.len());

    if videos.is_empty() {
        debug!("No new videos to process");
        return Ok(());
    }

    // Get configuration
    let config = config::get_config();

    // Process each video
    for mut video in videos {
        // Check if this video is new (not already posted)
        if video.is_new() {
            info!("Processing new video: {}", video.id);

            // Download the video content if needed
            if video.local_video_path.is_none() {
                match tiktok::download_video_content(&mut video).await {
                    Ok(_) => info!("Downloaded content for video: {}", video.id),
                    Err(e) => {
                        error!("Failed to download content for video {}: {}", video.id, e);
                        continue;
                    }
                }
            }

            // Get the video path as a string
            let video_path = match video.local_video_path.as_ref() {
                Some(path) => path.to_string_lossy().to_string(),
                None => {
                    error!("No local video path available for video: {}", video.id);
                    continue;
                }
            };

            // Post to Instagram if enabled
            if config.instagram.enabled {
                match instagram_client::upload_to_instagram(&video_path, &video.description) {
                    Ok(_) => info!("Successfully posted to Instagram: {}", video.id),
                    Err(e) => error!("Failed to post to Instagram: {}", e),
                }
            }

            // Post to YouTube if enabled
            if config.youtube.enabled {
                match youtube_uploader::upload_to_youtube(
                    &video_path,
                    &video.description,
                    &video.description,
                ) {
                    Ok(_) => info!("Successfully posted to YouTube: {}", video.id),
                    Err(e) => error!("Failed to post to YouTube: {}", e),
                }
            }

            // Post to Facebook if enabled
            if config.facebook.enabled {
                match facebook_client::upload_to_facebook(&video_path, &video.description) {
                    Ok(_) => info!("Successfully posted to Facebook: {}", video.id),
                    Err(e) => error!("Failed to post to Facebook: {}", e),
                }
            }

            // Mark the video as posted
            video.mark_posted();
            info!("Marked video as posted: {}", video.id);
        } else {
            debug!("Skipping already posted video: {}", video.id);
        }
    }

    info!("Dispatcher run completed");
    Ok(())
}
