use crate::config;
use crate::content::ProcessedContent;
use crate::error::{AppError, AppResult};
use log::{error, info, warn};
use std::path::Path;

/// Post content to all configured platforms
pub async fn post_to_all_platforms(content: ProcessedContent) -> AppResult<()> {
    info!(
        "Posting content to all platforms: {}",
        content.original_video.id
    );

    let mut success_count = 0;
    let mut errors = Vec::new();

    // Post to Facebook
    match post_to_facebook(&content).await {
        Ok(_) => {
            info!(
                "Successfully posted to Facebook: {}",
                content.original_video.id
            );
            success_count += 1;
        },
        Err(e) => {
            error!("Failed to post to Facebook: {}", e);
            errors.push(format!("Facebook: {}", e));
        },
    }

    // Post to Instagram
    match post_to_instagram(&content).await {
        Ok(_) => {
            info!(
                "Successfully posted to Instagram: {}",
                content.original_video.id
            );
            success_count += 1;
        },
        Err(e) => {
            error!("Failed to post to Instagram: {}", e);
            errors.push(format!("Instagram: {}", e));
        },
    }

    // Post to YouTube
    match post_to_youtube(&content).await {
        Ok(_) => {
            info!(
                "Successfully posted to YouTube: {}",
                content.original_video.id
            );
            success_count += 1;
        },
        Err(e) => {
            error!("Failed to post to YouTube: {}", e);
            errors.push(format!("YouTube: {}", e));
        },
    }

    // Check if we had any successful posts
    if success_count == 0 {
        return Err(AppError::MediaUploadError(format!(
            "Failed to post to any platform: {}",
            errors.join(", ")
        )));
    }

    info!(
        "Posted content to {}/{} platforms: {}",
        success_count,
        content.target_platforms.len(),
        content.original_video.id
    );

    Ok(())
}

/// Post content to Facebook
async fn post_to_facebook(content: &ProcessedContent) -> AppResult<()> {
    info!("Posting to Facebook: {}", content.original_video.id);

    let config = config::get_config();
    let facebook_config = &config.facebook;

    // Ensure we have a local video file
    let video_path = content
        .original_video
        .local_video_path
        .as_ref()
        .ok_or_else(|| {
            AppError::MediaUploadError("No local video file available for upload".to_string())
        })?;

    // Use browser automation to post to Facebook
    browser_facebook_post(
        &facebook_config.email,
        &facebook_config.password,
        &content.enhanced_description,
        video_path,
    )
    .await?;

    info!(
        "Successfully posted to Facebook: {}",
        content.original_video.id
    );

    Ok(())
}

/// Post content to Instagram
async fn post_to_instagram(content: &ProcessedContent) -> AppResult<()> {
    info!("Posting to Instagram: {}", content.original_video.id);

    let config = config::get_config();
    let instagram_config = &config.instagram;

    // Ensure we have a local video file
    let video_path = content
        .original_video
        .local_video_path
        .as_ref()
        .ok_or_else(|| {
            AppError::MediaUploadError("No local video file available for upload".to_string())
        })?;

    // Use browser automation to post to Instagram
    browser_instagram_post(
        &instagram_config.email,
        &instagram_config.password,
        &content.enhanced_description,
        video_path,
    )
    .await?;

    info!(
        "Successfully posted to Instagram: {}",
        content.original_video.id
    );

    Ok(())
}

/// Post content to YouTube
async fn post_to_youtube(content: &ProcessedContent) -> AppResult<()> {
    info!("Posting to YouTube: {}", content.original_video.id);

    let config = config::get_config();
    let youtube_config = &config.youtube;

    // Ensure we have a local video file
    let video_path = content
        .original_video
        .local_video_path
        .as_ref()
        .ok_or_else(|| {
            AppError::MediaUploadError("No local video file available for upload".to_string())
        })?;

    // Use browser automation to post to YouTube
    browser_youtube_post(
        &youtube_config.email,
        &youtube_config.password,
        &content.enhanced_description,
        video_path,
    )
    .await?;

    info!(
        "Successfully posted to YouTube: {}",
        content.original_video.id
    );

    Ok(())
}

/// Post to Facebook using browser automation
async fn browser_facebook_post(
    email: &str,
    password: &str,
    description: &str,
    video_path: &Path,
) -> AppResult<()> {
    info!("Starting Facebook post via browser automation");

    // Use real browser automation for Facebook posting
    use crate::browser::FacebookBrowser;
    
    let mut browser = FacebookBrowser::new().await?;
    
    // Login to Facebook
    browser.login(email, password).await?;
    
    // Navigate to video upload page
    browser.navigate_to_video_upload().await?;
    
    // Upload video file
    browser.upload_video(video_path).await?;
    
    // Add description with dynamic hashtags
    browser.add_description(description).await?;
    
    // Set privacy and posting options
    browser.set_posting_options().await?;
    
    // Submit the post
    browser.submit_post().await?;
    
    info!("Facebook post completed successfully");
    Ok(())
}

/// Post to Instagram using browser automation
async fn browser_instagram_post(
    email: &str,
    password: &str,
    description: &str,
    video_path: &Path,
) -> AppResult<()> {
    info!("Starting Instagram post via browser automation");

    // Use real browser automation for Instagram posting
    use crate::browser::InstagramBrowser;
    
    let mut browser = InstagramBrowser::new().await?;
    
    // Login to Instagram
    browser.login(email, password).await?;
    
    // Navigate to video upload page
    browser.navigate_to_create_post().await?;
    
    // Upload video file
    browser.upload_video(video_path).await?;
    
    // Add description with dynamic hashtags
    browser.add_description(description).await?;
    
    // Set posting options
    browser.set_posting_options().await?;
    
    // Submit the post
    browser.submit_post().await?;
    
    info!("Instagram post completed successfully");
    Ok(())
}

/// Post to YouTube using browser automation
async fn browser_youtube_post(
    email: &str,
    password: &str,
    description: &str,
    video_path: &Path,
) -> AppResult<()> {
    info!("Starting YouTube post via browser automation");

    // Use real browser automation for YouTube posting
    use crate::browser::YouTubeBrowser;
    
    let mut browser = YouTubeBrowser::new().await?;
    
    // Login to YouTube
    browser.login(email, password).await?;
    
    // Navigate to video upload page
    browser.navigate_to_upload().await?;
    
    // Upload video file
    browser.upload_video(video_path).await?;
    
    // Add title and description with dynamic hashtags
    browser.add_title_and_description(description).await?;
    
    // Set video options (visibility, category, etc.)
    browser.set_video_options().await?;
    
    // Submit the video
    browser.submit_video().await?;
    
    info!("YouTube post completed successfully");
    Ok(())
}

/// Authenticate with all social media platforms
pub async fn authenticate_all_platforms() -> AppResult<()> {
    info!("Authenticating with all social media platforms");

    // Authenticate with Facebook
    match authenticate_facebook().await {
        Ok(_) => info!("Successfully authenticated with Facebook"),
        Err(e) => warn!("Failed to authenticate with Facebook: {}", e),
    }

    // Authenticate with Instagram
    match authenticate_instagram().await {
        Ok(_) => info!("Successfully authenticated with Instagram"),
        Err(e) => warn!("Failed to authenticate with Instagram: {}", e),
    }

    // Authenticate with YouTube
    match authenticate_youtube().await {
        Ok(_) => info!("Successfully authenticated with YouTube"),
        Err(e) => warn!("Failed to authenticate with YouTube: {}", e),
    }

    info!("Authentication with all platforms completed");

    Ok(())
}

/// Authenticate with Facebook
async fn authenticate_facebook() -> AppResult<()> {
    info!("Authenticating with Facebook via browser automation");

    let config = config::get_config();
    let facebook_config = &config.facebook;

    // Use real browser automation for Facebook authentication
    use crate::browser::FacebookBrowser;
    
    let mut browser = FacebookBrowser::new().await?;
    
    // Test login
    browser.login(&facebook_config.email, &facebook_config.password).await?;
    
    // Verify authentication was successful
    browser.verify_authentication().await?;
    
    info!("Facebook authentication completed successfully");
    Ok(())
}

/// Authenticate with Instagram
async fn authenticate_instagram() -> AppResult<()> {
    info!("Authenticating with Instagram via browser automation");

    let config = config::get_config();
    let instagram_config = &config.instagram;

    // Use real browser automation for Instagram authentication
    use crate::browser::InstagramBrowser;
    
    let mut browser = InstagramBrowser::new().await?;
    
    // Test login
    browser.login(&instagram_config.email, &instagram_config.password).await?;
    
    // Verify authentication was successful
    browser.verify_authentication().await?;
    
    info!("Instagram authentication completed successfully");
    Ok(())
}

/// Authenticate with YouTube
async fn authenticate_youtube() -> AppResult<()> {
    info!("Authenticating with YouTube via browser automation");

    let config = config::get_config();
    let youtube_config = &config.youtube;

    // Use real browser automation for YouTube authentication
    use crate::browser::YouTubeBrowser;
    
    let mut browser = YouTubeBrowser::new().await?;
    
    // Test login
    browser.login(&youtube_config.email, &youtube_config.password).await?;
    
    // Verify authentication was successful
    browser.verify_authentication().await?;
    
    info!("YouTube authentication completed successfully");
    Ok(())
}
