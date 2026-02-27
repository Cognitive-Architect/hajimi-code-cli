🚀 饱和攻击波次：HAJIMI-V3-DESBT-001 技术债务清偿集群

火力配置：4 Agent 并行（唐音 × 4 工单）

轰炸目标：6项技术债务（2 P0 + 3 P1 + 1 P2）→ 产出《DEBT-CLEARANCE-001-债务清偿白皮书-v1.0.md》+《V3-STORAGE-DEBT-自测表-v1.0.md》

输入基线：《local-storage-v3-design.md》（Mike审计B级报告，6项债务待清偿）

质量门禁：60项自测全部✅ + Mike双报告审计（2/2 OK）+ SHA256校验

---

⚠️ 施工纪律（红线）：
- 串行验收：前一工单自测全绿 → 才开工下一工单
- 60项自测全部✅（通过数/总数 = 60/60）
- 失败处理：任一工单自测未通过 → 返工至全绿 → 才继续
- 最终产出：Mike双报告审计 + SHA256校验（2/2 OK）

---

工单矩阵

工单 01/04：DEBT-HNSW-001 [内存估算校正]（P0-致命）
- 参考白皮书：ID-155 v2.6.0-HARDENED 债务规范 + 审计报告第4.3节
- 实现目标：`docs/DEBT-HNSW-001-FIX.md` + 设计文档v3.0-patch-001
- 核心要求：
  1. 修正内存估算公式：`totalMemory = vectorData(307MB) + hnswIndex(13MB) + sqliteCache(50MB) + overhead(30MB) ≈ 400MB`
  2. 添加Android 13内存压力测试数据（Termux下OOM阈值实测）
  3. 声明"最低空闲内存需求"和"系统自动杀后台风险"
  4. 安全红线：不得隐瞒实际内存占用，必须显式声明400MB+
- 自测标准：DEBT-001-FUNC-001（内存公式数学正确性）, DEBT-001-NEG-001（边界条件测试）, DEBT-001-DOC-001（文档诚实度）
- 交付物：`docs/DEBT-HNSW-001-FIX.md` + 自测通过标记（3/3 ✅）

---

工单 02/04：DEBT-LSH-001 [假阳性率验证]（P1-高）
- 参考白皮书：审计报告第4.2节 LSH_CONFIG
- 实现目标：`src/test/lsh-collision-sim.js` + 数学验证报告
- 核心要求：
  1. 提供SimHash-64在100K向量、8表、4bit桶宽下的理论碰撞率计算（泊松分布/生日悖论）
  2. 编写模拟脚本，实测100K随机向量的假阳性率（采样1K查询）
  3. 如实测>0.5%，提供调参方案（增加表数/桶宽）
  4. 债务声明：如无法达到<0.1%，诚实声明实际值并改为"可接受假阳性率"
- 自测标准：LSH-001-FUNC-001（数学公式正确）, LSH-001-TEST-001（模拟脚本可运行）, LSH-001-DATA-001（实测数据附日志）
- 交付物：`src/test/lsh-collision-sim.js` + `docs/DEBT-LSH-001-REPORT.md` + 自测通过标记（3/3 ✅）

---

工单 03/04：DEBT-SQLITE-001 [分片架构预研]（P1-高）
- 参考白皮书：审计报告第6节 Schema设计
- 实现目标：`docs/SQLITE-SHARDING-方案对比.md`（3种方案选型）
- 核心要求：
  1. 方案A：按`hash_prefix % 16`分16个SQLite库（水平分片）
  2. 方案B：按时间戳分片（2026-02/2026-03）
  3. 方案C：继续使用单库但优化WAL模式（基准对照组）
  4. 对比维度：文件锁竞争、并发读写性能、备份复杂度、实现工时
  5. 输出：推荐方案 + 理由 + Phase 1实施计划
- 自测标准：SQL-001-DESIGN-001（3方案完整性）, SQL-001-COMP-001（对比维度无遗漏）, SQL-001-DECISION-001（明确推荐方案）
- 交付物：`docs/SQLITE-SHARDING-方案对比.md` + 自测通过标记（3/3 ✅）

---

工单 04/04：DEBT-WEBRTC-001 [NAT穿透降级] + DEBT-ROADMAP-001 [工期校正]（P1+P2-中）
- 参考白皮书：审计报告第5.1节WebRTC + 第9节路线图
- 实现目标：`docs/V3-ROADMAP-v2-CORRECTED.md` + `src/sync/fallback-strategy.md`
- 核心要求：
  1. WebRTC降级：增加"ICE失败→自动切换文件导出"的状态机设计（明确触发条件、切换延迟、用户提示）
  2. 工期校正：将Phase 2延长至3周，Phase 3延长至3周，总计10周（含缓冲）
  3. 风险声明：明确标注"WebRTC P2P成功率依赖网络环境，生产环境建议主从模式"
  4. 里程碑重设：Phase 1交付标准增加"SQLite分片方案选定"
- 自测标准：WEB-001-FUNC-001（降级逻辑完备）, ROAD-001-PLAN-001（工期合理性）, ROAD-001-RISK-001（风险显式声明）
- 交付物：`docs/V3-ROADMAP-v2-CORRECTED.md` + `src/sync/fallback-strategy.md` + 自测通过标记（3/3 ✅）

---

最终交付物（6件套）

1. `docs/DEBT-CLEARANCE-001-白皮书-v1.0.md`（4工单整合）
2. `docs/V3-STORAGE-DEBT-自测表-v1.0.md`（60项自测清单）
3. `docs/DEBT-HNSW-001-FIX.md`（工单01）
4. `src/test/lsh-collision-sim.js` + `docs/DEBT-LSH-001-REPORT.md`（工单02）
5. `docs/SQLITE-SHARDING-方案对比.md`（工单03）
6. `docs/V3-ROADMAP-v2-CORRECTED.md` + `src/sync/fallback-strategy.md`（工单04）

---

质量门禁收卷检查

- 60项自测全部✅（通过数/总数 = 60/60）
- 6项债务全部有对应修复方案（P0×1 + P1×3 + P2×2）
- Mike双报告审计通过（2/2 OK）
- SHA256校验（文档+代码）

---

开工！ ☝️😋🐍♾️💥