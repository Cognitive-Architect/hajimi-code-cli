#!/bin/bash
# Real Network E2E Test Runner - 30秒熔断
set -e

echo "[RUNNER] Starting real network E2E test..."
echo "[RUNNER] Timeout: 30 seconds"

# Run test with timeout
timeout 30 node tests/datachannel-real-network.e2e.js

echo "[RUNNER] Test completed successfully"
