#!/bin/bash

# TikTok Cross-posting Agent Launcher
# This script handles building, configuration, and launching of the POST-AGENT

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Function to check dependencies
check_dependencies() {
    print_status "Checking dependencies..."
    
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo is not installed. Please install Rust from https://rustup.rs/"
        exit 1
    fi
    
    print_success "All dependencies are available"
}

# Function to build the project
build_project() {
    print_status "Building the project..."
    
    if cargo build --release; then
        print_success "Project built successfully"
    else
        print_error "Failed to build the project"
        exit 1
    fi
}

# Function to install dependencies
install_dependencies() {
    print_status "Installing/updating dependencies..."
    
    if cargo update; then
        print_success "Dependencies updated successfully"
    else
        print_warning "Some dependencies may have issues, but continuing..."
    fi
}

# Function to run config wizard
run_config_wizard() {
    print_status "Running configuration wizard..."
    
    if [ -f "config.toml" ]; then
        print_warning "Existing config.toml found"
        read -p "Do you want to reconfigure? (y/N): " choice
        case "$choice" in 
            y|Y ) 
                print_status "Running configuration wizard..."
                if cargo run --bin config_wizard; then
                    print_success "Configuration completed"
                else
                    print_error "Configuration failed"
                    exit 1
                fi
                ;;
            * ) 
                print_status "Using existing configuration"
                ;;
        esac
    else
        print_status "No configuration found, running wizard..."
        if cargo run --bin config_wizard; then
            print_success "Configuration completed"
        else
            print_error "Configuration failed"
            exit 1
        fi
    fi
}

# Function to validate config
validate_config() {
    print_status "Validating configuration..."
    
    if [ ! -f "config.toml" ]; then
        print_error "No configuration file found. Please run configuration first."
        return 1
    fi
    
    # Try to load and validate config by running a quick check
    if cargo run -- --help &> /dev/null; then
        print_success "Configuration appears to be valid"
    else
        print_warning "Configuration may have issues"
    fi
}

# Function to launch main agent
launch_agent() {
    print_status "Launching TikTok Cross-posting Agent..."
    
    if [ ! -f "config.toml" ]; then
        print_error "No configuration file found. Please run configuration first."
        exit 1
    fi
    
    print_status "Starting main agent..."
    cargo run --release
}

# Function to launch background watcher
launch_watcher() {
    print_status "Launching background watcher..."
    
    if [ ! -f "config.toml" ]; then
        print_error "No configuration file found. Please run configuration first."
        exit 1
    fi
    
    print_status "Starting background watcher (this will run the main agent in daemon mode)..."
    nohup cargo run --release > agent.log 2>&1 &
    echo $! > agent.pid
    print_success "Background watcher started with PID $(cat agent.pid)"
    print_status "Logs are being written to agent.log"
}

# Function to stop background watcher
stop_watcher() {
    if [ -f "agent.pid" ]; then
        local pid=$(cat agent.pid)
        print_status "Stopping background watcher (PID: $pid)..."
        if kill $pid 2>/dev/null; then
            print_success "Background watcher stopped"
            rm -f agent.pid
        else
            print_warning "Failed to stop process or process was already stopped"
            rm -f agent.pid
        fi
    else
        print_warning "No background watcher PID file found"
    fi
}

# Function to show status
show_status() {
    print_status "=== TikTok Cross-posting Agent Status ==="
    
    # Check if config exists
    if [ -f "config.toml" ]; then
        print_success "Configuration file: Found"
    else
        print_error "Configuration file: Missing"
    fi
    
    # Check if binary is built
    if [ -f "target/release/tiktok-crosspost" ]; then
        print_success "Binary: Built"
    else
        print_warning "Binary: Not built"
    fi
    
    # Check if background watcher is running
    if [ -f "agent.pid" ]; then
        local pid=$(cat agent.pid)
        if ps -p $pid > /dev/null 2>&1; then
            print_success "Background watcher: Running (PID: $pid)"
        else
            print_error "Background watcher: PID file exists but process is not running"
            rm -f agent.pid
        fi
    else
        print_warning "Background watcher: Not running"
    fi
    
    # Check log file
    if [ -f "agent.log" ]; then
        local log_size=$(wc -l < agent.log)
        print_status "Log file: $log_size lines"
    else
        print_status "Log file: No log file"
    fi
}

# Function to show help
show_help() {
    echo "TikTok Cross-posting Agent Launcher"
    echo ""
    echo "Usage: $0 [COMMAND]"
    echo ""
    echo "Commands:"
    echo "  setup          - Complete setup (install deps, build, configure)"
    echo "  build          - Build the project"
    echo "  deps           - Install/update dependencies"
    echo "  config         - Run configuration wizard"
    echo "  validate       - Validate configuration"
    echo "  run            - Launch main agent (foreground)"
    echo "  start          - Start background watcher"
    echo "  stop           - Stop background watcher"
    echo "  status         - Show status"
    echo "  logs           - Show recent log entries"
    echo "  help           - Show this help message"
    echo ""
    echo "For first-time setup, run: $0 setup"
}

# Function to show logs
show_logs() {
    if [ -f "agent.log" ]; then
        print_status "Showing last 50 lines of agent.log:"
        tail -n 50 agent.log
    else
        print_warning "No log file found"
    fi
}

# Main script logic
case "${1:-help}" in
    "setup")
        print_status "=== Setting up TikTok Cross-posting Agent ==="
        check_dependencies
        install_dependencies
        build_project
        run_config_wizard
        validate_config
        print_success "Setup completed! You can now run '$0 run' to start the agent"
        ;;
    "build")
        check_dependencies
        build_project
        ;;
    "deps")
        check_dependencies
        install_dependencies
        ;;
    "config")
        check_dependencies
        run_config_wizard
        ;;
    "validate")
        validate_config
        ;;
    "run")
        check_dependencies
        validate_config
        launch_agent
        ;;
    "start")
        check_dependencies
        validate_config
        launch_watcher
        ;;
    "stop")
        stop_watcher
        ;;
    "status")
        show_status
        ;;
    "logs")
        show_logs
        ;;
    "help"|"--help"|"-h")
        show_help
        ;;
    *)
        print_error "Unknown command: $1"
        show_help
        exit 1
        ;;
esac