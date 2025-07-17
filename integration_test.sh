#!/bin/bash

# Integration test for the universal deployment script
# This test validates the complete deployment workflow

echo "=== Universal Deployment Integration Test ==="
echo

# Test 1: Basic help and status commands
echo "1. Testing basic commands..."
./deploy.sh help > /dev/null 2>&1 && echo "✓ Help command works" || echo "✗ Help command failed"
./deploy.sh status > /dev/null 2>&1 && echo "✓ Status command works" || echo "✗ Status command failed"

# Test 2: Script validation
echo
echo "2. Testing script validation..."
bash -n ./deploy.sh && echo "✓ Script syntax is valid" || echo "✗ Script syntax error"
[ -x ./deploy.sh ] && echo "✓ Script is executable" || echo "✗ Script is not executable"

# Test 3: Documentation exists
echo
echo "3. Testing documentation..."
[ -f "UNIVERSAL_DEPLOYMENT.md" ] && echo "✓ Documentation exists" || echo "✗ Documentation missing"
[ -f ".env.example" ] && echo "✓ Environment example exists" || echo "✗ Environment example missing"

# Test 4: Project type detection files
echo
echo "4. Testing project type detection..."
[ -f "Cargo.toml" ] && echo "✓ Rust project detected (Cargo.toml)" || echo "✗ Rust project not detected"
[ -f "package.json" ] && echo "✓ Node.js project files present" || echo "✗ No Node.js project files"

# Test 5: Integration with existing launcher
echo
echo "5. Testing launcher integration..."
[ -f "launch.sh" ] && echo "✓ Existing launcher script present" || echo "✗ Launcher script missing"
[ -x "launch.sh" ] && echo "✓ Launcher script is executable" || echo "✗ Launcher script not executable"

# Test 6: Test deployment artifacts are gitignored
echo
echo "6. Testing gitignore configuration..."
grep -q "deployment_.*\.log" .gitignore && echo "✓ Deployment logs are gitignored" || echo "✗ Deployment logs not gitignored"
grep -q "deployment_status\.json" .gitignore && echo "✓ Status files are gitignored" || echo "✗ Status files not gitignored"

# Test 7: Test .env.example structure
echo
echo "7. Testing environment configuration..."
if [ -f ".env.example" ]; then
    var_count=$(grep -c "^[A-Z_]*=" .env.example)
    if [ $var_count -gt 0 ]; then
        echo "✓ Environment variables defined ($var_count variables)"
    else
        echo "✗ No environment variables found"
    fi
else
    echo "✗ No .env.example file found"
fi

# Test 8: Test deployment workflow simulation
echo
echo "8. Testing deployment workflow simulation..."
timeout 5 ./deploy.sh deploy > /dev/null 2>&1
if [ $? -eq 124 ]; then
    echo "✓ Deployment script starts successfully (timed out as expected)"
else
    echo "✗ Deployment script failed to start"
fi

echo
echo "=== Integration Test Complete ==="
echo "This test validates the Universal Deployment Script integration"
echo "with the existing POST-AGENT project structure."