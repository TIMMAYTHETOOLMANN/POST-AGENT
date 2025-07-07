# Environment Variables for TikTok Cross-posting Agent

This document outlines all the environment variables that need to be configured after deployment of the TikTok Cross-posting Agent.

## Overview

The application can be configured using environment variables with the prefix `APP__` and double underscores (`__`) as separators. These environment variables can be set in your system environment or in a `.env` file.

## Required Environment Variables

### TikTok Configuration
```
APP__TIKTOK__USERNAME=your_tiktok_username
APP__TIKTOK__API_KEY=your_tiktok_api_key
APP__TIKTOK__API_SECRET=your_tiktok_api_secret
APP__TIKTOK__ACCESS_TOKEN=your_tiktok_access_token  # Optional
```

### Facebook Configuration
```
APP__FACEBOOK__PAGE_ID=your_facebook_page_id
APP__FACEBOOK__EMAIL=your_facebook_email@example.com
APP__FACEBOOK__PASSWORD=your_facebook_password
APP__FACEBOOK__API_KEY=your_facebook_api_key  # Optional, for hybrid approach
APP__FACEBOOK__API_SECRET=your_facebook_api_secret  # Optional, for hybrid approach
APP__FACEBOOK__ACCESS_TOKEN=your_facebook_access_token  # Optional, for hybrid approach
```

### Instagram Configuration
```
APP__INSTAGRAM__ACCOUNT_ID=your_instagram_account_id
APP__INSTAGRAM__EMAIL=your_instagram_email@example.com
APP__INSTAGRAM__PASSWORD=your_instagram_password
APP__INSTAGRAM__API_KEY=your_instagram_api_key  # Optional, for hybrid approach
APP__INSTAGRAM__API_SECRET=your_instagram_api_secret  # Optional, for hybrid approach
APP__INSTAGRAM__ACCESS_TOKEN=your_instagram_access_token  # Optional, for hybrid approach
```

### YouTube Configuration
```
APP__YOUTUBE__CHANNEL_ID=your_youtube_channel_id
APP__YOUTUBE__EMAIL=your_youtube_email@example.com
APP__YOUTUBE__PASSWORD=your_youtube_password
APP__YOUTUBE__API_KEY=your_youtube_api_key  # Optional, for hybrid approach
APP__YOUTUBE__CLIENT_ID=your_youtube_client_id  # Optional, for hybrid approach
APP__YOUTUBE__CLIENT_SECRET=your_youtube_client_secret  # Optional, for hybrid approach
APP__YOUTUBE__REFRESH_TOKEN=your_youtube_refresh_token  # Optional, for hybrid approach
```

### Content Configuration
```
APP__CONTENT__HASHTAG_COUNT=5
APP__CONTENT__CUSTOM_HASHTAGS=fyp,viral  # Comma-separated list
APP__CONTENT__TAGLINE_TEMPLATES="You won't believe what happens next! 😱","This changed everything for me! 🔥","Wait for it... 👀"  # Comma-separated list
APP__CONTENT__USE_TRENDING_HASHTAGS=true
```

### Scheduler Configuration
```
APP__SCHEDULER__CHECK_INTERVAL_MINUTES=5
APP__SCHEDULER__LEGACY_POST_INTERVAL_MINUTES=10
APP__SCHEDULER__MAX_POSTS_PER_DAY=24
```

### Browser Configuration
```
APP__BROWSER__HEADLESS=true
APP__BROWSER__PAGE_LOAD_TIMEOUT=30
APP__BROWSER__ELEMENT_WAIT_TIMEOUT=10
APP__BROWSER__MAX_RETRIES=3
APP__BROWSER__RETRY_DELAY_MS=2000
APP__BROWSER__WINDOW_WIDTH=1920
APP__BROWSER__WINDOW_HEIGHT=1080
APP__BROWSER__USER_AGENT="Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36"  # Optional
```

## Generating Environment Variables

You can generate a `.env` file with all the necessary environment variables using the configuration wizard:

```bash
# Linux/macOS
./launch.sh config -e

# Windows
.\launch.bat config -e
```

## Security Considerations

- Store sensitive information like passwords and API keys securely
- Consider using application-specific passwords where available
- Enable two-factor authentication on your accounts
- Monitor login notifications for any suspicious activity

## Troubleshooting

If you encounter issues with environment variables:

1. Verify that the variables are correctly formatted with the `APP__` prefix and double underscores (`__`) as separators
2. Check for typos in variable names
3. Ensure that the values are properly escaped if they contain special characters
4. Run the validation command to check your configuration:
   ```bash
   # Linux/macOS
   ./launch.sh validate
   
   # Windows
   .\launch.bat validate
   ```