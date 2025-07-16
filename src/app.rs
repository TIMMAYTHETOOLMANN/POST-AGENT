use crate::error::AppResult;
use crate::{content, platforms, scheduler, tiktok};
use log::{error, info, warn};
use tokio::time::Duration;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};

/// Circuit breaker for handling repeated failures
struct CircuitBreaker {
    failure_count: Arc<AtomicU32>,
    max_failures: u32,
    reset_timeout: Duration,
    last_failure_time: Arc<std::sync::Mutex<Option<std::time::Instant>>>,
}

impl CircuitBreaker {
    fn new(max_failures: u32, reset_timeout: Duration) -> Self {
        Self {
            failure_count: Arc::new(AtomicU32::new(0)),
            max_failures,
            reset_timeout,
            last_failure_time: Arc::new(std::sync::Mutex::new(None)),
        }
    }

    fn should_attempt(&self) -> bool {
        let current_failures = self.failure_count.load(Ordering::Relaxed);
        if current_failures < self.max_failures {
            return true;
        }

        // Check if we should reset the circuit breaker
        if let Some(last_failure) = *self.last_failure_time.lock().unwrap() {
            if last_failure.elapsed() > self.reset_timeout {
                info!("Circuit breaker reset after timeout");
                self.failure_count.store(0, Ordering::Relaxed);
                return true;
            }
        }

        false
    }

    fn record_success(&self) {
        self.failure_count.store(0, Ordering::Relaxed);
    }

    fn record_failure(&self) {
        self.failure_count.fetch_add(1, Ordering::Relaxed);
        *self.last_failure_time.lock().unwrap() = Some(std::time::Instant::now());
    }
}

/// Global circuit breaker for new content monitoring
static NEW_CONTENT_CIRCUIT_BREAKER: std::sync::OnceLock<CircuitBreaker> = std::sync::OnceLock::new();

/// Initialize circuit breakers
pub fn init_circuit_breakers() {
    NEW_CONTENT_CIRCUIT_BREAKER.set(CircuitBreaker::new(5, Duration::from_secs(300))).ok();
}

/// Performs the initial synchronization of all TikTok content to all platforms
pub async fn initial_sync_all_content() -> AppResult<()> {
    info!("Starting initial sync of all TikTok content");

    // Fetch all TikTok videos
    let mut videos = tiktok::fetch_all_videos().await?;
    info!("Fetched {} TikTok videos", videos.len());

    if videos.is_empty() {
        info!("No TikTok videos found to sync");
        return Ok(());
    }

    // Sort videos by creation date (oldest first)
    videos.sort_by(|a, b| a.created_at.cmp(&b.created_at));

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

        // Add a small delay between posts to avoid rate limiting
        tokio::time::sleep(Duration::from_secs(2)).await;
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

    // Check circuit breaker
    let circuit_breaker = NEW_CONTENT_CIRCUIT_BREAKER.get()
        .expect("Circuit breaker not initialized");
    
    if !circuit_breaker.should_attempt() {
        warn!("Circuit breaker is open, skipping new content check");
        return Ok(());
    }

    // Fetch new TikTok videos
    let new_videos = match tiktok::fetch_new_videos().await {
        Ok(videos) => videos,
        Err(e) => {
            error!("Failed to fetch new videos: {}", e);
            circuit_breaker.record_failure();
            return Err(e);
        }
    };

    info!("Found {} new TikTok videos", new_videos.len());

    if new_videos.is_empty() {
        circuit_breaker.record_success();
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
            circuit_breaker.record_failure();
            continue;
        }
        info!("Downloaded content for video: {}", video.id);

        // Process content (add hashtags and taglines)
        let processed_content = match content::process_content(video).await {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to process content: {}", e);
                circuit_breaker.record_failure();
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
                circuit_breaker.record_failure();
                // Continue with next video
            },
        }
    }

    if processed_count > 0 {
        circuit_breaker.record_success();
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
