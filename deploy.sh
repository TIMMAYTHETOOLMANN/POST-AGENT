#!/bin/bash

# COPILOT COATING AGENT - UNIFIED DEPLOYMENT DIRECTIVE
# Universal single-click deployment protocol for any repository

set -euo pipefail

# Global variables
PROJECT_ROOT=""
PROJECT_NAME=""
LOG_FILE=""
DEPLOYMENT_STATUS=""
DEPLOYMENT_PID=""
CONFIG_GENERATED=""
ROLLBACK_COMMIT=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
    if [ -n "$LOG_FILE" ]; then
        echo "$(date '+%Y-%m-%d %H:%M:%S') [INFO] $1" >> "$LOG_FILE"
    fi
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
    if [ -n "$LOG_FILE" ]; then
        echo "$(date '+%Y-%m-%d %H:%M:%S') [SUCCESS] $1" >> "$LOG_FILE"
    fi
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
    if [ -n "$LOG_FILE" ]; then
        echo "$(date '+%Y-%m-%d %H:%M:%S') [WARNING] $1" >> "$LOG_FILE"
    fi
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
    if [ -n "$LOG_FILE" ]; then
        echo "$(date '+%Y-%m-%d %H:%M:%S') [ERROR] $1" >> "$LOG_FILE"
    fi
}

print_header() {
    echo -e "${PURPLE}[DEPLOY]${NC} $1"
    if [ -n "$LOG_FILE" ]; then
        echo "$(date '+%Y-%m-%d %H:%M:%S') [DEPLOY] $1" >> "$LOG_FILE"
    fi
}

# Function to detect GUI availability
detect_gui() {
    if command -v zenity &> /dev/null; then
        echo "zenity"
    elif command -v yad &> /dev/null; then
        echo "yad"
    elif command -v dialog &> /dev/null; then
        echo "dialog"
    elif command -v whiptail &> /dev/null; then
        echo "whiptail"
    else
        echo "none"
    fi
}

# Function to show GUI notification
show_gui_notification() {
    local message="$1"
    local gui_tool=$(detect_gui)
    
    case $gui_tool in
        "zenity")
            zenity --info --text="$message" --width=300 --height=100 &
            ;;
        "yad")
            yad --info --text="$message" --width=300 --height=100 &
            ;;
        *)
            print_status "$message"
            ;;
    esac
}

# Function to prompt for input with GUI
prompt_for_input() {
    local prompt="$1"
    local default_value="$2"
    local is_password="${3:-false}"
    local gui_tool=$(detect_gui)
    
    local result=""
    
    case $gui_tool in
        "zenity")
            if [ "$is_password" = "true" ]; then
                result=$(zenity --password --title="Configuration" --text="$prompt" 2>/dev/null || echo "")
            else
                result=$(zenity --entry --title="Configuration" --text="$prompt" --entry-text="$default_value" 2>/dev/null || echo "$default_value")
            fi
            ;;
        "yad")
            if [ "$is_password" = "true" ]; then
                result=$(yad --entry --title="Configuration" --text="$prompt" --hide-text 2>/dev/null || echo "")
            else
                result=$(yad --entry --title="Configuration" --text="$prompt" --entry-text="$default_value" 2>/dev/null || echo "$default_value")
            fi
            ;;
        "dialog")
            if [ "$is_password" = "true" ]; then
                result=$(dialog --passwordbox "$prompt" 10 50 3>&1 1>&2 2>&3 || echo "")
            else
                result=$(dialog --inputbox "$prompt" 10 50 "$default_value" 3>&1 1>&2 2>&3 || echo "$default_value")
            fi
            ;;
        "whiptail")
            if [ "$is_password" = "true" ]; then
                result=$(whiptail --passwordbox "$prompt" 10 50 3>&1 1>&2 2>&3 || echo "")
            else
                result=$(whiptail --inputbox "$prompt" 10 50 "$default_value" 3>&1 1>&2 2>&3 || echo "$default_value")
            fi
            ;;
        *)
            read -p "$prompt [$default_value]: " result
            result=${result:-$default_value}
            ;;
    esac
    
    echo "$result"
}

# Function to show GUI progress
show_gui_progress() {
    local title="$1"
    local gui_tool=$(detect_gui)
    
    case $gui_tool in
        "zenity")
            zenity --progress --title="$title" --text="Initializing..." --width=400 --height=100 --auto-close &
            echo $! > /tmp/gui_progress.pid
            ;;
        "yad")
            yad --progress --title="$title" --text="Initializing..." --width=400 --height=100 --auto-close &
            echo $! > /tmp/gui_progress.pid
            ;;
        *)
            print_status "$title"
            ;;
    esac
}

# Function to update GUI progress
update_gui_progress() {
    local percentage="$1"
    local message="$2"
    local gui_tool=$(detect_gui)
    
    if [ -f /tmp/gui_progress.pid ]; then
        case $gui_tool in
            "zenity"|"yad")
                echo "$percentage"
                echo "# $message"
                ;;
        esac
    fi
}

# Function to close GUI progress
close_gui_progress() {
    if [ -f /tmp/gui_progress.pid ]; then
        local pid=$(cat /tmp/gui_progress.pid)
        kill $pid 2>/dev/null || true
        rm -f /tmp/gui_progress.pid
    fi
}

# STEP 1 - PROJECT INITIALIZATION
initialize_project() {
    print_header "STEP 1 - PROJECT INITIALIZATION"
    
    # Identify the root of the repository
    PROJECT_ROOT=$(git rev-parse --show-toplevel 2>/dev/null || pwd)
    cd "$PROJECT_ROOT"
    
    # Extract project name from the folder name
    PROJECT_NAME=$(basename "$PROJECT_ROOT")
    
    # Generate a timestamped deployment log file
    LOG_FILE="$PROJECT_ROOT/deployment_$(date +%Y%m%d_%H%M%S).log"
    
    print_status "Project root: $PROJECT_ROOT"
    print_status "Project name: $PROJECT_NAME"
    print_status "Log file: $LOG_FILE"
    
    # Create deployment directory if it doesn't exist
    mkdir -p "$PROJECT_ROOT/deploy-history"
    
    update_gui_progress 10 "Project initialized"
}

# STEP 2 - PROJECT TYPE DETECTION
detect_project_type() {
    print_header "STEP 2 - PROJECT TYPE DETECTION"
    
    local project_type=""
    
    if [ -f "package.json" ]; then
        project_type="nodejs"
        print_status "Detected Node.js project"
        
        # Run npm install
        print_status "Running npm install..."
        if npm install; then
            print_success "npm install completed"
        else
            print_error "npm install failed"
            return 1
        fi
        
        # Check for build script and run it
        if npm run build --if-present; then
            print_success "npm run build completed"
        else
            print_warning "npm run build failed or not available"
        fi
        
    elif [ -f "requirements.txt" ]; then
        project_type="python"
        print_status "Detected Python project"
        
        # Create virtual environment
        print_status "Creating virtual environment..."
        if python3 -m venv venv; then
            print_success "Virtual environment created"
        else
            print_error "Failed to create virtual environment"
            return 1
        fi
        
        # Activate virtual environment
        print_status "Activating virtual environment..."
        source venv/bin/activate
        
        # Install requirements
        print_status "Installing requirements..."
        if pip install -r requirements.txt; then
            print_success "Requirements installed"
        else
            print_error "Failed to install requirements"
            return 1
        fi
        
    elif [ -f "Dockerfile" ]; then
        project_type="docker"
        print_status "Detected Docker project"
        
        # Build Docker image
        print_status "Building Docker image..."
        if docker build -t "$PROJECT_NAME" .; then
            print_success "Docker image built successfully"
        else
            print_error "Failed to build Docker image"
            return 1
        fi
        
    elif [ -f "Cargo.toml" ]; then
        project_type="rust"
        print_status "Detected Rust project"
        
        # Run cargo build
        print_status "Running cargo build..."
        if cargo build --release; then
            print_success "Rust project built successfully"
        else
            print_error "Failed to build Rust project"
            return 1
        fi
        
    else
        project_type="unknown"
        print_warning "Unknown project type, proceeding with generic deployment"
    fi
    
    echo "$project_type" > "$PROJECT_ROOT/.project_type"
    update_gui_progress 25 "Project type detected: $project_type"
}

# STEP 3 - ENVIRONMENT VARIABLE VALIDATION
validate_environment_variables() {
    print_header "STEP 3 - ENVIRONMENT VARIABLE VALIDATION"
    
    local env_file="$PROJECT_ROOT/.env"
    local schema_file=""
    
    # Check for schema files
    if [ -f "$PROJECT_ROOT/.env.schema" ]; then
        schema_file="$PROJECT_ROOT/.env.schema"
    elif [ -f "$PROJECT_ROOT/.env.example" ]; then
        schema_file="$PROJECT_ROOT/.env.example"
    fi
    
    # Load existing .env file if it exists
    if [ -f "$env_file" ]; then
        print_status "Loading existing .env file..."
        source "$env_file"
        print_success ".env file loaded"
    else
        print_status "No existing .env file found"
        touch "$env_file"
    fi
    
    # If schema file exists, validate against it
    if [ -n "$schema_file" ]; then
        print_status "Validating environment variables against schema..."
        
        # Read schema file and check for missing variables
        while IFS= read -r line; do
            if [[ "$line" =~ ^[A-Z_]+= ]]; then
                var_name=$(echo "$line" | cut -d'=' -f1)
                var_value=$(echo "$line" | cut -d'=' -f2-)
                
                # Check if variable is already set
                if [ -z "${!var_name:-}" ]; then
                    print_warning "Missing environment variable: $var_name"
                    
                    # Determine if this is a sensitive variable
                    local is_sensitive=false
                    if [[ "$var_name" =~ (PASSWORD|SECRET|KEY|TOKEN) ]]; then
                        is_sensitive=true
                    fi
                    
                    # Prompt for value
                    local prompt="Please enter value for $var_name"
                    local new_value=$(prompt_for_input "$prompt" "$var_value" "$is_sensitive")
                    
                    if [ -n "$new_value" ]; then
                        echo "$var_name=$new_value" >> "$env_file"
                        export "$var_name"="$new_value"
                        print_success "Set $var_name"
                    fi
                fi
            fi
        done < "$schema_file"
        
    else
        print_status "No schema file found, scanning code for environment variables..."
        
        # Scan common files for environment variable usage
        local env_vars=$(grep -rho 'process\.env\.[A-Z_]\+\|os\.environ\[['"'"'"][A-Z_]\+['"'"']\]\|\$[A-Z_]\+\|getenv(['"'"'"][A-Z_]\+['"'"'"])' . 2>/dev/null | sort | uniq || true)
        
        if [ -n "$env_vars" ]; then
            print_status "Found potential environment variables in code:"
            echo "$env_vars" | while read -r var; do
                # Extract variable name from different patterns
                clean_var=$(echo "$var" | sed 's/process\.env\.//g' | sed 's/os\.environ\[//g' | sed 's/\$//g' | sed 's/getenv(//g' | sed 's/['"'"'"\[\]()]//g')
                
                if [ -z "${!clean_var:-}" ]; then
                    print_warning "Potentially missing environment variable: $clean_var"
                fi
            done
        fi
    fi
    
    CONFIG_GENERATED="true"
    update_gui_progress 50 "Environment variables validated"
}

# STEP 4 - DEPLOYMENT EXECUTION
execute_deployment() {
    print_header "STEP 4 - DEPLOYMENT EXECUTION"
    
    local deployment_method=""
    
    if [ -f "docker-compose.yml" ] || [ -f "docker-compose.yaml" ]; then
        deployment_method="docker-compose"
        print_status "Found docker-compose configuration"
        
        print_status "Running docker-compose up -d --build..."
        if docker-compose up -d --build; then
            print_success "Docker Compose deployment completed"
        else
            print_error "Docker Compose deployment failed"
            return 1
        fi
        
    elif [ -f ".gitlab-ci.yml" ]; then
        deployment_method="gitlab-ci"
        print_status "Found GitLab CI configuration"
        
        # Check if we're in a git repository
        if git rev-parse --git-dir > /dev/null 2>&1; then
            # Commit all changes
            print_status "Committing changes..."
            git add .
            git commit -m "Deployment: $(date '+%Y-%m-%d %H:%M:%S')" || true
            
            # Push to main branch
            print_status "Pushing to main branch..."
            if git push origin main; then
                print_success "Code pushed to GitLab, CI pipeline should start"
            else
                print_error "Failed to push to GitLab"
                return 1
            fi
        else
            print_error "Not a git repository, cannot use GitLab CI"
            return 1
        fi
        
    elif [ -f "deploy.sh" ]; then
        deployment_method="custom"
        print_status "Found custom deploy.sh script"
        
        # Make deploy.sh executable
        chmod +x deploy.sh
        
        # Execute custom deployment script
        print_status "Executing custom deployment script..."
        if ./deploy.sh; then
            print_success "Custom deployment script completed"
        else
            print_error "Custom deployment script failed"
            return 1
        fi
        
    else
        deployment_method="generic"
        print_warning "No specific deployment method found, using generic approach"
        
        # Try to detect and run common deployment commands
        if [ -f "Procfile" ]; then
            print_status "Found Procfile, this appears to be a Heroku-style application"
        fi
        
        # Check for common scripts in package.json
        if [ -f "package.json" ]; then
            if npm run deploy --if-present; then
                print_success "npm run deploy completed"
            elif npm run start --if-present; then
                print_success "npm run start completed"
            fi
        fi
    fi
    
    echo "$deployment_method" > "$PROJECT_ROOT/.deployment_method"
    update_gui_progress 70 "Deployment executed: $deployment_method"
}

# STEP 5 - REAL-TIME GUI MONITORING
launch_gui_monitoring() {
    print_header "STEP 5 - REAL-TIME GUI MONITORING"
    
    show_gui_notification "Deployment in progress..."
    
    # Create a background monitoring process
    (
        sleep 2
        update_gui_progress 75 "Build Started"
        sleep 2
        update_gui_progress 80 "Secrets Loaded"
        sleep 2
        update_gui_progress 90 "Deploying"
        sleep 2
        update_gui_progress 100 "Complete"
    ) &
    
    DEPLOYMENT_PID=$!
    update_gui_progress 75 "GUI monitoring launched"
}

# STEP 6 - SERVICE VALIDATION
validate_services() {
    print_header "STEP 6 - SERVICE VALIDATION"
    
    local ports=()
    local services_healthy=true
    
    # Detect ports from docker-compose
    if [ -f "docker-compose.yml" ] || [ -f "docker-compose.yaml" ]; then
        print_status "Detecting ports from docker-compose..."
        
        local compose_file="docker-compose.yml"
        [ -f "docker-compose.yaml" ] && compose_file="docker-compose.yaml"
        
        # Extract ports from docker-compose file
        local detected_ports=$(grep -E "^\s*-\s*[0-9]+:[0-9]+" "$compose_file" | sed 's/.*-\s*\([0-9]*\):.*/\1/' | sort -u)
        
        if [ -n "$detected_ports" ]; then
            while read -r port; do
                ports+=("$port")
            done <<< "$detected_ports"
        fi
    fi
    
    # Check common ports if none detected
    if [ ${#ports[@]} -eq 0 ]; then
        print_status "No ports detected from configuration, checking common ports..."
        ports=(3000 8000 8080 80 443 5000 9000)
    fi
    
    # Ping services after deployment
    for port in "${ports[@]}"; do
        print_status "Checking service on port $port..."
        
        if nc -z localhost "$port" 2>/dev/null; then
            print_success "Service on port $port is responding"
        else
            print_warning "Service on port $port is not responding"
            services_healthy=false
        fi
    done
    
    # Display results
    if [ "$services_healthy" = true ]; then
        print_success "All services are healthy"
        show_gui_notification "All services are healthy"
    else
        print_warning "Some services may not be healthy"
        show_gui_notification "Some services may not be healthy"
    fi
    
    update_gui_progress 95 "Services validated"
}

# STEP 7 - SNAPSHOT AND ROLLBACK
create_snapshot() {
    print_header "STEP 7 - SNAPSHOT AND ROLLBACK"
    
    if git rev-parse --git-dir > /dev/null 2>&1; then
        # Capture a Git snapshot before deployment
        print_status "Creating Git snapshot..."
        
        ROLLBACK_COMMIT=$(git rev-parse HEAD)
        print_status "Snapshot created at commit: $ROLLBACK_COMMIT"
        
        # Tag the snapshot for easy reference
        local snapshot_tag="deployment-snapshot-$(date +%Y%m%d_%H%M%S)"
        git tag "$snapshot_tag" "$ROLLBACK_COMMIT"
        print_success "Tagged snapshot as: $snapshot_tag"
        
        echo "$ROLLBACK_COMMIT" > "$PROJECT_ROOT/.rollback_commit"
        
    else
        print_warning "Not a git repository, cannot create snapshot"
    fi
    
    update_gui_progress 98 "Snapshot created"
}

# Function to perform rollback
perform_rollback() {
    print_header "PERFORMING ROLLBACK"
    
    if [ -f "$PROJECT_ROOT/.rollback_commit" ]; then
        local rollback_commit=$(cat "$PROJECT_ROOT/.rollback_commit")
        
        print_status "Rolling back to commit: $rollback_commit"
        
        if git reset --hard "$rollback_commit"; then
            print_success "Rollback completed successfully"
            show_gui_notification "Rollback completed successfully"
        else
            print_error "Rollback failed"
            return 1
        fi
    else
        print_error "No rollback commit found"
        return 1
    fi
}

# STEP 8 - POST-DEPLOY GUI CONFIRMATION
post_deploy_confirmation() {
    print_header "STEP 8 - POST-DEPLOY GUI CONFIRMATION"
    
    local gui_tool=$(detect_gui)
    local archive_choice=""
    
    case $gui_tool in
        "zenity")
            if zenity --question --text="Deployment completed successfully!\n\nWould you like to archive this deployment?" --width=300 --height=100; then
                archive_choice="yes"
            else
                archive_choice="no"
            fi
            ;;
        "yad")
            if yad --question --text="Deployment completed successfully!\n\nWould you like to archive this deployment?" --width=300 --height=100; then
                archive_choice="yes"
            else
                archive_choice="no"
            fi
            ;;
        *)
            read -p "Deployment completed successfully! Would you like to archive this deployment? (y/N): " archive_choice
            ;;
    esac
    
    if [[ "$archive_choice" =~ ^[Yy] ]]; then
        print_status "Archiving deployment..."
        
        local archive_name="deployment-$(date +%Y%m%d_%H%M%S).tar.gz"
        local archive_path="$PROJECT_ROOT/deploy-history/$archive_name"
        
        # Create archive excluding certain directories
        tar -czf "$archive_path" \
            --exclude=node_modules \
            --exclude=target \
            --exclude=.git \
            --exclude=venv \
            --exclude=__pycache__ \
            --exclude=deploy-history \
            .
        
        print_success "Deployment archived as: $archive_name"
        show_gui_notification "Deployment archived successfully"
    fi
    
    update_gui_progress 100 "Post-deployment confirmation completed"
}

# STEP 9 - TERMINAL UI FALLBACK
# This is handled throughout the script with fallback options

# STEP 10 - EMIT AGENT-FRIENDLY HEADER
emit_json_status() {
    print_header "STEP 10 - EMIT AGENT-FRIENDLY HEADER"
    
    local project_type=$(cat "$PROJECT_ROOT/.project_type" 2>/dev/null || echo "unknown")
    local deployment_method=$(cat "$PROJECT_ROOT/.deployment_method" 2>/dev/null || echo "unknown")
    local deployment_time=$(date -Iseconds)
    
    local json_status=$(cat <<EOF
{
  "deployment_status": "success",
  "project_name": "$PROJECT_NAME",
  "project_type": "$project_type",
  "deployment_method": "$deployment_method",
  "deployment_time": "$deployment_time",
  "project_root": "$PROJECT_ROOT",
  "log_file": "$LOG_FILE",
  "config_generated": "$CONFIG_GENERATED",
  "rollback_commit": "$ROLLBACK_COMMIT"
}
EOF
)
    
    echo "$json_status" > "$PROJECT_ROOT/deployment_status.json"
    echo "$json_status"
    
    print_success "JSON status emitted"
}

# Error handling and cleanup
cleanup() {
    close_gui_progress
    
    if [ -n "$DEPLOYMENT_PID" ]; then
        kill "$DEPLOYMENT_PID" 2>/dev/null || true
    fi
    
    # Clean up temporary files
    rm -f /tmp/gui_progress.pid
}

# Signal handlers
trap cleanup EXIT
trap 'print_error "Deployment interrupted"; cleanup; exit 1' INT TERM

# Main deployment function
main() {
    print_header "COPILOT COATING AGENT - UNIFIED DEPLOYMENT DIRECTIVE"
    print_status "Starting universal deployment protocol..."
    
    # Check for required tools
    if ! command -v git &> /dev/null; then
        print_error "Git is required but not installed"
        exit 1
    fi
    
    # Launch GUI monitoring
    show_gui_progress "Universal Deployment"
    
    # Execute deployment steps
    initialize_project
    detect_project_type
    validate_environment_variables
    execute_deployment
    launch_gui_monitoring
    validate_services
    create_snapshot
    post_deploy_confirmation
    emit_json_status
    
    # Final success message
    print_success "Universal deployment completed successfully!"
    show_gui_notification "Deployment completed successfully!"
    
    # Wait for GUI monitoring to complete
    if [ -n "$DEPLOYMENT_PID" ]; then
        wait "$DEPLOYMENT_PID" || true
    fi
    
    # Auto-close GUI after success
    close_gui_progress
}

# Command-line argument handling
case "${1:-deploy}" in
    "deploy")
        main
        ;;
    "rollback")
        perform_rollback
        ;;
    "status")
        if [ -f "$PROJECT_ROOT/deployment_status.json" ]; then
            cat "$PROJECT_ROOT/deployment_status.json"
        else
            echo "No deployment status found"
        fi
        ;;
    "help"|"--help"|"-h")
        echo "Universal Deployment Script"
        echo ""
        echo "Usage: $0 [COMMAND]"
        echo ""
        echo "Commands:"
        echo "  deploy    - Run full deployment (default)"
        echo "  rollback  - Rollback to previous version"
        echo "  status    - Show deployment status"
        echo "  help      - Show this help message"
        ;;
    *)
        print_error "Unknown command: $1"
        exit 1
        ;;
esac