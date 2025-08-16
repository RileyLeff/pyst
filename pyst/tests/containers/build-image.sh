#!/bin/bash
set -e

# Build the optimized pyst test image
echo "🏗️ Building optimized pyst test container..."

# Use the current directory as build context
cd "$(dirname "$0")"

# Build with explicit tag and platform support
docker build \
  --tag pyst-test:latest \
  --tag pyst-test:$(date +%Y%m%d) \
  --platform linux/amd64,linux/arm64 \
  --progress=plain \
  .

echo "✅ Pyst test image built successfully!"
echo "   Tag: pyst-test:latest"
echo "   Usage: docker run --rm -it pyst-test:latest"