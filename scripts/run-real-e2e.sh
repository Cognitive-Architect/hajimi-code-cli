#!/bin/bash
# 真实Yjs+LevelDB E2E测试运行脚本
# DEBT-TEST-001清偿执行脚本

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
DOCKERFILE="$PROJECT_ROOT/docker/test-e2e.Dockerfile"
IMAGE_TAG="hajimi-real-e2e:latest"
CONTAINER_NAME="hajimi-real-e2e-test"

echo "[run-real-e2e] Building Docker image..."
docker build -f "$DOCKERFILE" -t "$IMAGE_TAG" "$PROJECT_ROOT"

echo "[run-real-e2e] Running E2E tests in container..."
docker run --rm \
  --name "$CONTAINER_NAME" \
  -v "$PROJECT_ROOT/data:/app/data" \
  -e "TEST_LOG_LEVEL=debug" \
  "$IMAGE_TAG" 2>&1 | tee "$PROJECT_ROOT/data/real-e2e-result.log"

EXIT_CODE=${PIPESTATUS[0]}

echo "[run-real-e2e] ================================="
if [ $EXIT_CODE -eq 0 ]; then
  echo "[run-real-e2e] ✅ All tests PASSED"
  echo "[run-real-e2e] DEBT-TEST-001: 已清偿"
else
  echo "[run-real-e2e] ❌ Tests FAILED (exit: $EXIT_CODE)"
fi
echo "[run-real-e2e] Log: data/real-e2e-result.log"
echo "[run-real-e2e] ================================="

exit $EXIT_CODE
