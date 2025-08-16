#!/bin/bash
set -e

echo "🧪 Testing CI workflow locally..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to run a command and report result
run_check() {
    local name="$1"
    local cmd="$2"
    
    echo -e "${BLUE}🔍 Running: $name${NC}"
    if eval "$cmd"; then
        echo -e "${GREEN}✅ $name passed${NC}"
        return 0
    else
        echo -e "${RED}❌ $name failed${NC}"
        return 1
    fi
}

cd "$(dirname "$0")/.."

echo "🚀 Starting local CI validation (hybrid approach)..."

# Quick checks (similar to CI quick-checks job)
echo -e "\n${BLUE}=== Quick Checks ===${NC}"
run_check "Rust format check" "cargo fmt --all -- --check"
run_check "Clippy lints" "cargo clippy --all-targets --all-features -- -D warnings"
run_check "Cargo check" "cargo check --all-targets --all-features"

# Native tests (current platform only for local testing)
echo -e "\n${BLUE}=== Native Tests (Current Platform) ===${NC}"
run_check "Cargo build" "cargo build --verbose"
run_check "Unit tests" "cargo test --lib --bins --verbose"
run_check "Basic integration tests" "cargo test --test integration_tests --verbose"

# Container-based comprehensive tests
echo -e "\n${BLUE}=== Container Tests (Linux) ===${NC}"
cd pyst/tests/containers
run_check "Docker image build" "docker build --tag pyst-test:latest --progress=plain ."
run_check "Docker image verification" "docker run --rm pyst-test:latest sh -c 'rustc --version && cargo --version && uv --version && python --version'"
cd ../../..

run_check "Container infrastructure tests" "cargo test --package pyst containers -- --nocapture"
run_check "Comprehensive integration tests" "cargo test --package pyst integration -- --nocapture"
run_check "Optimized workflow test" "cargo test --package pyst test_optimized_pyst_workflow -- --nocapture"

# Optional: Full end-to-end (uncomment to run)
# echo -e "\n${BLUE}=== Full End-to-End (Optional) ===${NC}"
# run_check "Full development setup" "timeout 300 cargo test --package pyst test_full_dev_setup -- --nocapture"

echo -e "\n${GREEN}🎉 Local CI validation completed successfully!${NC}"
echo -e "${BLUE}💡 Your code is ready for CI pipeline${NC}"

# Clean up Docker resources
echo -e "\n${BLUE}🧹 Cleaning up Docker resources...${NC}"
docker container prune -f >/dev/null 2>&1 || true
docker image prune -f >/dev/null 2>&1 || true

echo -e "${GREEN}✨ All done!${NC}"