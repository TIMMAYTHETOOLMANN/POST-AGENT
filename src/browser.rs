use crate::config;
use crate::error::{AppError, AppResult};
use headless_chrome::protocol::cdp::Page;
use headless_chrome::{Browser, LaunchOptionsBuilder, Tab};
use log::{debug, error, info, warn};
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
            .map_err(|e| AppError::GenericError(format!("Failed to build launch options: {}", e)))?;

        let browser = Browser::new(launch_options)
            .map_err(|e| AppError::GenericError(format!("Failed to launch browser: {}", e)))?;

        Ok(Self { browser, config })
    }

    /// Create a new tab
    pub fn new_tab(&self) -> AppResult<Arc<Tab>> {
        let tab = self.browser.new_tab()
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
                        }
                        Err(e) => {
                            warn!("Page load timeout on attempt {}: {}", attempt, e);
                            if attempt == self.config.max_retries {
                                return Err(AppError::GenericError(format!(
                                    "Failed to load page after {} attempts: {}",
                                    self.config.max_retries, e
                                )));
                            }
                        }
                    }
                }
                Err(e) => {
                    warn!("Navigation failed on attempt {}: {}", attempt, e);
                    if attempt == self.config.max_retries {
                        return Err(AppError::GenericError(format!(
                            "Failed to navigate after {} attempts: {}",
                            self.config.max_retries, e
                        )));
                    }
                }
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
                }
                Err(e) => {
                    warn!("Element not found on attempt {}: {}", attempt, e);
                    if attempt == self.config.max_retries {
                        return Err(AppError::GenericError(format!(
                            "Element '{}' not found after {} attempts",
                            selector, self.config.max_retries
                        )));
                    }
                }
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
                            match element.call_js_fn("function() { this.value = ''; }", vec![], false) {
                                Ok(_) => {
                                    match element.type_into(text) {
                                        Ok(_) => {
                                            debug!("Successfully typed text into element: {}", selector);
                                            return Ok(());
                                        }
                                        Err(e) => {
                                            warn!("Failed to type text on attempt {}: {}", attempt, e);
                                        }
                                    }
                                }
                                Err(e) => {
                                    warn!("Failed to clear element on attempt {}: {}", attempt, e);
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to focus element on attempt {}: {}", attempt, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Element not found on attempt {}: {}", attempt, e);
                }
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
                Ok(element) => {
                    match element.click() {
                        Ok(_) => {
                            debug!("Successfully clicked element: {}", selector);
                            return Ok(());
                        }
                        Err(e) => {
                            warn!("Failed to click element on attempt {}: {}", attempt, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Element not found on attempt {}: {}", attempt, e);
                }
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
                Ok(element) => {
                    match element.set_input_files(&[&file_path_str]) {
                        Ok(_) => {
                            debug!("Successfully uploaded file to element: {}", selector);
                            return Ok(());
                        }
                        Err(e) => {
                            warn!("Failed to upload file on attempt {}: {}", attempt, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Element not found on attempt {}: {}", attempt, e);
                }
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
        let screenshot = tab.capture_screenshot(
            Page::CaptureScreenshotFormatOption::Png,
            None,
            None,
            true,
        )
        .map_err(|e| AppError::GenericError(format!("Failed to take screenshot: {}", e)))?;
        
        std::fs::write(path, screenshot)
            .map_err(|e| AppError::GenericError(format!("Failed to save screenshot: {}", e)))?;
        
        debug!("Screenshot saved to {}", path);
        Ok(())
    }
}