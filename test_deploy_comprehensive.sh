#!/bin/bash

# Comprehensive test suite for the Universal Deployment Script

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test counters
TESTS_PASSED=0
TESTS_FAILED=0
TESTS_TOTAL=0

# Function to print test results
print_test_result() {
    local test_name="$1"
    local result="$2"
    local details="$3"
    
    TESTS_TOTAL=$((TESTS_TOTAL + 1))
    
    if [ "$result" = "PASS" ]; then
        echo -e "${GREEN}[PASS]${NC} $test_name"
        TESTS_PASSED=$((TESTS_PASSED + 1))
    else
        echo -e "${RED}[FAIL]${NC} $test_name"
        if [ -n "$details" ]; then
            echo -e "${YELLOW}  Details: $details${NC}"
        fi
        TESTS_FAILED=$((TESTS_FAILED + 1))
    fi
}

# Function to run a test
run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}[TEST]${NC} Running: $test_name"
    
    if eval "$test_command"; then
        print_test_result "$test_name" "PASS"
    else
        print_test_result "$test_name" "FAIL" "Command failed: $test_command"
    fi
}

# Function to test project type detection
test_project_type_detection() {
    echo -e "${BLUE}[TEST SUITE]${NC} Testing project type detection..."
    
    # Test Node.js project detection
    (
        cd /tmp
        rm -rf test_nodejs
        mkdir -p test_nodejs
        cd test_nodejs
        echo '{"name": "test", "scripts": {"build": "echo build"}}' > package.json
        git init && git add . && git commit -m "test" > /dev/null 2>&1
        
        # Mock npm command
        echo '#!/bin/bash
echo "Mock npm $@"
exit 0' > npm
        chmod +x npm
        export PATH="/tmp/test_nodejs:$PATH"
        
        timeout 30 /home/runner/work/POST-AGENT/POST-AGENT/deploy.sh deploy > /dev/null 2>&1
        
        if [ -f ".project_type" ] && [ "$(cat .project_type)" = "nodejs" ]; then
            exit 0
        else
            exit 1
        fi
    )
    
    local nodejs_result=$?
    if [ $nodejs_result -eq 0 ]; then
        print_test_result "Node.js project detection" "PASS"
    else
        print_test_result "Node.js project detection" "FAIL"
    fi
    
    # Test Python project detection
    (
        cd /tmp
        rm -rf test_python
        mkdir -p test_python
        cd test_python
        echo "requests==2.25.1" > requirements.txt
        git init && git add . && git commit -m "test" > /dev/null 2>&1
        
        # Mock python3 and pip commands
        echo '#!/bin/bash
if [ "$1" = "-m" ] && [ "$2" = "venv" ]; then
    mkdir -p venv/bin
    echo "#!/bin/bash" > venv/bin/activate
    echo "Virtual environment created"
else
    echo "Python command: $@"
fi
exit 0' > python3
        chmod +x python3
        
        echo '#!/bin/bash
echo "Mock pip install $@"
exit 0' > pip
        chmod +x pip
        
        export PATH="/tmp/test_python:$PATH"
        
        timeout 30 /home/runner/work/POST-AGENT/POST-AGENT/deploy.sh deploy > /dev/null 2>&1
        
        if [ -f ".project_type" ] && [ "$(cat .project_type)" = "python" ]; then
            exit 0
        else
            exit 1
        fi
    )
    
    local python_result=$?
    if [ $python_result -eq 0 ]; then
        print_test_result "Python project detection" "PASS"
    else
        print_test_result "Python project detection" "FAIL"
    fi
    
    # Test Docker project detection
    (
        cd /tmp
        rm -rf test_docker
        mkdir -p test_docker
        cd test_docker
        echo "FROM alpine:latest" > Dockerfile
        git init && git add . && git commit -m "test" > /dev/null 2>&1
        
        # Mock docker command
        echo '#!/bin/bash
echo "Mock docker build $@"
exit 0' > docker
        chmod +x docker
        
        export PATH="/tmp/test_docker:$PATH"
        
        timeout 30 /home/runner/work/POST-AGENT/POST-AGENT/deploy.sh deploy > /dev/null 2>&1
        
        if [ -f ".project_type" ] && [ "$(cat .project_type)" = "docker" ]; then
            exit 0
        else
            exit 1
        fi
    )
    
    local docker_result=$?
    if [ $docker_result -eq 0 ]; then
        print_test_result "Docker project detection" "PASS"
    else
        print_test_result "Docker project detection" "FAIL"
    fi
    
    # Cleanup
    rm -rf /tmp/test_nodejs /tmp/test_python /tmp/test_docker
}

# Function to test environment variable validation
test_env_validation() {
    echo -e "${BLUE}[TEST SUITE]${NC} Testing environment variable validation..."
    
    cd /tmp
    mkdir -p test_env
    cd test_env
    
    # Create .env.example file
    cat > .env.example << 'EOF'
NODE_ENV=production
PORT=3000
API_KEY=test_key
SECRET_KEY=test_secret
EOF
    
    # Create existing .env file
    cat > .env << 'EOF'
NODE_ENV=development
PORT=8000
EOF
    
    echo '{"name": "test"}' > package.json
    git init && git add . && git commit -m "test" > /dev/null 2>&1
    
    # Mock npm command
    echo '#!/bin/bash
exit 0' > npm
    chmod +x npm
    export PATH="/tmp/test_env:$PATH"
    
    # Test environment variable detection
    timeout 30 ../../../deploy.sh deploy > /dev/null 2>&1
    
    if [ -f ".env" ]; then
        print_test_result "Environment file creation" "PASS"
    else
        print_test_result "Environment file creation" "FAIL"
    fi
    
    # Cleanup
    cd /tmp
    rm -rf test_env
}

# Function to test deployment methods
test_deployment_methods() {
    echo -e "${BLUE}[TEST SUITE]${NC} Testing deployment methods..."
    
    # Test custom deploy.sh detection
    cd /tmp
    mkdir -p test_custom_deploy
    cd test_custom_deploy
    
    cat > deploy.sh << 'EOF'
#!/bin/bash
echo "Custom deployment executed"
exit 0
EOF
    chmod +x deploy.sh
    
    echo '{"name": "test"}' > package.json
    git init && git add . && git commit -m "test" > /dev/null 2>&1
    
    # Mock npm command
    echo '#!/bin/bash
exit 0' > npm
    chmod +x npm
    export PATH="/tmp/test_custom_deploy:$PATH"
    
    timeout 30 ../../../deploy.sh deploy > /dev/null 2>&1
    
    if [ -f ".deployment_method" ] && [ "$(cat .deployment_method)" = "custom" ]; then
        print_test_result "Custom deployment method detection" "PASS"
    else
        print_test_result "Custom deployment method detection" "FAIL"
    fi
    
    # Cleanup
    cd /tmp
    rm -rf test_custom_deploy
}

# Function to test basic script functionality
test_basic_functionality() {
    echo -e "${BLUE}[TEST SUITE]${NC} Testing basic script functionality..."
    
    # Test help command
    run_test "Help command" "cd /home/runner/work/POST-AGENT/POST-AGENT && ./deploy.sh help > /dev/null 2>&1"
    
    # Test status command
    run_test "Status command" "cd /home/runner/work/POST-AGENT/POST-AGENT && ./deploy.sh status > /dev/null 2>&1"
    
    # Test script permissions
    run_test "Script is executable" "[ -x /home/runner/work/POST-AGENT/POST-AGENT/deploy.sh ]"
    
    # Test script syntax
    run_test "Script syntax check" "bash -n /home/runner/work/POST-AGENT/POST-AGENT/deploy.sh"
}

# Function to test logging functionality
test_logging() {
    echo -e "${BLUE}[TEST SUITE]${NC} Testing logging functionality..."
    
    cd /tmp
    mkdir -p test_logging
    cd test_logging
    
    echo '{"name": "test", "scripts": {"build": "echo build"}}' > package.json
    git init && git add . && git commit -m "test" > /dev/null 2>&1
    
    # Mock npm command
    echo '#!/bin/bash
exit 0' > npm
    chmod +x npm
    export PATH="/tmp/test_logging:$PATH"
    
    timeout 30 ../../../deploy.sh deploy > /dev/null 2>&1
    
    # Check if log file was created
    if ls deployment_*.log > /dev/null 2>&1; then
        print_test_result "Log file creation" "PASS"
    else
        print_test_result "Log file creation" "FAIL"
    fi
    
    # Check if log contains expected content
    if ls deployment_*.log > /dev/null 2>&1 && grep -q "STEP 1 - PROJECT INITIALIZATION" deployment_*.log; then
        print_test_result "Log content validation" "PASS"
    else
        print_test_result "Log content validation" "FAIL"
    fi
    
    # Cleanup
    cd /tmp
    rm -rf test_logging
}

# Function to test JSON output
test_json_output() {
    echo -e "${BLUE}[TEST SUITE]${NC} Testing JSON output..."
    
    cd /tmp
    mkdir -p test_json
    cd test_json
    
    echo '{"name": "test"}' > package.json
    git init && git add . && git commit -m "test" > /dev/null 2>&1
    
    # Mock npm command
    echo '#!/bin/bash
exit 0' > npm
    chmod +x npm
    export PATH="/tmp/test_json:$PATH"
    
    timeout 30 ../../../deploy.sh deploy > /dev/null 2>&1
    
    # Check if JSON status file was created
    if [ -f "deployment_status.json" ]; then
        print_test_result "JSON status file creation" "PASS"
    else
        print_test_result "JSON status file creation" "FAIL"
    fi
    
    # Check if JSON is valid
    if [ -f "deployment_status.json" ] && python3 -c "import json; json.load(open('deployment_status.json'))" > /dev/null 2>&1; then
        print_test_result "JSON validity check" "PASS"
    else
        print_test_result "JSON validity check" "FAIL"
    fi
    
    # Cleanup
    cd /tmp
    rm -rf test_json
}

# Main test execution
main() {
    echo -e "${BLUE}[TEST SUITE]${NC} Universal Deployment Script Test Suite"
    echo "=================================================="
    
    # Check if script exists
    if [ ! -f "/home/runner/work/POST-AGENT/POST-AGENT/deploy.sh" ]; then
        echo -e "${RED}[ERROR]${NC} deploy.sh not found!"
        exit 1
    fi
    
    # Run test suites
    test_basic_functionality
    test_project_type_detection
    test_env_validation
    test_deployment_methods
    test_logging
    test_json_output
    
    # Print summary
    echo ""
    echo "=================================================="
    echo -e "${BLUE}[SUMMARY]${NC} Test Results:"
    echo "  Total Tests: $TESTS_TOTAL"
    echo -e "  Passed: ${GREEN}$TESTS_PASSED${NC}"
    echo -e "  Failed: ${RED}$TESTS_FAILED${NC}"
    
    if [ $TESTS_FAILED -eq 0 ]; then
        echo -e "${GREEN}[SUCCESS]${NC} All tests passed!"
        exit 0
    else
        echo -e "${RED}[FAILURE]${NC} Some tests failed!"
        exit 1
    fi
}

# Run the test suite
main "$@"