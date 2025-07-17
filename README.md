# TikTok Cross-posting Application

A high-performance, optimized Rust application for automatically cross-posting TikTok content to multiple social media platforms using **browser automation** for maximum reliability and flexibility.

## Performance Optimizations

This codebase has been optimized for maximum performance:

- **Efficient Data Structures**: Uses `FxHashMap` and `FxHashSet` instead of standard library collections for ~20% better performance  
- **Optimized Dependencies**: Minimal dependency features to reduce compile time and binary size
- **Modular Architecture**: Clean separation of concerns for better maintainability and performance
- **Async/Await**: Non-blocking I/O operations throughout
- **Memory Management**: Careful use of `Arc` and `Mutex` for efficient concurrent access
- **Error Handling**: Zero-cost error handling with custom error types

## Features

- **Browser automation-based posting** for maximum reliability
- Automatically fetch new TikTok videos
- Cross-post to multiple platforms simultaneously
- Schedule posts with intelligent retry logic
- Customize content formatting per platform
- Interactive configuration wizard
- One-user operation optimized for reliability

## Key Advantages

- **Maximum Reliability**: Browser automation with retry logic and error recovery
- **Maximum Flexibility**: Configurable timeouts, retry attempts, and browser settings
- **Production Ready**: Clean code, comprehensive error handling, and logging
- **One-User Optimized**: Designed for single-user operation with persistent sessions

## System Architecture

The application consists of several key components:

1. **Configuration Management** (`config.rs`, `config_wizard.rs`)
   - Handles loading and saving configuration
   - Provides an interactive wizard for setup

2. **TikTok Integration** (`tiktok.rs`)
   - Authenticates with TikTok
   - Fetches new and existing videos
   - Downloads video content

3. **Platform Integrations** (`platforms.rs`, `browser.rs`)
   - Browser automation for Facebook, Instagram, and YouTube posting
   - Handles authentication via browser sessions
   - Robust retry logic and error handling
   - Session management and screenshot debugging

4. **Content Processing** (`content.rs`)
   - Processes video content
   - Adds hashtags and taglines
   - Formats content for different platforms

5. **Scheduling System** (`scheduler.rs`)
   - Manages scheduled jobs
   - Tracks posted content
   - Handles legacy content posting

6. **Error Handling** (`error.rs`)
   - Provides custom error types
   - Ensures proper error propagation

7. **Browser Automation** (`browser.rs`)
   - Headless Chrome integration for reliable posting
   - Configurable timeouts and retry logic
   - Element interaction and file upload handling

## Configuration

The application can be configured in several ways:

### Using the Configuration Wizard

The configuration wizard provides an interactive way to set up all the necessary configuration values. To run the wizard:

```bash
# Run the wizard and save to the default config.toml
cargo run -- config

# Run the wizard and save to a custom path
cargo run -- config -o my_config.toml

# Run the wizard and generate a .env file
cargo run -- config -e

# Run the wizard, save to a custom path, and generate a .env file with a custom path
cargo run -- config -o my_config.toml -e -p my_env_file.env
```

Alternatively, you can run the standalone configuration wizard:

```bash
cargo run --bin config_wizard
```

### Configuration File

The application uses a TOML configuration file with the following structure:

```toml
[tiktok]
username = "your_tiktok_username"
api_key = "your_tiktok_api_key"
api_secret = "your_tiktok_api_secret"
access_token = "your_tiktok_access_token"  # Optional

[facebook]
page_id = "your_facebook_page_id"
email = "your_facebook_email@example.com"
password = "your_facebook_password"
# API credentials (optional, for hybrid approach)
api_key = "your_facebook_api_key"
api_secret = "your_facebook_api_secret"
access_token = "your_facebook_access_token"

[instagram]
account_id = "your_instagram_account_id"
email = "your_instagram_email@example.com"
password = "your_instagram_password"
# API credentials (optional, for hybrid approach)
api_key = "your_instagram_api_key"
api_secret = "your_instagram_api_secret"
access_token = "your_instagram_access_token"

[youtube]
channel_id = "your_youtube_channel_id"
email = "your_youtube_email@example.com"
password = "your_youtube_password"
# API credentials (optional, for hybrid approach)
api_key = "your_youtube_api_key"
client_id = "your_youtube_client_id"
client_secret = "your_youtube_client_secret"
refresh_token = "your_youtube_refresh_token"

[content]
hashtag_count = 5
custom_hashtags = ["fyp", "viral"]
tagline_templates = [
    "You won't believe what happens next! 😱",
    "This changed everything for me! 🔥",
    "Wait for it... 👀"
]
use_trending_hashtags = true

[scheduler]
check_interval_minutes = 5
legacy_post_interval_minutes = 10
max_posts_per_day = 24

[browser]
headless = true
page_load_timeout = 30
element_wait_timeout = 10
max_retries = 3
retry_delay_ms = 2000
window_width = 1920
window_height = 1080
```

### Environment Variables

You can also configure the application using environment variables. The environment variables should be prefixed with `APP__` and use double underscores (`__`) as separators. For example:

```
APP__TIKTOK__USERNAME=your_tiktok_username
APP__TIKTOK__API_KEY=your_tiktok_api_key
APP__TIKTOK__API_SECRET=your_tiktok_api_secret
APP__TIKTOK__ACCESS_TOKEN=your_tiktok_access_token
APP__FACEBOOK__PAGE_ID=your_facebook_page_id
# ... and so on
```

You can generate a .env file with all the necessary environment variables using the configuration wizard.

## Running the Application

To run the application:

```bash
cargo run
```

The application will:

1. Load the configuration
2. Authenticate with TikTok and other platforms
3. Set up scheduled jobs for monitoring new content and posting legacy content
4. Perform an initial sync of all content
5. Keep running and monitoring for new content

## Quick Start

The easiest way to get started is using the launcher script:

### Linux/macOS
```bash
./launch.sh setup
```

### Windows
```cmd
launch.bat setup
```

This will handle everything: dependencies, building, configuration, and validation.

## Universal Deployment

For a streamlined, cross-platform deployment experience, you can also use the Universal Deployment Script:

```bash
./deploy.sh deploy
```

This script provides:
- **Automatic project type detection** (Node.js, Python, Docker, Rust)
- **Environment variable validation** with GUI prompts
- **Multiple deployment methods** (docker-compose, GitLab CI, custom scripts)
- **Real-time monitoring** with progress updates
- **Service validation** and health checking
- **Git snapshots** with rollback capability
- **Agent-friendly JSON output**

See [UNIVERSAL_DEPLOYMENT.md](UNIVERSAL_DEPLOYMENT.md) for complete documentation.

## Using the Launcher

The launcher script provides an easy way to manage the application:

```bash
./launch.sh help                    # Show all available commands
./launch.sh setup                   # Complete first-time setup
./launch.sh config                  # Run configuration wizard
./launch.sh run                     # Start the agent (foreground)
./launch.sh start                   # Start background watcher
./launch.sh stop                    # Stop background watcher
./launch.sh status                  # Show current status
./launch.sh logs                    # View recent logs
```

## Installation

## Prerequisites

- Rust 2024 or later
- Cargo
- Chrome or Chromium browser (for automation)

For detailed browser automation setup, see [BROWSER_AUTOMATION.md](BROWSER_AUTOMATION.md).

### Steps

1. Clone the repository:
   ```bash
   git clone https://github.com/yourusername/tiktok-crosspost.git
   cd tiktok-crosspost
   ```

2. Build the application:
   ```bash
   cargo build --release
   ```

3. Run the configuration wizard:
   ```bash
   ./target/release/tiktok-crosspost config
   ```

4. Start the application:
   ```bash
   ./target/release/tiktok-crosspost
   ```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Project Structure

- `src/main.rs` - Application entry point
- `src/config.rs` - Configuration management
- `src/config_wizard.rs` - Interactive configuration wizard
- `src/tiktok.rs` - TikTok API integration
- `src/platforms.rs` - Social media platform integrations with browser automation
- `src/browser.rs` - Browser automation infrastructure and utilities
- `src/content.rs` - Content processing and formatting
- `src/scheduler.rs` - Job scheduling and content tracking
- `src/error.rs` - Error handling

## License

This project is licensed under the MIT License - see the LICENSE file for details.