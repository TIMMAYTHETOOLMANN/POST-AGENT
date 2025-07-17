#!/bin/bash

# Simple deployment test script that doesn't require compilation
cd /tmp
mkdir -p test_deployment
cd test_deployment

# Create a simple test project
echo "Testing deployment script with simple project..."

# Create a minimal package.json for Node.js detection
cat > package.json << 'EOF'
{
  "name": "test-project",
  "version": "1.0.0",
  "scripts": {
    "build": "echo 'Build completed'",
    "start": "echo 'Starting application'"
  }
}
EOF

# Create a minimal .env.example
cat > .env.example << 'EOF'
NODE_ENV=production
PORT=3000
DATABASE_URL=postgresql://localhost:5432/test
API_KEY=your_api_key_here
SECRET_KEY=your_secret_key_here
EOF

# Initialize git
git init
git add .
git commit -m "Initial test project"

# Mock npm install and build commands
cat > npm << 'EOF'
#!/bin/bash
echo "Mock npm $@"
if [ "$1" = "install" ]; then
  echo "npm install completed successfully"
elif [ "$1" = "run" ] && [ "$2" = "build" ]; then
  echo "npm run build completed successfully"
else
  echo "npm command executed"
fi
EOF

chmod +x npm
export PATH="/tmp/test_deployment:$PATH"

# Test the deployment script
echo "Running deployment script..."
timeout 30 /home/runner/work/POST-AGENT/POST-AGENT/deploy.sh deploy

echo "Deployment test completed!"

# Clean up
cd /tmp
rm -rf test_deployment

echo "Test cleanup completed!"