# TikTok Cross-posting Application

This application allows you to automatically cross-post your TikTok content to other social media platforms like
Facebook, Instagram, and YouTube.

## Features

- Automatically fetch new TikTok videos
- Cross-post to multiple platforms
- Schedule posts
- Customize content formatting
- Interactive configuration wizard

## Configuration

The application can be configured in several ways:

1. Using the interactive configuration wizard
2. Editing the config.toml file directly
3. Setting environment variables

### Using the Configuration Wizard

The configuration wizard provides an interactive way to set up all the necessary configuration values. To run the
wizard:

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
api_key = "your_facebook_api_key"
api_secret = "your_facebook_api_secret"
access_token = "your_facebook_access_token"

[instagram]
account_id = "your_instagram_account_id"
api_key = "your_instagram_api_key"
api_secret = "your_instagram_api_secret"
access_token = "your_instagram_access_token"

[youtube]
channel_id = "your_youtube_channel_id"
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
```

### Environment Variables

You can also configure the application using environment variables. The environment variables should be prefixed with
`APP__` and use double underscores (`__`) as separators. For example:

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

## Development

### Prerequisites

- Rust 2024 or later
- Cargo

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.