use tiktok_crosspost::config;
use tiktok_crosspost::tiktok;
use tiktok_crosspost::content;
use tiktok_crosspost::scheduler;
use tiktok_crosspost::error::AppResult;
use chrono::Utc;

/// Test that verifies the complete workflow from TikTok scraping to content processing
#[tokio::test]
async fn test_complete_workflow() -> AppResult<()> {
    // Initialize test environment
    unsafe {
        std::env::set_var("RUST_LOG", "info");
    }
    
    // Load configuration first
    let _config = config::load_config()?;
    
    // Test scheduler initialization
    scheduler::init().await?;
    
    // Test TikTok video fetching (simulation)
    let videos = tiktok::fetch_all_videos().await?;
    assert!(!videos.is_empty());
    
    // Test content processing
    if let Some(video) = videos.first() {
        let processed = content::process_content(video.clone()).await?;
        
        // Verify that content was enhanced
        assert!(!processed.tagline.is_empty());
        assert!(!processed.trending_hashtags.is_empty());
        assert!(processed.enhanced_description.contains(&processed.tagline));
        assert!(processed.target_platforms.contains(&"facebook".to_string()));
        assert!(processed.target_platforms.contains(&"instagram".to_string()));
        assert!(processed.target_platforms.contains(&"youtube".to_string()));
        
        // Verify hashtags were added
        for hashtag in &processed.trending_hashtags {
            assert!(processed.enhanced_description.contains(&format!("#{}", hashtag)));
        }
    }
    
    Ok(())
}

/// Test that verifies the scheduling logic works correctly
#[tokio::test]
async fn test_scheduling_logic() -> AppResult<()> {
    // Load configuration first
    let _config = config::load_config()?;
    
    // Initialize scheduler
    scheduler::init().await?;
    
    // Use a unique test video ID to avoid conflicts with other tests
    let test_video_id = format!("test_video_scheduling_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0));
    
    // Initially should not be posted
    assert!(!scheduler::is_video_posted(&test_video_id).await?);
    
    // Mark as posted
    scheduler::mark_as_posted(&test_video_id).await?;
    
    // Should now be marked as posted
    assert!(scheduler::is_video_posted(&test_video_id).await?);
    
    Ok(())
}

/// Test that verifies content enhancement produces good results
#[tokio::test]
async fn test_content_enhancement() -> AppResult<()> {
    // Load configuration first
    let _config = config::load_config()?;
    
    // Create a test video
    let test_video = tiktok::TikTokVideo {
        id: "test_video_content".to_string(),
        url: "https://tiktok.com/test".to_string(),
        description: "Original description".to_string(),
        created_at: Utc::now(),
        likes: 100,
        comments: 10,
        shares: 5,
        views: 500,
        duration: 15.0,
        video_url: "https://tiktok.com/video.mp4".to_string(),
        cover_url: "https://tiktok.com/cover.jpg".to_string(),
        hashtags: vec!["original".to_string()],
        local_video_path: None,
        local_cover_path: None,
        is_posted: false,
    };
    
    // Process the content
    let processed = content::process_content(test_video).await?;
    
    // Verify enhancements
    assert!(processed.enhanced_description.len() > processed.original_video.description.len());
    assert!(!processed.tagline.is_empty());
    assert!(!processed.trending_hashtags.is_empty());
    
    // Verify the structure of the enhanced description
    let parts: Vec<&str> = processed.enhanced_description.split("\n\n").collect();
    assert_eq!(parts.len(), 3); // tagline, original description, hashtags
    
    Ok(())
}

/// Test that verifies configuration has all required fields
#[tokio::test]
async fn test_configuration_completeness() -> AppResult<()> {
    // Load configuration first
    let config = config::load_config()?;
    
    // Test that all platform configs have browser automation credentials
    assert!(!config.facebook.email.is_empty());
    assert!(!config.facebook.password.is_empty());
    assert!(!config.instagram.email.is_empty());
    assert!(!config.instagram.password.is_empty());
    assert!(!config.youtube.email.is_empty());
    assert!(!config.youtube.password.is_empty());
    
    // Test scheduler configuration
    assert_eq!(config.scheduler.legacy_post_interval_minutes, 10);
    assert_eq!(config.scheduler.check_interval_minutes, 5);
    assert!(config.scheduler.max_posts_per_day > 0);
    
    // Test content configuration
    assert!(config.content.hashtag_count > 0);
    assert!(!config.content.tagline_templates.is_empty());
    
    Ok(())
}

/// Test that verifies the platform targeting is correct
#[tokio::test]
async fn test_platform_targeting() -> AppResult<()> {
    // Load configuration first
    let _config = config::load_config()?;
    
    // Create a test video
    let test_video = tiktok::TikTokVideo {
        id: "test_platforms".to_string(),
        url: "https://tiktok.com/test".to_string(),
        description: "Platform test".to_string(),
        created_at: Utc::now(),
        likes: 100,
        comments: 10,
        shares: 5,
        views: 500,
        duration: 15.0,
        video_url: "https://tiktok.com/video.mp4".to_string(),
        cover_url: "https://tiktok.com/cover.jpg".to_string(),
        hashtags: vec!["test".to_string()],
        local_video_path: None,
        local_cover_path: None,
        is_posted: false,
    };
    
    // Process the content
    let processed = content::process_content(test_video).await?;
    
    // Verify all three platforms are targeted
    assert_eq!(processed.target_platforms.len(), 3);
    assert!(processed.target_platforms.contains(&"facebook".to_string()));
    assert!(processed.target_platforms.contains(&"instagram".to_string()));
    assert!(processed.target_platforms.contains(&"youtube".to_string()));
    
    Ok(())
}