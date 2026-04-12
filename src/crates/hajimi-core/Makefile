# HAJIMI Core - Makefile
# 债务检查快捷命令

.PHONY: all check test debt-check clean

all: check test

check:
	cargo check

test:
	cargo test

# Windows 环境使用 PowerShell 脚本
debts:
	@echo "=== Debt Check ==="
	@powershell -ExecutionPolicy Bypass -File debt-check.ps1

debts-bash:
	@echo "=== Debt Check ==="
	@echo "Scanning for unwrap in production code..."
	@UNWRAP_COUNT=$$(find src -name "*.rs" -not -name "*test*" -exec grep -H "unwrap()" {} \; 2>/dev/null | grep -v "//.*unwrap" | wc -l); \
	if [ "$$UNWRAP_COUNT" -eq 0 ]; then \
		echo "✅ No unwrap in production code"; \
	else \
		echo "❌ Found $$UNWRAP_COUNT unwrap() in production code:"; \
		find src -name "*.rs" -not -name "*test*" -exec grep -H "unwrap()" {} \; | head -20; \
		exit 1; \
	fi
	@echo ""
	@echo "Scanning for panic!..."
	@PANIC_COUNT=$$(find src -name "*.rs" -not -name "*test*" -exec grep -H "panic!" {} \; 2>/dev/null | wc -l); \
	if [ "$$PANIC_COUNT" -eq 0 ]; then \
		echo "✅ No panic! in production code"; \
	else \
		echo "❌ Found $$PANIC_COUNT panic! in production code"; \
		exit 1; \
	fi
	@echo ""
	@echo "Running clippy..."
	@cargo clippy -- -D warnings 2>&1 | grep "error:" > /dev/null && echo "❌ Clippy errors found" && exit 1 || echo "✅ Clippy clean"
	@echo ""
	@echo "=== Debt Check Complete ==="

clean:
	cargo clean

# 债务状态查看
debts-list:
	@grep -E "DEBT-|CLEARED|NEXT-AUDIT|LAST-CLEARED" src/lib.rs

# 快速修复检查
debts-fix:
	@echo "Running cargo fix..."
	cargo fix --allow-dirty
	@echo "Running cargo fmt..."
	cargo fmt
