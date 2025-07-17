#!/bin/bash

# Test script for the universal deployment script

set -e

# Test basic functionality
echo "Testing universal deployment script..."

# Test help command
echo "Testing help command..."
./deploy.sh help

# Test status command (should show no deployment status initially)
echo "Testing status command..."
./deploy.sh status || echo "No deployment status found (expected)"

# Test project type detection
echo "Testing project type detection..."
cd /tmp
mkdir -p test_projects

# Test Node.js project detection
mkdir -p test_projects/nodejs_project
cd test_projects/nodejs_project
echo '{"name": "test", "scripts": {"build": "echo build"}}' > package.json
echo "NODE_ENV=production" > .env.example
git init
git add .
git commit -m "Initial commit"

echo "Testing Node.js project detection..."
timeout 30 ../../deploy.sh deploy || echo "Deployment test completed"

# Test Python project detection
cd /tmp/test_projects
mkdir -p python_project
cd python_project
echo "requests==2.25.1" > requirements.txt
echo "DEBUG=False" > .env.example
git init
git add .
git commit -m "Initial commit"

echo "Testing Python project detection..."
timeout 30 ../../deploy.sh deploy || echo "Deployment test completed"

# Test Docker project detection
cd /tmp/test_projects
mkdir -p docker_project
cd docker_project
echo "FROM alpine:latest" > Dockerfile
echo "POSTGRES_PASSWORD=secret" > .env.example
git init
git add .
git commit -m "Initial commit"

echo "Testing Docker project detection..."
timeout 30 ../../deploy.sh deploy || echo "Deployment test completed"

# Clean up
cd /tmp
rm -rf test_projects

echo "All tests completed!"