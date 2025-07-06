# Browser Automation Setup Guide

This application now uses browser automation for posting to social media platforms, providing maximum reliability and flexibility for one-user operation.

## Overview

The system uses headless Chrome browser automation to:
- Login to Facebook, Instagram, and YouTube automatically
- Navigate to posting interfaces
- Upload videos and add descriptions
- Handle errors and retries gracefully

## Configuration

### Browser Settings

Add the following section to your `config.toml`:

```toml
[browser]
# Run browser in headless mode (recommended for production)
headless = true

# Timeout for page loads in seconds
page_load_timeout = 30

# Timeout for element waits in seconds
element_wait_timeout = 10

# Number of retry attempts for failed operations
max_retries = 3

# Delay between retry attempts in milliseconds
retry_delay_ms = 2000

# Browser window size
window_width = 1920
window_height = 1080

# Optional: Custom user agent string
# user_agent = "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"
```

### Platform Credentials

Update your platform configurations to include login credentials:

```toml
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
```

## Environment Variables

You can also configure using environment variables:

```bash
export APP__BROWSER__HEADLESS=true
export APP__BROWSER__PAGE_LOAD_TIMEOUT=30
export APP__BROWSER__ELEMENT_WAIT_TIMEOUT=10
export APP__BROWSER__MAX_RETRIES=3
export APP__BROWSER__RETRY_DELAY_MS=2000
export APP__BROWSER__WINDOW_WIDTH=1920
export APP__BROWSER__WINDOW_HEIGHT=1080

export APP__FACEBOOK__EMAIL=your_facebook_email@example.com
export APP__FACEBOOK__PASSWORD=your_facebook_password
export APP__INSTAGRAM__EMAIL=your_instagram_email@example.com
export APP__INSTAGRAM__PASSWORD=your_instagram_password
export APP__YOUTUBE__EMAIL=your_youtube_email@example.com
export APP__YOUTUBE__PASSWORD=your_youtube_password
```

## Prerequisites

### Chrome/Chromium Installation

The application requires Chrome or Chromium to be installed:

**Ubuntu/Debian:**
```bash
# Install Chrome
wget -q -O - https://dl.google.com/linux/linux_signing_key.pub | sudo apt-key add -
echo "deb [arch=amd64] http://dl.google.com/linux/chrome/deb/ stable main" | sudo tee /etc/apt/sources.list.d/google-chrome.list
sudo apt update
sudo apt install google-chrome-stable

# Or install Chromium
sudo apt install chromium-browser
```

**macOS:**
```bash
# Install Chrome
brew install --cask google-chrome

# Or install Chromium
brew install --cask chromium
```

**Windows:**
Download and install Chrome from https://www.google.com/chrome/

## Security Considerations

### Password Security

- Store passwords securely using environment variables
- Consider using application-specific passwords where available
- Enable two-factor authentication on your accounts
- Monitor login notifications for any suspicious activity

### Network Security

- Run the application on trusted networks
- Consider using VPN if posting from different locations
- Monitor for unusual activity patterns

## Troubleshooting

### Common Issues

1. **Browser not found**
   - Install Chrome or Chromium
   - Check PATH environment variable

2. **Login failures**
   - Verify email/password credentials
   - Check for two-factor authentication requirements
   - Use application-specific passwords if available

3. **Element not found errors**
   - Social media sites may update their layouts
   - Increase `element_wait_timeout` for slower connections
   - Check logs for specific selectors that failed

4. **Upload failures**
   - Verify video file format compatibility
   - Check file size limits for each platform
   - Ensure sufficient disk space for temporary files

### Debug Mode

For troubleshooting, set headless mode to false:

```toml
[browser]
headless = false
```

This will show the browser window so you can see what's happening.

### Logging

Enable debug logging:

```bash
export RUST_LOG=debug
cargo run
```

## Platform-Specific Notes

### Facebook
- Posts to your personal profile or pages you manage
- Supports video files up to 10GB
- Auto-detects video format and duration

### Instagram
- Posts to your Instagram account
- Supports video files up to 100MB
- Automatically handles video processing

### YouTube
- Uploads to your YouTube channel
- Supports large video files
- Sets videos to public visibility by default

## Performance Optimization

- Use headless mode for better performance
- Adjust retry delays based on your network speed
- Monitor memory usage for long-running sessions
- Consider running during off-peak hours for better reliability

## Backup Strategy

- Keep API credentials as fallback option
- Test posting manually periodically
- Monitor success rates and adjust retry settings
- Have alternative posting methods ready