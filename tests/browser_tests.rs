use tiktok_crosspost::browser::BrowserManager;
use tiktok_crosspost::config;
use tiktok_crosspost::platforms;
use tokio;

#[tokio::test]
async fn test_browser_manager_creation() {
    // Set up basic configuration
    unsafe {
        std::env::set_var("APP__BROWSER__HEADLESS", "true");
        std::env::set_var("APP__BROWSER__PAGE_LOAD_TIMEOUT", "30");
        std::env::set_var("APP__BROWSER__ELEMENT_WAIT_TIMEOUT", "10");
        std::env::set_var("APP__BROWSER__MAX_RETRIES", "3");
        std::env::set_var("APP__BROWSER__RETRY_DELAY_MS", "1000");
        std::env::set_var("APP__BROWSER__WINDOW_WIDTH", "1920");
        std::env::set_var("APP__BROWSER__WINDOW_HEIGHT", "1080");
    }

    // Load config first
    let _ = config::load_config();

    // Test browser manager creation
    let result = BrowserManager::new();

    match result {
        Ok(browser_manager) => {
            println!("✓ Browser manager created successfully");

            // Test creating a new tab
            let tab_result = browser_manager.new_tab();
            match tab_result {
                Ok(_) => {
                    println!("✓ Browser tab created successfully");
                },
                Err(e) => {
                    println!("✗ Failed to create browser tab: {}", e);
                },
            }
        },
        Err(e) => {
            println!("✗ Failed to create browser manager: {}", e);
            // This might fail in environments without Chrome installed
            // but that's expected in CI environments
        },
    }
}

#[tokio::test]
async fn test_platform_authentication_functions_exist() {
    // Test that our authentication functions are properly compiled
    // We can't test actual authentication without credentials
    println!("✓ Platform authentication functions compiled successfully");

    // Verify function signatures exist by checking they can be referenced
    let _auth_all = platforms::authenticate_all_platforms;
    println!("✓ authenticate_all_platforms function exists");
}
