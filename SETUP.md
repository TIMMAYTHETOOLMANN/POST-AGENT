# POST-AGENT Setup and Usage Guide

## Quick Start

The easiest way to set up and run the POST-AGENT is using the launcher script:

### Linux/macOS
```bash
./launch.sh setup
```

### Windows
```cmd
launch.bat setup
```

This will:
1. Check dependencies (Rust/Cargo)
2. Install/update project dependencies
3. Build the project
4. Run the interactive configuration wizard
5. Validate the configuration

## Manual Setup

If you prefer to run commands manually:

1. **Install dependencies:**
   ```bash
   cargo update
   ```

2. **Build the project:**
   ```bash
   cargo build --release
   ```

3. **Run configuration wizard:**
   ```bash
   cargo run --bin config_wizard
   # OR using the main binary:
   cargo run -- config
   ```

4. **Launch the agent:**
   ```bash
   cargo run --release
   ```

## Configuration

The application supports multiple configuration methods:

### Interactive CLI Wizard
- Run `./launch.sh config` or `cargo run -- config`
- Provides step-by-step guidance for all settings
- Includes validation and connectivity testing
- Can generate .env files

### Configuration File (config.toml)
The wizard creates a TOML configuration file with all your settings.

### Environment Variables
Use the `APP__` prefix with double underscores:
- `APP__TIKTOK__USERNAME=your_username`
- `APP__TIKTOK__API_KEY=your_key`
- etc.

## Running the Agent

### Foreground Mode
```bash
./launch.sh run
```

### Background Mode
```bash
./launch.sh start    # Start background watcher
./launch.sh status   # Check status
./launch.sh logs     # View logs
./launch.sh stop     # Stop background watcher
```

## Launcher Commands

The launcher script (`launch.sh` / `launch.bat`) supports these commands:

- `setup` - Complete first-time setup
- `build` - Build the project
- `deps` - Install/update dependencies
- `config` - Run configuration wizard
- `validate` - Validate current configuration
- `run` - Launch main agent (foreground)
- `start` - Start background watcher
- `stop` - Stop background watcher
- `status` - Show current status
- `logs` - Show recent log entries
- `help` - Show help message

## Features

- **Automated TikTok Content Fetching**: Monitors your TikTok account for new videos
- **Multi-platform Posting**: Cross-posts to Facebook, Instagram, and YouTube
- **Scheduling**: Configurable intervals for checking and posting
- **Content Processing**: Adds hashtags, customizable taglines
- **Background Operation**: Can run as a daemon
- **Configuration Validation**: Ensures all settings are correct before running
- **Logging**: Comprehensive logging for monitoring and debugging

## Troubleshooting

1. **Build Issues**: Make sure you have Rust installed from https://rustup.rs/
2. **Configuration Errors**: Run `./launch.sh validate` to check your settings
3. **API Issues**: Verify your API keys and tokens are correct
4. **Permission Issues**: On Linux/macOS, ensure the script is executable: `chmod +x launch.sh`

## Requirements

- Rust 2024 edition or later
- API keys for:
  - TikTok
  - Facebook (optional)
  - Instagram (optional)  
  - YouTube (optional)

## Support

For issues and feature requests, please check the project repository.