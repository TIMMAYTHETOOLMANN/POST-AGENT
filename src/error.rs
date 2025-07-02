use thiserror::Error;

/// Custom error types for the TikTok cross-posting application
#[derive(Error, Debug)]
pub enum AppError {
    /// Error when fetching content from TikTok
    #[error("Failed to fetch TikTok content: {0}")]
    TikTokFetchError(String),

    /// Error when processing content
    #[error("Failed to process content: {0}")]
    ContentProcessingError(String),

    /// Error when posting to Facebook
    #[error("Failed to post to Facebook: {0}")]
    FacebookPostError(String),

    /// Error when posting to Instagram
    #[error("Failed to post to Instagram: {0}")]
    InstagramPostError(String),

    /// Error when posting to YouTube
    #[error("Failed to post to YouTube: {0}")]
    YouTubePostError(String),

    /// Error when loading configuration
    #[error("Failed to load configuration: {0}")]
    ConfigError(String),

    /// Error when scheduling tasks
    #[error("Failed to schedule task: {0}")]
    SchedulerError(String),

    /// Error when downloading media
    #[error("Failed to download media: {0}")]
    MediaDownloadError(String),

    /// Error when uploading media
    #[error("Failed to upload media: {0}")]
    MediaUploadError(String),

    /// Error when authenticating with a service
    #[error("Authentication failed: {0}")]
    AuthenticationError(String),

    /// Error when rate limited by a service
    #[error("Rate limited by service: {0}")]
    RateLimitError(String),

    /// Error when a required file is not found
    #[error("File not found: {0}")]
    FileNotFoundError(String),

    /// Error when a database operation fails
    #[error("Database error: {0}")]
    DatabaseError(String),

    /// Generic error for other cases
    #[error("An error occurred: {0}")]
    GenericError(String),

    /// Error from an external library
    #[error("External error: {0}")]
    ExternalError(#[from] anyhow::Error),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Serialization/deserialization error
    #[error("Serialization error: {0}")]
    SerdeError(#[from] serde_json::Error),

    /// HTTP request error
    #[error("HTTP request error: {0}")]
    HttpError(#[from] reqwest::Error),

    /// TOML serialization/deserialization error
    #[error("TOML serialization error: {0}")]
    TomlError(#[from] toml::ser::Error),
}

/// Result type alias for the application
pub type AppResult<T> = std::result::Result<T, AppError>;

/// Convert a string error to an AppError::GenericError
pub fn to_app_error<E: std::fmt::Display>(err: E) -> AppError {
    AppError::GenericError(err.to_string())
}

/// Helper function to convert any error to an AppResult
pub fn to_app_result<T, E: std::fmt::Display>(result: std::result::Result<T, E>) -> AppResult<T> {
    result.map_err(to_app_error)
}
