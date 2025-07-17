# Universal Deployment Script

This document describes the Universal Deployment Script (`deploy.sh`) that implements the **COPILOT COATING AGENT - UNIFIED DEPLOYMENT DIRECTIVE**.

## Overview

The Universal Deployment Script is a comprehensive, single-click deployment protocol designed to work with any repository type. It provides a standardized deployment workflow with GUI monitoring, environment variable validation, and automatic rollback capabilities.

## Features

- **Multi-language Support**: Automatically detects and handles Node.js, Python, Docker, and Rust projects
- **Environment Variable Validation**: Compares against schema files and prompts for missing variables
- **GUI Integration**: Uses zenity, yad, dialog, or whiptail for user interactions
- **Multiple Deployment Methods**: Supports docker-compose, GitLab CI, and custom deployment scripts
- **Real-time Monitoring**: Provides status updates throughout the deployment process
- **Service Validation**: Automatically tests deployed services
- **Rollback Capability**: Creates Git snapshots and allows one-click rollback
- **Agent Integration**: Outputs JSON status for automation tools
- **Logging**: Comprehensive logging with timestamped entries

## Usage

### Basic Commands

```bash
# Run full deployment (default)
./deploy.sh deploy

# Show deployment status
./deploy.sh status

# Rollback to previous version
./deploy.sh rollback

# Show help
./deploy.sh help
```

### First-time Setup

1. Make the script executable:
   ```bash
   chmod +x deploy.sh
   ```

2. Ensure you have the required dependencies:
   - Git (required)
   - Docker (if using Docker projects)
   - Node.js and npm (for Node.js projects)
   - Python 3 (for Python projects)
   - Rust and Cargo (for Rust projects)

3. Optional GUI tools for better user experience:
   - `zenity` or `yad` for modern GUI dialogs
   - `dialog` or `whiptail` for terminal-based dialogs

## Deployment Process

The script follows a 10-step protocol:

### Step 1: Project Initialization
- Identifies the repository root
- Extracts project name from folder name
- Creates timestamped deployment log

### Step 2: Project Type Detection
- **Node.js**: Detects `package.json`, runs `npm install` and `npm run build`
- **Python**: Detects `requirements.txt`, creates virtual environment, installs dependencies
- **Docker**: Detects `Dockerfile`, builds Docker image
- **Rust**: Detects `Cargo.toml`, runs `cargo build --release`

### Step 3: Environment Variable Validation
- Loads existing `.env` file
- Compares against `.env.schema` or `.env.example`
- Prompts for missing variables with appropriate input methods:
  - Password prompts for sensitive variables (PASSWORD, SECRET, KEY, TOKEN)
  - Regular prompts for other variables
- Scans code for environment variable usage if no schema exists

### Step 4: Deployment Execution
- **Docker Compose**: Runs `docker-compose up -d --build`
- **GitLab CI**: Commits changes and pushes to trigger pipeline
- **Custom Script**: Executes `deploy.sh` if present
- **Generic**: Attempts common deployment commands

### Step 5: Real-time GUI Monitoring
- Launches background GUI window with status updates
- Shows progress: "Build Started" → "Secrets Loaded" → "Deploying" → "Complete"
- Auto-closes after successful completion

### Step 6: Service Validation
- Detects ports from docker-compose configuration
- Tests service connectivity on discovered ports
- Falls back to common ports (3000, 8000, 8080, 80, 443, 5000, 9000)
- Reports service health status

### Step 7: Snapshot and Rollback
- Creates Git snapshot before deployment
- Tags snapshot with timestamp
- Stores rollback commit reference
- Provides one-click rollback capability

### Step 8: Post-deploy GUI Confirmation
- Shows success message via GUI or terminal
- Offers option to archive deployment
- Creates compressed archive in `deploy-history/` folder

### Step 9: Terminal UI Fallback
- Automatically detects available UI tools
- Falls back to terminal prompts if no GUI available
- Supports `zenity`, `yad`, `dialog`, `whiptail`

### Step 10: Agent-friendly Output
- Generates JSON status payload
- Includes deployment metadata
- Saves to `deployment_status.json`

## Configuration

### Environment Variables Schema

Create a `.env.example` or `.env.schema` file in your project root:

```bash
# Example .env.schema
NODE_ENV=production
PORT=3000
DATABASE_URL=postgresql://localhost:5432/myapp
API_KEY=your_api_key_here
SECRET_KEY=your_secret_key_here
```

### Custom Deployment Scripts

Create a `deploy.sh` file in your project root for custom deployment logic:

```bash
#!/bin/bash
# Custom deployment script

echo "Starting custom deployment..."

# Your deployment logic here
docker-compose up -d
nginx -s reload

echo "Custom deployment completed!"
```

## File Structure

After deployment, the following files may be created:

```
project-root/
├── deploy.sh                    # Universal deployment script
├── .env                        # Environment variables (auto-generated)
├── .env.example               # Environment variables schema
├── deployment_YYYYMMDD_HHMMSS.log  # Deployment log
├── deployment_status.json     # JSON status output
├── .project_type             # Detected project type
├── .deployment_method        # Used deployment method
├── .rollback_commit          # Rollback reference
└── deploy-history/           # Deployment archives
    └── deployment-YYYYMMDD_HHMMSS.tar.gz
```

## GUI Tools Installation

### Ubuntu/Debian
```bash
sudo apt-get install zenity dialog whiptail
```

### CentOS/RHEL
```bash
sudo yum install zenity dialog newt
```

### macOS
```bash
brew install zenity dialog
```

## Examples

### Node.js Project
```bash
# Project with package.json
./deploy.sh deploy
# → Detects Node.js, runs npm install, npm run build
# → Prompts for environment variables from .env.example
# → Deploys using docker-compose or custom script
```

### Python Project
```bash
# Project with requirements.txt
./deploy.sh deploy
# → Creates virtual environment
# → Installs dependencies
# → Validates environment variables
# → Deploys application
```

### Docker Project
```bash
# Project with Dockerfile
./deploy.sh deploy
# → Builds Docker image
# → Validates configuration
# → Runs docker-compose up -d --build
```

## Troubleshooting

### Common Issues

1. **Permission Denied**
   ```bash
   chmod +x deploy.sh
   ```

2. **Git Not Found**
   ```bash
   sudo apt-get install git  # Ubuntu/Debian
   sudo yum install git      # CentOS/RHEL
   ```

3. **No GUI Available**
   - The script automatically falls back to terminal prompts
   - Install zenity or dialog for better user experience

4. **Deployment Failed**
   - Check the deployment log file
   - Use `./deploy.sh rollback` to revert changes
   - Verify environment variables are correct

### Debugging

1. **Check Deployment Status**
   ```bash
   ./deploy.sh status
   ```

2. **View Deployment Log**
   ```bash
   cat deployment_*.log
   ```

3. **Rollback on Failure**
   ```bash
   ./deploy.sh rollback
   ```

## Integration with Existing Projects

The Universal Deployment Script is designed to work alongside existing deployment infrastructure:

- **Compatible with existing `launch.sh`**: Use alongside project-specific launchers
- **Respects existing configuration**: Loads and validates current `.env` files
- **Non-destructive**: Creates backups and supports rollback
- **Extensible**: Supports custom deployment scripts

## Security Considerations

- Sensitive variables are prompted with password input
- Environment files are excluded from version control
- Deployment logs may contain sensitive information
- Always review generated `.env` files before committing

## License

This script is part of the POST-AGENT project and follows the same license terms.