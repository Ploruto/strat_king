#!/bin/bash

# Build the Rust project first (debug mode for faster builds)
echo "🦀 Building Rust project..."
cargo build

# Check if build was successful
if [ $? -ne 0 ]; then
    echo "❌ Rust build failed!"
    exit 1
fi

# Build Docker image using the debug binary
echo "🐳 Building Docker image..."
docker build -t strat-king-server:latest .

# Check if Docker build was successful
if [ $? -eq 0 ]; then
    echo "✅ Docker image built successfully!"
    echo "📋 Image: strat-king-server:latest"
    echo ""
    echo "🧪 Test run command:"
    echo 'SERVER_SECRET="test123" MATCH_ID="1" EXPECTED_PLAYERS="[1,2]" docker run --rm -p 7777:7777 strat-king-server:latest'
else
    echo "❌ Docker build failed!"
    exit 1
fi