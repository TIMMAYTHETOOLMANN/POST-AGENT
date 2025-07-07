# Deployment Guide for TikTok Cross-posting Agent

This document provides a comprehensive guide for deploying the TikTok Cross-posting Agent, including steps for single-click deployment and post-deployment configuration.

## Prerequisites

Before deploying the application, ensure you have the following prerequisites installed:

- **Rust 2021 edition or later** - Install from [https://rustup.rs/](https://rustup.rs/)
- **Chrome or Chromium browser** - Required for browser automation
  - Windows: Download from [https://www.google.com/chrome/](https://www.google.com/chrome/)
  - Linux: `sudo apt install google-chrome-stable` or `sudo apt install chromium-browser`
  - macOS: `brew install --cask google-chrome` or `brew install --cask chromium`

## Single-Click Deployment

The application provides launcher scripts for both Windows and Linux/macOS that handle the complete setup process in a single command.

### Windows

```cmd
.\launch.bat setup
```

### Linux/macOS

```bash
chmod +x launch.sh  # Make the script executable (first time only)
./launch.sh setup
```

This single command will:
1. Check for required dependencies (Rust/Cargo)
2. Install/update project dependencies
3. Build the project
4. Run the interactive configuration wizard
5. Validate the configuration

## Manual Deployment

If you prefer to deploy the application manually, follow these steps:

1. **Clone the repository**:
   ```bash
   git clone https://github.com/yourusername/tiktok-crosspost.git
   cd tiktok-crosspost
   ```

2. **Install dependencies**:
   ```bash
   cargo update
   ```

3. **Build the project**:
   ```bash
   cargo build --release
   ```

4. **Run configuration wizard**:
   ```bash
   cargo run --bin config_wizard
   ```

5. **Validate configuration**:
   ```bash
   cargo run -- --help
   ```

## Post-Deployment Configuration

After deploying the application, you need to configure the environment variables required for the application to function properly.

### Using the Configuration Wizard

The easiest way to configure the application is to use the built-in configuration wizard:

```bash
# Windows
.\launch.bat config -e

# Linux/macOS
./launch.sh config -e
```

The `-e` flag tells the wizard to generate a `.env` file with all the necessary environment variables.

### Manual Environment Variable Configuration

If you prefer to configure the environment variables manually, refer to the [ENV_VARIABLES.md](ENV_VARIABLES.md) file for a complete list of required environment variables.

## Running the Application

### Foreground Mode

To run the application in foreground mode:

```bash
# Windows
.\launch.bat run

# Linux/macOS
./launch.sh run
```

### Background Mode

To run the application in background mode:

```bash
# Windows
.\launch.bat start

# Linux/macOS
./launch.sh start
```

To check the status of the background process:

```bash
# Windows
.\launch.bat status

# Linux/macOS
./launch.sh status
```

To view logs:

```bash
# Windows
.\launch.bat logs

# Linux/macOS
./launch.sh logs
```

To stop the background process:

```bash
# Windows
.\launch.bat stop

# Linux/macOS
./launch.sh stop
```

## Troubleshooting

If you encounter issues during deployment or running the application:

1. **Build Issues**: Make sure you have Rust installed from [https://rustup.rs/](https://rustup.rs/)
2. **Configuration Errors**: Run `.\launch.bat validate` or `./launch.sh validate` to check your settings
3. **API Issues**: Verify your API keys and tokens are correct
4. **Permission Issues**: On Linux/macOS, ensure the script is executable: `chmod +x launch.sh`
5. **Browser Automation Issues**: Ensure Chrome or Chromium is installed and accessible

For more detailed troubleshooting information, refer to the [BROWSER_AUTOMATION.md](BROWSER_AUTOMATION.md) file.

## Security Considerations

- Store sensitive information like passwords and API keys securely
- Consider using application-specific passwords where available
- Enable two-factor authentication on your accounts
- Monitor login notifications for any suspicious activity

## Updating the Application

To update the application to the latest version:

```bash
# Pull the latest changes
git pull

# Rebuild the application
.\launch.bat build  # Windows
./launch.sh build   # Linux/macOS
```

## Conclusion

By following this guide, you should have successfully deployed the TikTok Cross-posting Agent and configured it for your environment. If you encounter any issues or have questions, please refer to the documentation or contact support.
