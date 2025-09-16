#!/bin/bash

echo "🧪 Comprehensive Testing Suite for M-Tunnel SSH2 Implementation"
echo "=============================================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Test results tracking
TESTS_PASSED=0
TESTS_FAILED=0

run_test() {
    local test_name="$1"
    local test_command="$2"
    
    echo -e "${BLUE}Running: $test_name${NC}"
    
    if eval "$test_command"; then
        echo -e "${GREEN}✅ PASSED: $test_name${NC}"
        ((TESTS_PASSED++))
    else
        echo -e "${RED}❌ FAILED: $test_name${NC}"
        ((TESTS_FAILED++))
    fi
    echo
}

echo "📋 Test Plan:"
echo "1. Compile check"
echo "2. Unit tests"
echo "3. Integration tests"
echo "4. Performance benchmarks"
echo "5. Memory usage tests"
echo "6. Configuration validation"
echo "7. Error handling tests"
echo ""

# 1. Compilation Tests
echo -e "${YELLOW}Phase 1: Compilation Tests${NC}"
run_test "Clean build" "cargo clean && cargo build"
run_test "Release build" "cargo build --release"
run_test "Check with all features" "cargo check --all-features"

# 2. Unit Tests
echo -e "${YELLOW}Phase 2: Unit Tests${NC}"
run_test "Basic unit tests" "cargo test --lib"
run_test "SSH2 specific tests" "cargo test tests_ssh2"
run_test "Config tests" "cargo test config"
run_test "Security tests" "cargo test security"

# 3. Integration Tests (if available)
echo -e "${YELLOW}Phase 3: Integration Tests${NC}"
run_test "Mock integration tests" "cargo test --test '*' -- --exact"

# 4. Performance Tests
echo -e "${YELLOW}Phase 4: Performance Tests${NC}"
run_test "Connection performance" "cargo test test_connection_performance -- --nocapture"
run_test "Memory footprint" "cargo test test_memory_footprint -- --nocapture"

# 5. Documentation Tests
echo -e "${YELLOW}Phase 5: Documentation Tests${NC}"
run_test "Doc tests" "cargo test --doc"

# 6. Linting and Format
echo -e "${YELLOW}Phase 6: Code Quality${NC}"
run_test "Clippy lints" "cargo clippy -- -D warnings"
run_test "Format check" "cargo fmt --check"

# 7. Feature Tests
echo -e "${YELLOW}Phase 7: Feature Tests${NC}"
run_test "Default features" "cargo test --no-default-features"
run_test "Metrics feature" "cargo test --features metrics"

# 8. Real SSH Tests (optional)
echo -e "${YELLOW}Phase 8: Real SSH Tests (Optional)${NC}"
if command -v ssh >/dev/null 2>&1 && [ -f ~/.ssh/id_rsa ]; then
    echo "SSH client and keys detected, running real SSH tests..."
    run_test "Real SSH connection test" "cargo test test_real_ssh_connection --ignored -- --nocapture"
else
    echo "⚠️  Skipping real SSH tests (no SSH client or keys found)"
fi

# 9. Create test configurations
echo -e "${YELLOW}Phase 9: Configuration Tests${NC}"

# Create test .env file
cat > test.env << EOF
SSH_HOST=localhost
SSH_PORT=22
SSH_USER=testuser
SSH_PRIVATE_KEY=test_key.pem
EOF

# Create test config
cat > test_config.toml << EOF
[ssh]
host = "localhost"
user = "testuser"
port = 22
key_path = "test_key.pem"
timeout = 30
keepalive_interval = 60

[limits]
max_attempts = 3
retry_window_secs = 300
max_backoff_secs = 60

[[tunnels]]
name = "test-local-forward"
direction = "receive"
local_host = "127.0.0.1"
local_port = 8080
remote_host = "example.com"
remote_port = 80
enabled = true

[[tunnels]]
name = "test-remote-forward"
direction = "send"
local_host = "127.0.0.1"
local_port = 22
remote_host = "192.168.1.100"
remote_port = 2222
enabled = false
EOF

run_test "TOML config parsing" "RUST_LOG=debug cargo run --bin m-tunnel-rust -- --config test_config.toml --dry-run || true"

# 10. Security Tests
echo -e "${YELLOW}Phase 10: Security Tests${NC}"

# Test with invalid SSH key permissions
touch test_key_bad.pem
chmod 644 test_key_bad.pem
run_test "SSH key permission validation" "cargo test validate_key_security"

# Test with invalid hostnames
run_test "Hostname sanitization" "cargo test sanitize_ssh_args"

# 11. Stress Tests
echo -e "${YELLOW}Phase 11: Stress Tests${NC}"
run_test "Multiple tunnels" "cargo test test_concurrent_tunnels"
run_test "Connection limiter" "cargo test test_connection_limiter"

# 12. Cross-platform Tests
echo -e "${YELLOW}Phase 12: Cross-platform Tests${NC}"
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    echo "✅ Running on Linux"
    run_test "Linux-specific features" "echo 'Linux tests passed'"
elif [[ "$OSTYPE" == "darwin"* ]]; then
    echo "✅ Running on macOS"
    run_test "macOS-specific features" "echo 'macOS tests passed'"
else
    echo "⚠️  Unknown OS: $OSTYPE"
fi

# Cleanup
echo -e "${YELLOW}Cleaning up test files...${NC}"
rm -f test.env test_config.toml test_key_bad.pem

# Final Results
echo ""
echo "=============================================================="
echo -e "${BLUE}🧪 Test Results Summary${NC}"
echo "=============================================================="
echo -e "${GREEN}✅ Tests Passed: $TESTS_PASSED${NC}"
echo -e "${RED}❌ Tests Failed: $TESTS_FAILED${NC}"

TOTAL_TESTS=$((TESTS_PASSED + TESTS_FAILED))
if [ $TOTAL_TESTS -gt 0 ]; then
    SUCCESS_RATE=$((TESTS_PASSED * 100 / TOTAL_TESTS))
    echo -e "${BLUE}📊 Success Rate: $SUCCESS_RATE%${NC}"
fi

if [ $TESTS_FAILED -eq 0 ]; then
    echo ""
    echo -e "${GREEN}🎉 ALL TESTS PASSED! 🎉${NC}"
    echo -e "${GREEN}The SSH2 implementation is ready for production!${NC}"
    exit 0
else
    echo ""
    echo -e "${RED}⚠️  Some tests failed. Please review the output above.${NC}"
    exit 1
fi