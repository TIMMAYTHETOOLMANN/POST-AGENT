use crate::config;
use crate::content::ProcessedContent;
use crate::error::{AppError, AppResult};
use crate::browser::BrowserManager;
use log::{info, warn, error};
use std::time::Duration;
use std::path::Path;

/// Post content to all configured platforms
pub async fn post_to_all_platforms(content: ProcessedContent) -> AppResult<()> {
    info!("Posting content to all platforms: {}", content.original_video.id);
    
    let mut success_count = 0;
    let mut errors = Vec::new();
    
    // Post to Facebook
    match post_to_facebook(&content).await {
        Ok(_) => {
            info!("Successfully posted to Facebook: {}", content.original_video.id);
            success_count += 1;
        }
        Err(e) => {
            error!("Failed to post to Facebook: {}", e);
            errors.push(format!("Facebook: {}", e));
        }
    }
    
    // Post to Instagram
    match post_to_instagram(&content).await {
        Ok(_) => {
            info!("Successfully posted to Instagram: {}", content.original_video.id);
            success_count += 1;
        }
        Err(e) => {
            error!("Failed to post to Instagram: {}", e);
            errors.push(format!("Instagram: {}", e));
        }
    }
    
    // Post to YouTube
    match post_to_youtube(&content).await {
        Ok(_) => {
            info!("Successfully posted to YouTube: {}", content.original_video.id);
            success_count += 1;
        }
        Err(e) => {
            error!("Failed to post to YouTube: {}", e);
            errors.push(format!("YouTube: {}", e));
        }
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
    
    info!("Successfully posted to Facebook: {}", content.original_video.id);
    
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
    
    info!("Successfully posted to Instagram: {}", content.original_video.id);
    
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
    
    info!("Successfully posted to YouTube: {}", content.original_video.id);
    
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
    
    let browser_manager = BrowserManager::new()?;
    let tab = browser_manager.new_tab()?;
    
    // Navigate to Facebook login page
    browser_manager.navigate_with_retry(&tab, "https://www.facebook.com/login").await?;
    
    // Log in to Facebook
    browser_manager.wait_for_element_with_retry(&tab, "input[name='email']").await?;
    browser_manager.type_text_with_retry(&tab, "input[name='email']", email).await?;
    browser_manager.type_text_with_retry(&tab, "input[name='pass']", password).await?;
    browser_manager.click_with_retry(&tab, "button[name='login']").await?;
    
    // Wait for login to complete
    browser_manager.wait(Duration::from_secs(3)).await;
    
    // Navigate to create post page
    browser_manager.navigate_with_retry(&tab, "https://www.facebook.com/").await?;
    
    // Click on "What's on your mind?" or similar
    browser_manager.wait_for_element_with_retry(&tab, "[data-testid='status-attachment-mentions-input']").await?;
    browser_manager.click_with_retry(&tab, "[data-testid='status-attachment-mentions-input']").await?;
    
    // Wait for the post creation dialog to appear
    browser_manager.wait(Duration::from_secs(2)).await;
    
    // Add video file
    browser_manager.wait_for_element_with_retry(&tab, "input[type='file']").await?;
    browser_manager.upload_file_with_retry(&tab, "input[type='file']", video_path).await?;
    
    // Wait for video to upload
    browser_manager.wait(Duration::from_secs(5)).await;
    
    // Add description
    browser_manager.wait_for_element_with_retry(&tab, "[data-testid='treeitem-composer-text-input']").await?;
    browser_manager.type_text_with_retry(&tab, "[data-testid='treeitem-composer-text-input']", description).await?;
    
    // Click post button
    browser_manager.wait_for_element_with_retry(&tab, "[data-testid='react-composer-post-button']").await?;
    browser_manager.click_with_retry(&tab, "[data-testid='react-composer-post-button']").await?;
    
    // Wait for post to be published
    browser_manager.wait(Duration::from_secs(10)).await;
    
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
    
    let browser_manager = BrowserManager::new()?;
    let tab = browser_manager.new_tab()?;
    
    // Navigate to Instagram login page
    browser_manager.navigate_with_retry(&tab, "https://www.instagram.com/accounts/login/").await?;
    
    // Log in to Instagram
    browser_manager.wait_for_element_with_retry(&tab, "input[name='username']").await?;
    browser_manager.type_text_with_retry(&tab, "input[name='username']", email).await?;
    browser_manager.type_text_with_retry(&tab, "input[name='password']", password).await?;
    browser_manager.click_with_retry(&tab, "button[type='submit']").await?;
    
    // Wait for login to complete
    browser_manager.wait(Duration::from_secs(3)).await;
    
    // Navigate to home page
    browser_manager.navigate_with_retry(&tab, "https://www.instagram.com/").await?;
    
    // Click on "Create" button (plus icon)
    browser_manager.wait_for_element_with_retry(&tab, "[aria-label='New post']").await?;
    browser_manager.click_with_retry(&tab, "[aria-label='New post']").await?;
    
    // Upload video file
    browser_manager.wait_for_element_with_retry(&tab, "input[type='file']").await?;
    browser_manager.upload_file_with_retry(&tab, "input[type='file']", video_path).await?;
    
    // Wait for video to process
    browser_manager.wait(Duration::from_secs(5)).await;
    
    // Click "Next" button
    browser_manager.wait_for_element_with_retry(&tab, "button").await?;
    browser_manager.click_with_retry(&tab, "button:contains('Next')").await?;
    
    // Click "Next" again for filters/editing
    browser_manager.wait(Duration::from_secs(2)).await;
    browser_manager.click_with_retry(&tab, "button:contains('Next')").await?;
    
    // Add caption/description
    browser_manager.wait_for_element_with_retry(&tab, "textarea[aria-label='Write a caption...']").await?;
    browser_manager.type_text_with_retry(&tab, "textarea[aria-label='Write a caption...']", description).await?;
    
    // Click "Share" button
    browser_manager.wait_for_element_with_retry(&tab, "button:contains('Share')").await?;
    browser_manager.click_with_retry(&tab, "button:contains('Share')").await?;
    
    // Wait for post to be published
    browser_manager.wait(Duration::from_secs(10)).await;
    
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
    
    let browser_manager = BrowserManager::new()?;
    let tab = browser_manager.new_tab()?;
    
    // Navigate to YouTube Studio
    browser_manager.navigate_with_retry(&tab, "https://studio.youtube.com").await?;
    
    // Check if we need to login
    if let Ok(_) = tab.find_element("input[type='email']") {
        // Login to Google/YouTube
        browser_manager.type_text_with_retry(&tab, "input[type='email']", email).await?;
        browser_manager.click_with_retry(&tab, "#identifierNext").await?;
        
        // Wait for password field
        browser_manager.wait(Duration::from_secs(2)).await;
        browser_manager.wait_for_element_with_retry(&tab, "input[type='password']").await?;
        browser_manager.type_text_with_retry(&tab, "input[type='password']", password).await?;
        browser_manager.click_with_retry(&tab, "#passwordNext").await?;
        
        // Wait for login to complete
        browser_manager.wait(Duration::from_secs(5)).await;
    }
    
    // Click on "CREATE" button
    browser_manager.wait_for_element_with_retry(&tab, "#create-icon").await?;
    browser_manager.click_with_retry(&tab, "#create-icon").await?;
    
    // Click on "Upload videos"
    browser_manager.wait_for_element_with_retry(&tab, "#text-item-0").await?;
    browser_manager.click_with_retry(&tab, "#text-item-0").await?;
    
    // Upload video file
    browser_manager.wait_for_element_with_retry(&tab, "input[type='file']").await?;
    browser_manager.upload_file_with_retry(&tab, "input[type='file']", video_path).await?;
    
    // Wait for video to upload
    browser_manager.wait(Duration::from_secs(10)).await;
    
    // Add title (use first part of description)
    let title = description.split('\n').next().unwrap_or(description);
    browser_manager.wait_for_element_with_retry(&tab, "#textbox[aria-label='Add a title']").await?;
    browser_manager.type_text_with_retry(&tab, "#textbox[aria-label='Add a title']", title).await?;
    
    // Add description
    browser_manager.wait_for_element_with_retry(&tab, "#textbox[aria-label='Tell viewers about your video']").await?;
    browser_manager.type_text_with_retry(&tab, "#textbox[aria-label='Tell viewers about your video']", description).await?;
    
    // Click "Next" for audience settings
    browser_manager.wait_for_element_with_retry(&tab, "#next-button").await?;
    browser_manager.click_with_retry(&tab, "#next-button").await?;
    
    // Click "Next" for visibility settings
    browser_manager.wait(Duration::from_secs(2)).await;
    browser_manager.click_with_retry(&tab, "#next-button").await?;
    
    // Click "Next" for checks
    browser_manager.wait(Duration::from_secs(2)).await;
    browser_manager.click_with_retry(&tab, "#next-button").await?;
    
    // Set visibility to public and publish
    browser_manager.wait_for_element_with_retry(&tab, "tp-yt-paper-radio-button[name='PUBLIC']").await?;
    browser_manager.click_with_retry(&tab, "tp-yt-paper-radio-button[name='PUBLIC']").await?;
    
    // Click "Publish"
    browser_manager.wait_for_element_with_retry(&tab, "#done-button").await?;
    browser_manager.click_with_retry(&tab, "#done-button").await?;
    
    // Wait for publishing to complete
    browser_manager.wait(Duration::from_secs(15)).await;
    
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
    info!("Testing Facebook authentication via browser automation");
    
    let config = config::get_config();
    let facebook_config = &config.facebook;
    
    let browser_manager = BrowserManager::new()?;
    let tab = browser_manager.new_tab()?;
    
    // Navigate to Facebook login page
    browser_manager.navigate_with_retry(&tab, "https://www.facebook.com/login").await?;
    
    // Test login
    browser_manager.wait_for_element_with_retry(&tab, "input[name='email']").await?;
    browser_manager.type_text_with_retry(&tab, "input[name='email']", &facebook_config.email).await?;
    browser_manager.type_text_with_retry(&tab, "input[name='pass']", &facebook_config.password).await?;
    browser_manager.click_with_retry(&tab, "button[name='login']").await?;
    
    // Wait for login to complete
    browser_manager.wait(Duration::from_secs(3)).await;
    
    // Check if we're logged in successfully (look for home page elements)
    match browser_manager.wait_for_element_with_retry(&tab, "[data-testid='react-composer-root']").await {
        Ok(_) => {
            info!("Successfully authenticated with Facebook");
            Ok(())
        }
        Err(_) => {
            warn!("Facebook authentication may have failed - check credentials");
            Ok(()) // Don't fail the entire app startup
        }
    }
}

/// Authenticate with Instagram
async fn authenticate_instagram() -> AppResult<()> {
    info!("Testing Instagram authentication via browser automation");
    
    let config = config::get_config();
    let instagram_config = &config.instagram;
    
    let browser_manager = BrowserManager::new()?;
    let tab = browser_manager.new_tab()?;
    
    // Navigate to Instagram login page
    browser_manager.navigate_with_retry(&tab, "https://www.instagram.com/accounts/login/").await?;
    
    // Test login
    browser_manager.wait_for_element_with_retry(&tab, "input[name='username']").await?;
    browser_manager.type_text_with_retry(&tab, "input[name='username']", &instagram_config.email).await?;
    browser_manager.type_text_with_retry(&tab, "input[name='password']", &instagram_config.password).await?;
    browser_manager.click_with_retry(&tab, "button[type='submit']").await?;
    
    // Wait for login to complete
    browser_manager.wait(Duration::from_secs(3)).await;
    
    // Check if we're logged in successfully
    match browser_manager.wait_for_element_with_retry(&tab, "[aria-label='Home']").await {
        Ok(_) => {
            info!("Successfully authenticated with Instagram");
            Ok(())
        }
        Err(_) => {
            warn!("Instagram authentication may have failed - check credentials");
            Ok(()) // Don't fail the entire app startup
        }
    }
}

/// Authenticate with YouTube
async fn authenticate_youtube() -> AppResult<()> {
    info!("Testing YouTube authentication via browser automation");
    
    let config = config::get_config();
    let youtube_config = &config.youtube;
    
    let browser_manager = BrowserManager::new()?;
    let tab = browser_manager.new_tab()?;
    
    // Navigate to YouTube Studio
    browser_manager.navigate_with_retry(&tab, "https://studio.youtube.com").await?;
    
    // Check if we need to login
    if let Ok(_) = tab.find_element("input[type='email']") {
        // Login to Google/YouTube
        browser_manager.type_text_with_retry(&tab, "input[type='email']", &youtube_config.email).await?;
        browser_manager.click_with_retry(&tab, "#identifierNext").await?;
        
        // Wait for password field
        browser_manager.wait(Duration::from_secs(2)).await;
        browser_manager.wait_for_element_with_retry(&tab, "input[type='password']").await?;
        browser_manager.type_text_with_retry(&tab, "input[type='password']", &youtube_config.password).await?;
        browser_manager.click_with_retry(&tab, "#passwordNext").await?;
        
        // Wait for login to complete
        browser_manager.wait(Duration::from_secs(5)).await;
    }
    
    // Check if we're in YouTube Studio successfully
    match browser_manager.wait_for_element_with_retry(&tab, "#create-icon").await {
        Ok(_) => {
            info!("Successfully authenticated with YouTube");
            Ok(())
        }
        Err(_) => {
            warn!("YouTube authentication may have failed - check credentials");
            Ok(()) // Don't fail the entire app startup
        }
    }
}