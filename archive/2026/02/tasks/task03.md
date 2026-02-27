🚀 饱和攻击波次：HAJIMI-AUDIT-FIX-001 基线项硬钢集群
火力配置：3 Agent 并行（唐音 ×3）

轰炸目标：3项基线债务（R-001/R-002/R-003）→ A级评级

输入基线：AUDIT-DEBT-CLEARANCE-001-REPORT.md + HAJIMI-V3-IMPROVEMENT-ROADMAP.md

收卷标准：3工单全部A级 + 60项自测全绿（新增27项）+ 6件套归档

---

⚠️ 质量门禁（必须全部满足才能开工）

```markdown
- [ ] 《P4自测轻量检查表》10/10项已勾选（每工单）
- [ ] 工时预算确认：R-001(4h) + R-002(3h) + R-003(6h) = 13h总计
- [ ] 债务预声明：修复过程中发现的新债务必须DEBT-XXX分级
- [ ] 验证命令可复制：每交付物附带即时验证命令（bash/node）
```

---

📋 工单矩阵（3路并行）

工单 B-01/03 唐音 → R-001（LSH验证一致性修复）
目标：修正 `lsh-collision-sim.js` 使用生产级SimHash-64，消除验证与生产实现差异

输入：
- `src/test/lsh-collision-sim.js`（第35-61行简化实现）
- `docs/DEBT-LSH-001-REPORT.md`（验证报告）
- 审计报告R-001节（修复路径）

输出：
- `src/test/lsh-collision-sim-v2.js`（或覆盖原文件）
- `src/utils/simhash64.js`（如生产级实现不存在则新建）
- `docs/DEBT-LSH-001-FIXED.md`（修复声明，含实现说明章节）

自测点：
- LSH-FIX-001：生产级SimHash汉明距离分布测试（峰值在32附近）
- LSH-FIX-002：FPR复测<0.1%（10000向量×100查询）
- LSH-FIX-003：与简化版实现差异显式声明（文档）
- LSH-FIX-004：Node.js加载测试（无报错）
- LSH-FIX-005：CLI参数兼容性（--vectors/--queries/--verbose）

P4自测轻量检查表（本工单）：

检查点	覆盖情况	相关用例ID	
CF（核心功能）	[ ]	LSH-FIX-001,002	
RG（约束回归）	[ ]	LSH-FIX-003	
NG（负面路径）	[ ]	LSH-FIX-004（加载失败场景）	
UX（用户体验）	[ ]	LSH-FIX-005（CLI可用性）	
E2E（端到端）	[ ]	LSH-FIX-002（全链路）	
High（高风险）	[ ]	LSH-FIX-001（正确性）	
字段完整性	[ ]	全部5项	
需求映射	[ ]	R-001	
执行结果	[ ]	待测试	
范围边界	[ ]	仅修复SimHash实现，不碰LSH逻辑	

即时可验证方法：

```bash
node src/test/lsh-collision-sim.js --vectors 10000 --queries 100 --verbose
# 预期：汉明距离分布峰值在32附近，FPR<0.1%
```

---

工单 B-02/03 唐音 → R-002（统一测试脚本）
目标：创建 `scripts/run-debt-tests.sh` 统一运行60项自测，实现一键回归测试

输入：
- `docs/V3-STORAGE-DEBT-自测表-v1.0.md`（60项自测清单）
- 4份债务修复文档（HNSW/LSH/SQLITE/WEBRTC）
- 审计报告R-002节（修复路径）

输出：
- `scripts/run-debt-tests.sh`（可执行bash脚本）
- `package.json`（新增test:debt脚本）
- `logs/debt-test-results.json`（JSON格式测试结果）
- `docs/DEBT-TEST-UNIFIED.md`（脚本使用文档）

自测点：
- TEST-UNI-001：脚本可执行（chmod +x后运行无报错）
- TEST-UNI-002：LSH测试子集通过（调用lsh-collision-sim.js）
- TEST-UNI-003：HNSW内存公式验证通过（node计算断言）
- TEST-UNI-004：文档完整性检查（6份文档存在性）
- TEST-UNI-005：跨平台兼容（Termux/Node.js环境）
- TEST-UNI-006：错误处理（单测试失败时整体退出码1）
- TEST-UNI-007：JSON报告生成（logs/debt-test-results.json）
- TEST-UNI-008：测试摘要输出（通过/跳过/失败统计）

P4自测轻量检查表（本工单）：

检查点	覆盖情况	相关用例ID	
CF	[ ]	TEST-UNI-001,002,003	
RG	[ ]	TEST-UNI-004（文档完整性约束）	
NG	[ ]	TEST-UNI-006（失败场景）	
UX	[ ]	TEST-UNI-008（摘要可读性）	
E2E	[ ]	TEST-UNI-005（跨平台端到端）	
High	[ ]	TEST-UNI-002（LSH正确性）	
字段完整性	[ ]	全部8项	
需求映射	[ ]	R-002	
执行结果	[ ]	待测试	
范围边界	[ ]	仅整合现有测试，不新增功能测试	

即时可验证方法：

```bash
chmod +x scripts/run-debt-tests.sh && ./scripts/run-debt-tests.sh
# 预期：全部测试PASS，输出摘要，exit 0
```

---

工单 B-03/03 唐音 → R-003（WebRTC降级代码实现）
目标：实现 `src/sync/fallback-manager.js` 核心类，含状态机和降级逻辑

输入：
- `src/sync/fallback-strategy.md`（设计文档）
- 审计报告R-003节（修复路径）
- HAJIMI-V3-IMPROVEMENT-ROADMAP.md（任务A3）

输出：
- `src/sync/fallback-manager.js`（核心实现，含SyncFallbackManager类）
- `src/test/fallback-manager.test.js`（基础单元测试）
- `docs/DEBT-WEBRTC-IMPLEMENTED.md`（实现状态更新）

功能规格（必须实现）：

```javascript
// 状态机：IDLE → DISCOVERING → CONNECTING → (CONNECTED | ICE_FAILED | TIMEOUT) → FILE_EXPORT
// 超时配置：gatheringTimeout=5s, connectionTimeout=10s, failedStateDelay=2s
// 事件：sync:fallback, sync:export:ready, sync:complete
```

自测点：
- FB-001：类可实例化（new SyncFallbackManager()不报错）
- FB-002：初始状态IDLE（fm.state === 'IDLE'）
- FB-003：配置可外部传入（webrtcTimeout覆盖默认）
- FB-004：状态机定义完整（5状态常量定义）
- FB-005：降级触发逻辑（ICE_FAILED自动切换FILE_EXPORT）
- FB-006：超时机制（10s无连接自动降级）
- FB-007：事件发射（fallback事件可监听）
- FB-008：错误处理（无效peerId不崩溃）

P4自测轻量检查表（本工单）：

检查点	覆盖情况	相关用例ID	
CF	[ ]	FB-001005（核心降级功能）	
RG	[ ]	FB-002（状态回归）	
NG	[ ]	FB-008（无效输入处理）	
UX	[ ]	FB-007（事件通知）	
E2E	[ ]	FB-005,006（完整降级链路）	
High	[ ]	FB-005,006（降级可靠性）	
字段完整性	[ ]	全部8项	
需求映射	[ ]	R-003	
执行结果	[ ]	待测试	
范围边界	[ ]	仅核心状态机，不实现完整WebRTC传输	

即时可验证方法：

```bash
node -e "
const { SyncFallbackManager } = require('./src/sync/fallback-manager');
const fm = new SyncFallbackManager({ webrtcTimeout: 100 });
console.assert(fm.state === 'IDLE', '初始状态错误');
console.log('✅ 基础测试通过');
"
# 预期：无报错，输出"✅ 基础测试通过"
```

---

🎯 集群收卷强制交付物（总计6件套）

#	交付物	来源工单	类型	
1	`src/test/lsh-collision-sim-v2.js` + `src/utils/simhash64.js`	B-01	代码	
2	`docs/DEBT-LSH-001-FIXED.md`	B-01	文档	
3	`scripts/run-debt-tests.sh` + `package.json`更新	B-02	脚本/配置	
4	`docs/DEBT-TEST-UNIFIED.md`	B-02	文档	
5	`src/sync/fallback-manager.js` + `src/test/fallback-manager.test.js`	B-03	代码/测试	
6	`docs/DEBT-WEBRTC-IMPLEMENTED.md`	B-03	文档