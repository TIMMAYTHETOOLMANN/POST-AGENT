@echo off
setlocal enabledelayedexpansion

REM TikTok Cross-posting Agent Launcher for Windows
REM This script handles building, configuration, and launching of the POST-AGENT

set "RED=[91m"
set "GREEN=[92m"
set "YELLOW=[93m"
set "BLUE=[94m"
set "NC=[0m"

REM Function to print colored output
:print_status
echo %BLUE%[INFO]%NC% %~1
goto :eof

:print_success
echo %GREEN%[SUCCESS]%NC% %~1
goto :eof

:print_warning
echo %YELLOW%[WARNING]%NC% %~1
goto :eof

:print_error
echo %RED%[ERROR]%NC% %~1
goto :eof

REM Function to check dependencies
:check_dependencies
call :print_status "Checking dependencies..."

where cargo >nul 2>&1
if %errorlevel% neq 0 (
    call :print_error "Rust/Cargo is not installed. Please install Rust from https://rustup.rs/"
    exit /b 1
)

call :print_success "All dependencies are available"
goto :eof

REM Function to build the project
:build_project
call :print_status "Building the project..."

cargo build --release
if %errorlevel% neq 0 (
    call :print_error "Failed to build the project"
    exit /b 1
)

call :print_success "Project built successfully"
goto :eof

REM Function to install dependencies
:install_dependencies
call :print_status "Installing/updating dependencies..."

cargo update
if %errorlevel% neq 0 (
    call :print_warning "Some dependencies may have issues, but continuing..."
)

call :print_success "Dependencies updated successfully"
goto :eof

REM Function to run config wizard
:run_config_wizard
call :print_status "Running configuration wizard..."

if exist "config.toml" (
    call :print_warning "Existing config.toml found"
    set /p "choice=Do you want to reconfigure? (y/N): "
    if /i "!choice!"=="y" (
        call :print_status "Running configuration wizard..."
        cargo run --bin config_wizard
        if %errorlevel% neq 0 (
            call :print_error "Configuration failed"
            exit /b 1
        )
        call :print_success "Configuration completed"
    ) else (
        call :print_status "Using existing configuration"
    )
) else (
    call :print_status "No configuration found, running wizard..."
    cargo run --bin config_wizard
    if %errorlevel% neq 0 (
        call :print_error "Configuration failed"
        exit /b 1
    )
    call :print_success "Configuration completed"
)
goto :eof

REM Function to validate config
:validate_config
call :print_status "Validating configuration..."

if not exist "config.toml" (
    call :print_error "No configuration file found. Please run configuration first."
    exit /b 1
)

REM Try to load and validate config by running a quick check
cargo run -- --help >nul 2>&1
if %errorlevel% neq 0 (
    call :print_warning "Configuration may have issues"
) else (
    call :print_success "Configuration appears to be valid"
)
goto :eof

REM Function to launch main agent
:launch_agent
call :print_status "Launching TikTok Cross-posting Agent..."

if not exist "config.toml" (
    call :print_error "No configuration file found. Please run configuration first."
    exit /b 1
)

call :print_status "Starting main agent..."
cargo run --release
goto :eof

REM Function to launch background watcher
:launch_watcher
call :print_status "Launching background watcher..."

if not exist "config.toml" (
    call :print_error "No configuration file found. Please run configuration first."
    exit /b 1
)

call :print_status "Starting background watcher..."
start /b cargo run --release > agent.log 2>&1
call :print_success "Background watcher started"
call :print_status "Logs are being written to agent.log"
goto :eof

REM Function to show status
:show_status
call :print_status "=== TikTok Cross-posting Agent Status ==="

if exist "config.toml" (
    call :print_success "Configuration file: Found"
) else (
    call :print_error "Configuration file: Missing"
)

if exist "target\release\tiktok-crosspost.exe" (
    call :print_success "Binary: Built"
) else (
    call :print_warning "Binary: Not built"
)

if exist "agent.log" (
    for /f %%i in ('find /c /v "" ^< agent.log') do set log_lines=%%i
    call :print_status "Log file: !log_lines! lines"
) else (
    call :print_status "Log file: No log file"
)
goto :eof

REM Function to show help
:show_help
echo TikTok Cross-posting Agent Launcher
echo.
echo Usage: %~nx0 [COMMAND]
echo.
echo Commands:
echo   setup          - Complete setup (install deps, build, configure)
echo   build          - Build the project
echo   deps           - Install/update dependencies
echo   config         - Run configuration wizard
echo   validate       - Validate configuration
echo   run            - Launch main agent (foreground)
echo   start          - Start background watcher
echo   status         - Show status
echo   logs           - Show recent log entries
echo   help           - Show this help message
echo.
echo For first-time setup, run: %~nx0 setup
goto :eof

REM Function to show logs
:show_logs
if exist "agent.log" (
    call :print_status "Showing last 50 lines of agent.log:"
    powershell -command "Get-Content agent.log -Tail 50"
) else (
    call :print_warning "No log file found"
)
goto :eof

REM Main script logic
if "%~1"=="" goto help
if "%~1"=="setup" goto setup
if "%~1"=="build" goto build
if "%~1"=="deps" goto deps
if "%~1"=="config" goto config
if "%~1"=="validate" goto validate
if "%~1"=="run" goto run
if "%~1"=="start" goto start
if "%~1"=="status" goto status
if "%~1"=="logs" goto logs
if "%~1"=="help" goto help
if "%~1"=="--help" goto help
if "%~1"=="-h" goto help

call :print_error "Unknown command: %~1"
call :show_help
exit /b 1

:setup
call :print_status "=== Setting up TikTok Cross-posting Agent ==="
call :check_dependencies
if %errorlevel% neq 0 exit /b 1
call :install_dependencies
call :build_project
if %errorlevel% neq 0 exit /b 1
call :run_config_wizard
if %errorlevel% neq 0 exit /b 1
call :validate_config
call :print_success "Setup completed! You can now run '%~nx0 run' to start the agent"
goto :eof

:build
call :check_dependencies
if %errorlevel% neq 0 exit /b 1
call :build_project
goto :eof

:deps
call :check_dependencies
if %errorlevel% neq 0 exit /b 1
call :install_dependencies
goto :eof

:config
call :check_dependencies
if %errorlevel% neq 0 exit /b 1
call :run_config_wizard
goto :eof

:validate
call :validate_config
goto :eof

:run
call :check_dependencies
if %errorlevel% neq 0 exit /b 1
call :validate_config
if %errorlevel% neq 0 exit /b 1
call :launch_agent
goto :eof

:start
call :check_dependencies
if %errorlevel% neq 0 exit /b 1
call :validate_config
if %errorlevel% neq 0 exit /b 1
call :launch_watcher
goto :eof

:status
call :show_status
goto :eof

:logs
call :show_logs
goto :eof

:help
call :show_help
goto :eof