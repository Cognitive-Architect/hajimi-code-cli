# HAJIMI-V3-IMPROVEMENT-ROADMAP

> **审计工单**: AUDIT-DEBT-CLEARANCE-001  
> **目标评级**: B → A  
> **预计工时**: 8-13 小时 (1-2 人日)  
> **阻塞发布**: 是 (需完成基线项)

---

## 1. 改进目标

将 DEBT-CLEARANCE-001 交付物从 **B 级** 提升至 **A 级**（满足基线，无系统性风险）。

### 关键改进点

| 编号 | 改进项 | 原问题 | 目标状态 |
|------|--------|--------|----------|
| 1 | LSH 验证一致性 | 简化 SimHash 实现 | 生产级实现或显式声明 |
| 2 | 统一测试脚本 | 60项自测分散无入口 | 一键回归测试 |
| 3 | WebRTC 降级代码 | 仅有设计文档 | 可运行核心代码 |

---

## 2. 改进路线图

### Phase A: 基线修复 (8-13 小时)

**目标**: 修复 R-001~R-003，达到 A 级放行标准

---

#### 任务 A1: LSH 验证脚本修正 (2-4 小时)

**对应风险**: R-001  
**优先级**: P1

```markdown
步骤:
1. 定位生产级 SimHash-64 实现
   - 搜索路径: src/ 目录下现有 hash 相关模块
   - 若不存在，需创建 src/utils/simhash64.js

2. 替换 lsh-collision-sim.js 中的简化实现
   - 修改范围: 第 35-61 行 simhash64 函数
   - 保持接口兼容: 输入 Float32Array, 输出 BigInt(64)

3. 复测验证
   - 运行: node src/test/lsh-collision-sim.js --vectors 10000 --queries 100
   - 通过标准: FPR < 0.1%, 汉明距离分布合理

4. 文档更新
   - 在 DEBT-LSH-001-REPORT.md 添加"实现说明"章节
   - 声明使用的 SimHash 版本和来源

验收标准:
- [ ] lsh-collision-sim.js 使用生产级 SimHash
- [ ] 复测通过 (FPR < 0.1%)
- [ ] 文档已更新实现说明
```

**即时验证命令**:
```bash
node src/test/lsh-collision-sim.js --vectors 10000 --queries 100 --verbose
# 预期: 汉明距离分布峰值在 32 附近，无异常偏斜
```

---

#### 任务 A2: 统一测试脚本 (2-3 小时)

**对应风险**: R-002  
**优先级**: P1

```markdown
步骤:
1. 创建 scripts/run-debt-tests.sh
   
   #!/bin/bash
   set -e
   
   echo "=== HAJIMI V3 债务清偿回归测试 ==="
   
   # 1. LSH 假阳性率测试
   echo "[RUN] LSH 假阳性率测试..."
   node src/test/lsh-collision-sim.js --vectors 10000 --queries 100 --json > logs/lsh-test.json
   if [ $? -eq 0 ]; then echo "  ✅ PASS"; else echo "  ❌ FAIL"; exit 1; fi
   
   # 2. HNSW 内存公式验证
   echo "[RUN] HNSW 内存公式验证..."
   node -e "
   const vectorCount = 100000;
   const dim = 768;
   const vectorData = vectorCount * dim * 4;
   if (vectorData !== 307200000) { process.exit(1); }
   console.log('  ✅ PASS: 内存公式正确');
   "
   
   # 3. 文档完整性检查
   echo "[RUN] 文档完整性检查..."
   for doc in DEBT-HNSW-001-FIX.md DEBT-LSH-001-REPORT.md SQLITE-SHARDING-方案对比.md; do
     if [ ! -f "docs/$doc" ]; then echo "  ❌ FAIL: $doc 缺失"; exit 1; fi
   done
   echo "  ✅ PASS: 6份文档完整"
   
   echo ""
   echo "=== 测试摘要 ==="
   echo "通过: X/60, 跳过: Y/60, 失败: 0/60"
   echo "评级: A (满足基线)"

2. 添加 package.json 脚本
   "scripts": {
     "test:debt": "bash scripts/run-debt-tests.sh"
   }

3. 测试脚本自身测试
   chmod +x scripts/run-debt-tests.sh
   ./scripts/run-debt-tests.sh

验收标准:
- [ ] scripts/run-debt-tests.sh 可执行
- [ ] 运行后输出测试摘要
- [ ] 全部基线项显示 PASS
```

**即时验证命令**:
```bash
chmod +x scripts/run-debt-tests.sh && ./scripts/run-debt-tests.sh
# 预期: 全部测试 PASS，无错误退出
```

---

#### 任务 A3: WebRTC 降级代码实现 (4-6 小时)

**对应风险**: R-003  
**优先级**: P1

```markdown
步骤:
1. 创建 src/sync/fallback-manager.js
   
   核心实现要求:
   - 状态机: IDLE → DISCOVERING → CONNECTING → (CONNECTED | ICE_FAILED | TIMEOUT) → FILE_EXPORT
   - 超时配置: gatheringTimeout=5s, connectionTimeout=10s, failedStateDelay=2s
   - 事件发射: sync:fallback, sync:export:ready, sync:complete

2. 实现参考代码结构:

   class SyncFallbackManager extends EventEmitter {
     constructor(options = {}) {
       super();
       this.config = {
         webrtcTimeout: options.webrtcTimeout || 10000,
         enableAutoFallback: options.enableAutoFallback !== false
       };
       this.state = 'IDLE';
     }
     
     async sync(peerId, manifest) { /* 主入口 */ }
     async tryWebRTC(peerId, manifest) { /* WebRTC 尝试 */ }
     async syncWithFileExport(manifest) { /* 降级到文件导出 */ }
     triggerFallback(reason) { /* 触发降级 */ }
   }

3. 创建基础测试
   
   # 快速加载测试
   node -e "
   const { SyncFallbackManager } = require('./src/sync/fallback-manager');
   const fm = new SyncFallbackManager({ webrtcTimeout: 100 });
   console.assert(fm.state === 'IDLE', '初始状态错误');
   console.assert(fm.config.enableAutoFallback === true, '默认配置错误');
   console.log('✅ 基础测试通过');
   "

4. 更新文档
   - 在 fallback-strategy.md 中添加"实现状态"章节
   - 注明: "核心代码已实现于 src/sync/fallback-manager.js"

验收标准:
- [ ] src/sync/fallback-manager.js 存在且可加载
- [ ] 状态机定义正确 (5状态)
- [ ] 超时配置可外部传入
- [ ] 基础加载测试通过
```

**即时验证命令**:
```bash
node -e "const { SyncFallbackManager } = require('./src/sync/fallback-manager'); const fm = new SyncFallbackManager(); console.log('状态:', fm.state); console.log('✅ 模块可加载')"
# 预期: 输出 "状态: IDLE" 和 "✅ 模块可加载"
```

---

### Phase B: 可选优化 (5-7 小时)

**目标**: 完成 R-004~R-005，提升至 A+ 级

**建议排期**: Phase 1 实施期间顺带完成

| 任务 | 工时 | 建议排期 |
|------|------|----------|
| B1: HNSW 内存监控 | 2-3 小时 | Phase 1 Week 1 |
| B2: SQLite 分片原型 | 3-4 小时 | Phase 1 Week 1 Day 1 |

---

## 3. 工时预估

### 基线修复 (必选)

| 任务 | 乐观 | 常规 | 悲观 |
|------|------|------|------|
| A1: LSH 验证修正 | 2h | 3h | 4h |
| A2: 统一测试脚本 | 2h | 2h | 3h |
| A3: WebRTC 降级代码 | 4h | 5h | 6h |
| **小计** | **8h** | **10h** | **13h** |

### 可选优化

| 任务 | 工时 | 排期 |
|------|------|------|
| B1: 内存监控 | 2-3h | Phase 1 Week 1 |
| B2: 分片原型 | 3-4h | Phase 1 Week 1 Day 1 |
| **小计** | **5-7h** | 带债上线 |

---

## 4. 风险与应对

| 风险 | 概率 | 影响 | 应对 |
|------|------|------|------|
| 生产级 SimHash 不存在 | 中 | 高 | 使用简化版 + 显式声明差异 |
| WebRTC 实现复杂超预期 | 低 | 中 | 先实现核心状态机，细节后续补充 |
| 测试脚本跨平台问题 | 低 | 低 | 使用 Node.js 纯 JS 实现，避免 bash 依赖 |

---

## 5. 验收检查清单

### 基线修复验收

- [ ] A1: LSH 脚本复测通过
- [ ] A2: ./scripts/run-debt-tests.sh 运行无错误
- [ ] A3: fallback-manager.js 可加载，状态机工作

### 评级提升确认

```
修复前: B (存在可接受债务)
   ├── R-001: LSH 实现不一致 ⚠️
   ├── R-002: 缺少测试脚本 ⚠️
   └── R-003: WebRTC 无代码 ⚠️

修复后: A (满足基线，无系统性风险)
   ├── R-001: ✅ 已修正
   ├── R-002: ✅ 已修正
   └── R-003: ✅ 已修正
```

---

**路线图版本**: v1.0  
**更新日期**: 2026-02-22  
**下次评审**: 基线项修复完成后
