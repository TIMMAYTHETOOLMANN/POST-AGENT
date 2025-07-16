use crate::error::AppResult;
use crate::{content, platforms, scheduler, tiktok};
use log::{error, info};
use tokio::time::Duration;

/// Performs the initial synchronization of all TikTok content to all platforms
pub async fn initial_sync_all_content() -> AppResult<()> {
    info!("Starting initial sync of all TikTok content");

    // Fetch all TikTok videos
    let videos = tiktok::fetch_all_videos().await?;
    info!("Fetched {} TikTok videos", videos.len());

    if videos.is_empty() {
        info!("No TikTok videos found to sync");
        return Ok(());
    }

    // Videos are already sorted by creation date (oldest first) from the API
    // Process and post each video to all platforms
    let mut success_count = 0;
    for mut video in videos {
        // Check if this video has already been posted
        if scheduler::is_video_posted(&video.id).await? {
            info!("Skipping already posted video: {}", video.id);
            continue;
        }

        // Download the video content
        if let Err(e) = tiktok::download_video_content(&mut video).await {
            error!("Failed to download content for video {}: {}", video.id, e);
            continue;
        }
        info!("Downloaded content for video: {}", video.id);

        // Process content (add hashtags and taglines)
        let processed_content = match content::process_content(video).await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to process content: {}", e);
                continue;
            },
        };

        // Post to all platforms
        match platforms::post_to_all_platforms(processed_content.clone()).await {
            Ok(_) => {
                info!(
                    "Successfully posted video to all platforms: {}",
                    processed_content.original_video.id
                );
                success_count += 1;

                // Record that this content has been posted
                if let Err(e) =
                    scheduler::mark_as_posted(&processed_content.original_video.id).await
                {
                    error!("Failed to mark video as posted: {}", e);
                }
            },
            Err(e) => {
                error!("Failed to post video to all platforms: {}", e);
                // Continue with next video
            },
        }

        // Add a 10-minute delay between posts as requested
        tokio::time::sleep(Duration::from_secs(600)).await;
    }

    info!(
        "Initial sync completed. Successfully posted {} videos",
        success_count
    );
    Ok(())
}

/// Monitors TikTok for new content and posts it to all platforms
pub async fn monitor_and_post_new_content() -> AppResult<()> {
    info!("Checking for new TikTok content");

    // Fetch new TikTok videos
    let new_videos = tiktok::fetch_new_videos().await?;
    info!("Found {} new TikTok videos", new_videos.len());

    if new_videos.is_empty() {
        return Ok(());
    }

    // Process and post each new video to all platforms
    let mut processed_count = 0;
    for mut video in new_videos {
        // Check if this video has already been posted
        if scheduler::is_video_posted(&video.id).await? {
            info!("Skipping already posted video: {}", video.id);
            continue;
        }

        // Download the video content
        if let Err(e) = tiktok::download_video_content(&mut video).await {
            error!("Failed to download content for video {}: {}", video.id, e);
            continue;
        }
        info!("Downloaded content for video: {}", video.id);

        // Process content (add hashtags and taglines)
        let processed_content = match content::process_content(video).await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to process content: {}", e);
                continue;
            },
        };

        // Post to all platforms
        match platforms::post_to_all_platforms(processed_content.clone()).await {
            Ok(_) => {
                info!(
                    "Successfully posted new video to all platforms: {}",
                    processed_content.original_video.id
                );
                processed_count += 1;

                // Record that this content has been posted
                if let Err(e) =
                    scheduler::mark_as_posted(&processed_content.original_video.id).await
                {
                    error!("Failed to mark video as posted: {}", e);
                }
            },
            Err(e) => {
                error!("Failed to post new video to all platforms: {}", e);
                // Continue with next video
            },
        }
    }

    if processed_count > 0 {
        info!("Successfully processed {} new videos", processed_count);
    }

    Ok(())
}

/// Posts legacy TikTok content according to the schedule
pub async fn post_legacy_content() -> AppResult<()> {
    info!("Posting next legacy TikTok content");

    // Get the next legacy video to post
    if let Some(mut video) = scheduler::get_next_legacy_video().await? {
        info!("Found legacy video to post: {}", video.id);

        // Check if this video has already been posted
        if scheduler::is_video_posted(&video.id).await? {
            info!("Skipping already posted legacy video: {}", video.id);
            return Ok(());
        }

        // Download the video content if needed
        if video.local_video_path.is_none() || video.local_cover_path.is_none() {
            if let Err(e) = tiktok::download_video_content(&mut video).await {
                error!(
                    "Failed to download content for legacy video {}: {}",
                    video.id, e
                );
                return Err(e);
            }
            info!("Downloaded content for legacy video: {}", video.id);
        }

        // Process content (add hashtags and taglines)
        let processed_content = content::process_content(video).await?;

        // Post to all platforms
        match platforms::post_to_all_platforms(processed_content.clone()).await {
            Ok(_) => {
                info!(
                    "Successfully posted legacy video to all platforms: {}",
                    processed_content.original_video.id
                );

                // Record that this content has been posted
                scheduler::mark_as_posted(&processed_content.original_video.id).await?;

                info!("Successfully posted legacy content");
            },
            Err(e) => {
                error!("Failed to post legacy video to all platforms: {}", e);
                return Err(e);
            },
        }
    } else {
        info!("No more legacy content to post");
    }

    Ok(())
}
