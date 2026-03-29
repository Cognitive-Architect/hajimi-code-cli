#!/bin/bash
#
# 债务清偿一键验证脚本
# Usage: ./scripts/run-debt-clearance.sh
#

set -e

echo "╔══════════════════════════════════════════════════════════════╗"
echo "║     Hajimi Phase 2.1 债务清偿验证                            ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo ""

cd "$(dirname "$0")/.."

echo "📦 步骤 1/3: 运行债务清偿验证器..."
node src/test/debt-clearance-validator.js

echo ""
echo "📊 步骤 2/3: 运行性能基准回归..."
node src/test/phase2.1-benchmark.test.js

echo ""
echo "✅ 步骤 3/3: 验证完成"
echo ""
echo "检查点:"
echo "  [✓] DEBT-PHASE2-006: WAL自动截断"
echo "  [✓] DEBT-PHASE2-007: 并发写入安全"
echo "  [✓] DEBT-PHASE2-005: 二进制序列化"
echo "  [✓] 性能基线回归验证"
echo ""
