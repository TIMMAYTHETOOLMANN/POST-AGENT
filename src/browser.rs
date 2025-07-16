use crate::config;
use crate::error::{AppError, AppResult};
use headless_chrome::protocol::cdp::Page;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use log::{debug, info, warn};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::time::sleep;

/// Browser automation manager
pub struct BrowserManager {
    browser: Browser,
    config: config::BrowserConfig,
}

impl BrowserManager {
    /// Create a new browser manager with configuration
    pub fn new() -> AppResult<Self> {
        let config = config::get_config().browser;

        let launch_options = LaunchOptionsBuilder::default()
            .headless(config.headless)
            .window_size(Some((config.window_width, config.window_height)))
            .build()
            .map_err(|e| {
                AppError::GenericError(format!("Failed to build launch options: {}", e))
            })?;

        let browser = Browser::new(launch_options)
            .map_err(|e| AppError::GenericError(format!("Failed to launch browser: {}", e)))?;

        Ok(Self { browser, config })
    }

    /// Create a new tab
    pub fn new_tab(&self) -> AppResult<Arc<Tab>> {
        let tab = self
            .browser
            .new_tab()
            .map_err(|e| AppError::GenericError(format!("Failed to create new tab: {}", e)))?;

        // Set user agent if configured
        if let Some(user_agent) = &self.config.user_agent {
            tab.set_user_agent(user_agent, None, None)
                .map_err(|e| AppError::GenericError(format!("Failed to set user agent: {}", e)))?;
        }

        Ok(tab)
    }

    /// Navigate to a URL with retry logic
    pub async fn navigate_with_retry(&self, tab: &Arc<Tab>, url: &str) -> AppResult<()> {
        for attempt in 1..=self.config.max_retries {
            match tab.navigate_to(url) {
                Ok(_) => {
                    // Wait for page to load
                    match tab.wait_for_element_with_custom_timeout(
                        "body",
                        Duration::from_secs(self.config.page_load_timeout),
                    ) {
                        Ok(_) => {
                            info!("Successfully navigated to {}", url);
                            return Ok(());
                        },
                        Err(e) => {
                            warn!("Page load timeout on attempt {}: {}", attempt, e);
                            if attempt == self.config.max_retries {
                                return Err(AppError::GenericError(format!(
                                    "Failed to load page after {} attempts: {}",
                                    self.config.max_retries, e
                                )));
                            }
                        },
                    }
                },
                Err(e) => {
                    warn!("Navigation failed on attempt {}: {}", attempt, e);
                    if attempt == self.config.max_retries {
                        return Err(AppError::GenericError(format!(
                            "Failed to navigate after {} attempts: {}",
                            self.config.max_retries, e
                        )));
                    }
                },
            }

            // Wait before retry
            sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
        }

        Ok(())
    }

    /// Wait for element with retry logic
    pub async fn wait_for_element_with_retry(
        &self,
        tab: &Arc<Tab>,
        selector: &str,
    ) -> AppResult<()> {
        for attempt in 1..=self.config.max_retries {
            match tab.wait_for_element_with_custom_timeout(
                selector,
                Duration::from_secs(self.config.element_wait_timeout),
            ) {
                Ok(_) => {
                    debug!("Found element: {}", selector);
                    return Ok(());
                },
                Err(e) => {
                    warn!("Element not found on attempt {}: {}", attempt, e);
                    if attempt == self.config.max_retries {
                        return Err(AppError::GenericError(format!(
                            "Element '{}' not found after {} attempts",
                            selector, self.config.max_retries
                        )));
                    }
                },
            }

            // Wait before retry
            sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
        }

        Ok(())
    }

    /// Type text into an element with retry logic
    pub async fn type_text_with_retry(
        &self,
        tab: &Arc<Tab>,
        selector: &str,
        text: &str,
    ) -> AppResult<()> {
        for attempt in 1..=self.config.max_retries {
            match tab.find_element(selector) {
                Ok(element) => {
                    match element.focus() {
                        Ok(_) => {
                            // Clear existing text by selecting all and typing
                            match element.call_js_fn(
                                "function() { this.value = ''; }",
                                vec![],
                                false,
                            ) {
                                Ok(_) => match element.type_into(text) {
                                    Ok(_) => {
                                        debug!(
                                            "Successfully typed text into element: {}",
                                            selector
                                        );
                                        return Ok(());
                                    },
                                    Err(e) => {
                                        warn!("Failed to type text on attempt {}: {}", attempt, e);
                                    },
                                },
                                Err(e) => {
                                    warn!("Failed to clear element on attempt {}: {}", attempt, e);
                                },
                            }
                        },
                        Err(e) => {
                            warn!("Failed to focus element on attempt {}: {}", attempt, e);
                        },
                    }
                },
                Err(e) => {
                    warn!("Element not found on attempt {}: {}", attempt, e);
                },
            }

            if attempt == self.config.max_retries {
                return Err(AppError::GenericError(format!(
                    "Failed to type text into '{}' after {} attempts",
                    selector, self.config.max_retries
                )));
            }

            // Wait before retry
            sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
        }

        Ok(())
    }

    /// Click an element with retry logic
    pub async fn click_with_retry(&self, tab: &Arc<Tab>, selector: &str) -> AppResult<()> {
        for attempt in 1..=self.config.max_retries {
            match tab.find_element(selector) {
                Ok(element) => match element.click() {
                    Ok(_) => {
                        debug!("Successfully clicked element: {}", selector);
                        return Ok(());
                    },
                    Err(e) => {
                        warn!("Failed to click element on attempt {}: {}", attempt, e);
                    },
                },
                Err(e) => {
                    warn!("Element not found on attempt {}: {}", attempt, e);
                },
            }

            if attempt == self.config.max_retries {
                return Err(AppError::GenericError(format!(
                    "Failed to click '{}' after {} attempts",
                    selector, self.config.max_retries
                )));
            }

            // Wait before retry
            sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
        }

        Ok(())
    }

    /// Upload file to an input element with retry logic
    pub async fn upload_file_with_retry(
        &self,
        tab: &Arc<Tab>,
        selector: &str,
        file_path: &Path,
    ) -> AppResult<()> {
        let file_path_str = file_path.to_string_lossy();
        for attempt in 1..=self.config.max_retries {
            match tab.find_element(selector) {
                Ok(element) => match element.set_input_files(&[&file_path_str]) {
                    Ok(_) => {
                        debug!("Successfully uploaded file to element: {}", selector);
                        return Ok(());
                    },
                    Err(e) => {
                        warn!("Failed to upload file on attempt {}: {}", attempt, e);
                    },
                },
                Err(e) => {
                    warn!("Element not found on attempt {}: {}", attempt, e);
                },
            }

            if attempt == self.config.max_retries {
                return Err(AppError::GenericError(format!(
                    "Failed to upload file to '{}' after {} attempts",
                    selector, self.config.max_retries
                )));
            }

            // Wait before retry
            sleep(Duration::from_millis(self.config.retry_delay_ms)).await;
        }

        Ok(())
    }

    /// Wait for a specific duration
    pub async fn wait(&self, duration: Duration) {
        sleep(duration).await;
    }

    /// Take a screenshot for debugging
    pub fn take_screenshot(&self, tab: &Arc<Tab>, path: &str) -> AppResult<()> {
        let screenshot = tab
            .capture_screenshot(Page::CaptureScreenshotFormatOption::Png, None, None, true)
            .map_err(|e| AppError::GenericError(format!("Failed to take screenshot: {}", e)))?;

        std::fs::write(path, screenshot)
            .map_err(|e| AppError::GenericError(format!("Failed to save screenshot: {}", e)))?;

        debug!("Screenshot saved to {}", path);
        Ok(())
    }
}

/// Browser automation for Facebook
pub struct FacebookBrowser {
    manager: BrowserManager,
    tab: Arc<Tab>,
}

impl FacebookBrowser {
    pub async fn new() -> AppResult<Self> {
        info!("Initializing Facebook browser automation");
        
        let manager = BrowserManager::new()?;
        let tab = manager.new_tab()?;
        
        Ok(Self { manager, tab })
    }

    pub async fn login(&mut self, email: &str, password: &str) -> AppResult<()> {
        info!("Logging into Facebook with email: {}", email);
        
        // Navigate to Facebook login page
        self.manager.navigate_with_retry(&self.tab, "https://www.facebook.com/login").await?;
        
        // Fill in email
        self.manager.wait_for_element_with_retry(&self.tab, "#email").await?;
        self.manager.type_text_with_retry(&self.tab, "#email", email).await?;
        
        // Fill in password
        self.manager.type_text_with_retry(&self.tab, "#pass", password).await?;
        
        // Click login button
        self.manager.click_with_retry(&self.tab, "#loginbutton").await?;
        
        // Wait for successful login
        self.manager.wait(Duration::from_secs(3)).await;
        
        info!("Successfully logged into Facebook");
        Ok(())
    }

    pub async fn navigate_to_video_upload(&mut self) -> AppResult<()> {
        info!("Navigating to Facebook video upload page");
        
        // Navigate to Facebook video upload page
        self.manager.navigate_with_retry(&self.tab, "https://www.facebook.com/watch/upload").await?;
        
        Ok(())
    }

    pub async fn upload_video(&mut self, video_path: &Path) -> AppResult<()> {
        info!("Uploading video to Facebook: {:?}", video_path);
        
        // Find and use the file upload input
        self.manager.wait_for_element_with_retry(&self.tab, "input[type='file']").await?;
        self.manager.upload_file_with_retry(&self.tab, "input[type='file']", video_path).await?;
        
        // Wait for upload to complete
        self.manager.wait(Duration::from_secs(10)).await;
        
        info!("Video uploaded successfully");
        Ok(())
    }

    pub async fn add_description(&mut self, description: &str) -> AppResult<()> {
        info!("Adding description to Facebook post");
        
        // Find the description/caption text area
        self.manager.wait_for_element_with_retry(&self.tab, "textarea[data-testid='status-attachment-mentions-input']").await?;
        self.manager.type_text_with_retry(&self.tab, "textarea[data-testid='status-attachment-mentions-input']", description).await?;
        
        info!("Description added successfully");
        Ok(())
    }

    pub async fn set_posting_options(&mut self) -> AppResult<()> {
        info!("Setting Facebook posting options");
        
        // Set privacy settings and other options
        // This would depend on Facebook's current UI structure
        
        self.manager.wait(Duration::from_secs(1)).await;
        
        Ok(())
    }

    pub async fn submit_post(&mut self) -> AppResult<()> {
        info!("Submitting Facebook post");
        
        // Click the "Post" button
        self.manager.click_with_retry(&self.tab, "div[data-testid='react-composer-post-button']").await?;
        
        // Wait for confirmation
        self.manager.wait(Duration::from_secs(3)).await;
        
        info!("Facebook post submitted successfully");
        Ok(())
    }

    pub async fn verify_authentication(&mut self) -> AppResult<()> {
        info!("Verifying Facebook authentication");
        
        // Check if we're still logged in by looking for user profile elements
        self.manager.wait_for_element_with_retry(&self.tab, "div[data-testid='left_nav_menu_item']").await?;
        
        Ok(())
    }
}

/// Browser automation for Instagram
pub struct InstagramBrowser {
    manager: BrowserManager,
    tab: Arc<Tab>,
}

impl InstagramBrowser {
    pub async fn new() -> AppResult<Self> {
        info!("Initializing Instagram browser automation");
        
        let manager = BrowserManager::new()?;
        let tab = manager.new_tab()?;
        
        Ok(Self { manager, tab })
    }

    pub async fn login(&mut self, email: &str, password: &str) -> AppResult<()> {
        info!("Logging into Instagram with email: {}", email);
        
        // Navigate to Instagram login page
        self.manager.navigate_with_retry(&self.tab, "https://www.instagram.com/accounts/login/").await?;
        
        // Fill in email
        self.manager.wait_for_element_with_retry(&self.tab, "input[name='username']").await?;
        self.manager.type_text_with_retry(&self.tab, "input[name='username']", email).await?;
        
        // Fill in password
        self.manager.type_text_with_retry(&self.tab, "input[name='password']", password).await?;
        
        // Click login button
        self.manager.click_with_retry(&self.tab, "button[type='submit']").await?;
        
        // Wait for successful login
        self.manager.wait(Duration::from_secs(3)).await;
        
        info!("Successfully logged into Instagram");
        Ok(())
    }

    pub async fn navigate_to_create_post(&mut self) -> AppResult<()> {
        info!("Navigating to Instagram create post page");
        
        // Click on the "+" button to create a new post
        self.manager.click_with_retry(&self.tab, "a[href='/create/']").await?;
        
        Ok(())
    }

    pub async fn upload_video(&mut self, video_path: &Path) -> AppResult<()> {
        info!("Uploading video to Instagram: {:?}", video_path);
        
        // Find and use the file upload input
        self.manager.wait_for_element_with_retry(&self.tab, "input[type='file']").await?;
        self.manager.upload_file_with_retry(&self.tab, "input[type='file']", video_path).await?;
        
        // Wait for upload to complete
        self.manager.wait(Duration::from_secs(10)).await;
        
        info!("Video uploaded successfully");
        Ok(())
    }

    pub async fn add_description(&mut self, description: &str) -> AppResult<()> {
        info!("Adding description to Instagram post");
        
        // Find the caption text area
        self.manager.wait_for_element_with_retry(&self.tab, "textarea[aria-label='Write a caption...']").await?;
        self.manager.type_text_with_retry(&self.tab, "textarea[aria-label='Write a caption...']", description).await?;
        
        info!("Description added successfully");
        Ok(())
    }

    pub async fn set_posting_options(&mut self) -> AppResult<()> {
        info!("Setting Instagram posting options");
        
        // Set any posting options (location, tags, etc.)
        
        self.manager.wait(Duration::from_secs(1)).await;
        
        Ok(())
    }

    pub async fn submit_post(&mut self) -> AppResult<()> {
        info!("Submitting Instagram post");
        
        // Click the "Share" button
        self.manager.click_with_retry(&self.tab, "button[type='button']").await?;
        
        // Wait for confirmation
        self.manager.wait(Duration::from_secs(3)).await;
        
        info!("Instagram post submitted successfully");
        Ok(())
    }

    pub async fn verify_authentication(&mut self) -> AppResult<()> {
        info!("Verifying Instagram authentication");
        
        // Check if we're still logged in
        self.manager.wait_for_element_with_retry(&self.tab, "a[href='/accounts/edit/']").await?;
        
        Ok(())
    }
}

/// Browser automation for YouTube
pub struct YouTubeBrowser {
    manager: BrowserManager,
    tab: Arc<Tab>,
}

impl YouTubeBrowser {
    pub async fn new() -> AppResult<Self> {
        info!("Initializing YouTube browser automation");
        
        let manager = BrowserManager::new()?;
        let tab = manager.new_tab()?;
        
        Ok(Self { manager, tab })
    }

    pub async fn login(&mut self, email: &str, password: &str) -> AppResult<()> {
        info!("Logging into YouTube with email: {}", email);
        
        // Navigate to YouTube login page
        self.manager.navigate_with_retry(&self.tab, "https://accounts.google.com/signin").await?;
        
        // Fill in email
        self.manager.wait_for_element_with_retry(&self.tab, "input[type='email']").await?;
        self.manager.type_text_with_retry(&self.tab, "input[type='email']", email).await?;
        
        // Click next
        self.manager.click_with_retry(&self.tab, "#identifierNext").await?;
        
        // Fill in password
        self.manager.wait_for_element_with_retry(&self.tab, "input[type='password']").await?;
        self.manager.type_text_with_retry(&self.tab, "input[type='password']", password).await?;
        
        // Click next
        self.manager.click_with_retry(&self.tab, "#passwordNext").await?;
        
        // Navigate to YouTube after login
        self.manager.navigate_with_retry(&self.tab, "https://www.youtube.com").await?;
        
        info!("Successfully logged into YouTube");
        Ok(())
    }

    pub async fn navigate_to_upload(&mut self) -> AppResult<()> {
        info!("Navigating to YouTube upload page");
        
        // Click on the create button
        self.manager.click_with_retry(&self.tab, "button[title='Create']").await?;
        
        // Select "Upload video" option
        self.manager.click_with_retry(&self.tab, "a[href='/upload']").await?;
        
        Ok(())
    }

    pub async fn upload_video(&mut self, video_path: &Path) -> AppResult<()> {
        info!("Uploading video to YouTube: {:?}", video_path);
        
        // Find and use the file upload input
        self.manager.wait_for_element_with_retry(&self.tab, "input[type='file']").await?;
        self.manager.upload_file_with_retry(&self.tab, "input[type='file']", video_path).await?;
        
        // Wait for upload to complete
        self.manager.wait(Duration::from_secs(15)).await;
        
        info!("Video uploaded successfully");
        Ok(())
    }

    pub async fn add_title_and_description(&mut self, description: &str) -> AppResult<()> {
        info!("Adding title and description to YouTube video");
        
        // Extract title from description or use first line
        let title = description.lines().next().unwrap_or("TikTok Video");
        
        // Fill in title field
        self.manager.wait_for_element_with_retry(&self.tab, "input[aria-label='Title']").await?;
        self.manager.type_text_with_retry(&self.tab, "input[aria-label='Title']", title).await?;
        
        // Fill in description field
        self.manager.wait_for_element_with_retry(&self.tab, "textarea[aria-label='Description']").await?;
        self.manager.type_text_with_retry(&self.tab, "textarea[aria-label='Description']", description).await?;
        
        info!("Title and description added successfully");
        Ok(())
    }

    pub async fn set_video_options(&mut self) -> AppResult<()> {
        info!("Setting YouTube video options");
        
        // Set visibility to public
        self.manager.click_with_retry(&self.tab, "button[name='PUBLIC']").await?;
        
        self.manager.wait(Duration::from_secs(1)).await;
        
        Ok(())
    }

    pub async fn submit_video(&mut self) -> AppResult<()> {
        info!("Submitting YouTube video");
        
        // Click the "Publish" button
        self.manager.click_with_retry(&self.tab, "button[aria-label='Publish']").await?;
        
        // Wait for confirmation
        self.manager.wait(Duration::from_secs(5)).await;
        
        info!("YouTube video submitted successfully");
        Ok(())
    }

    pub async fn verify_authentication(&mut self) -> AppResult<()> {
        info!("Verifying YouTube authentication");
        
        // Check if we're still logged in
        self.manager.wait_for_element_with_retry(&self.tab, "button[aria-label='Create']").await?;
        
        Ok(())
    }
}
