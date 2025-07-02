use crate::config;
use crate::error::{AppError, AppResult};
use crate::tiktok::TikTokVideo;
use log::{info, debug};
use rand::seq::SliceRandom;
use rand::Rng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use chrono::Utc;

/// Represents processed content ready for posting to other platforms
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessedContent {
    /// Original TikTok video
    pub original_video: TikTokVideo,

    /// Enhanced description with tagline and hashtags
    pub enhanced_description: String,

    /// Trending hashtags added to the content
    pub trending_hashtags: Vec<String>,

    /// Tagline added to the content
    pub tagline: String,

    /// Timestamp when the content was processed
    pub processed_at: chrono::DateTime<Utc>,

    /// Target platforms for this content
    pub target_platforms: Vec<String>,
}

/// Process a TikTok video to enhance it with taglines and hashtags
pub async fn process_content(video: TikTokVideo) -> AppResult<ProcessedContent> {
    info!("Processing TikTok video: {}", video.id);

    let config = config::get_config();

    // Generate a scroll-stopping tagline
    let tagline = generate_tagline(&config.content.tagline_templates)?;
    debug!("Generated tagline: {}", tagline);

    // Generate trending hashtags
    let trending_hashtags = generate_trending_hashtags(
        &video.hashtags,
        &config.content.custom_hashtags,
        config.content.hashtag_count,
        config.content.use_trending_hashtags,
    ).await?;
    debug!("Generated hashtags: {:?}", trending_hashtags);

    // Combine original description, tagline, and hashtags
    let hashtag_string = trending_hashtags
        .iter()
        .map(|tag| format!("#{}", tag))
        .collect::<Vec<String>>()
        .join(" ");

    let enhanced_description = format!("{}\n\n{}\n\n{}", tagline, video.description, hashtag_string);
    debug!("Enhanced description: {}", enhanced_description);

    // Define target platforms
    let target_platforms = vec![
        "facebook".to_string(),
        "instagram".to_string(),
        "youtube".to_string(),
    ];

    // Create processed content
    let processed_content = ProcessedContent {
        original_video: video,
        enhanced_description,
        trending_hashtags,
        tagline,
        processed_at: Utc::now(),
        target_platforms,
    };

    info!("Successfully processed TikTok video: {}", processed_content.original_video.id);

    Ok(processed_content)
}

/// Generate a scroll-stopping tagline from templates
fn generate_tagline(templates: &[String]) -> AppResult<String> {
    // Use StdRng which is Send + Sync
    let mut rng = rand::rngs::StdRng::from_entropy();

    if templates.is_empty() {
        return Err(AppError::ContentProcessingError(
            "No tagline templates available".to_string(),
        ));
    }

    // Randomly select a tagline template
    let tagline = templates
        .choose(&mut rng)
        .ok_or_else(|| {
            AppError::ContentProcessingError("Failed to select a tagline template".to_string())
        })?
        .clone();

    Ok(tagline)
}

/// Generate trending hashtags for the content
async fn generate_trending_hashtags(
    original_hashtags: &[String],
    custom_hashtags: &[String],
    hashtag_count: usize,
    use_trending: bool,
) -> AppResult<Vec<String>> {
    // Use StdRng which is Send + Sync
    let mut rng = rand::rngs::StdRng::from_entropy();
    let mut hashtags = HashSet::new();

    // Add original hashtags
    for tag in original_hashtags {
        hashtags.insert(tag.clone());
    }

    // Add custom hashtags
    for tag in custom_hashtags {
        hashtags.insert(tag.clone());
    }

    // Add trending hashtags if enabled
    if use_trending {
        let trending = fetch_trending_hashtags().await?;

        // Add some trending hashtags
        let trending_to_add = trending
            .choose_multiple(&mut rng, hashtag_count.saturating_sub(hashtags.len()))
            .cloned();

        for tag in trending_to_add {
            hashtags.insert(tag);
        }
    }

    // Ensure we have enough hashtags
    while hashtags.len() < hashtag_count {
        let generic_tags = vec![
            "viral".to_string(),
            "trending".to_string(),
            "fyp".to_string(),
            "foryou".to_string(),
            "foryoupage".to_string(),
            "tiktok".to_string(),
            "tiktokviral".to_string(),
            "explore".to_string(),
            "explorepage".to_string(),
            "trend".to_string(),
        ];

        let tag = generic_tags
            .choose(&mut rng)
            .ok_or_else(|| {
                AppError::ContentProcessingError("Failed to select a generic hashtag".to_string())
            })?
            .clone();

        hashtags.insert(tag);
    }

    // Convert to vector and limit to requested count
    let mut hashtag_vec: Vec<String> = hashtags.into_iter().collect();
    hashtag_vec.truncate(hashtag_count);

    Ok(hashtag_vec)
}

/// Fetch trending hashtags from an API or service
async fn fetch_trending_hashtags() -> AppResult<Vec<String>> {
    // In a real implementation, this would fetch trending hashtags from an API
    // For now, we'll return a static list of simulated trending hashtags

    Ok(vec![
        "trending".to_string(),
        "viral".to_string(),
        "fyp".to_string(),
        "foryou".to_string(),
        "tiktok".to_string(),
        "dance".to_string(),
        "music".to_string(),
        "funny".to_string(),
        "comedy".to_string(),
        "meme".to_string(),
        "challenge".to_string(),
        "duet".to_string(),
        "pov".to_string(),
        "foryoupage".to_string(),
        "trending2023".to_string(),
        "viralvideo".to_string(),
        "tiktokviral".to_string(),
        "explore".to_string(),
        "explorepage".to_string(),
        "trend".to_string(),
    ])
}
