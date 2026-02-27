🚀 饱和攻击波次：HAJIMI-WASM-COMPILE-001 WASM债务最终清偿
火力配置：1 Agent（WASM编译工程师）
轰炸目标：DEBT-PHASE2-001（WASM方案）→ 从"框架完成"转为"完全清偿"，产出WASM编译产物+5x加速比验证报告
输入基线：ID-174（Phase 4 A级归档态）+ wasm-pack 0.14.0已安装 + `crates/hajimi-hnsw/src/lib.rs`（193行Rust代码）

---

质量门禁（必须全部满足才能开工）

检查项	验证命令	通过标准	状态	
wasm-pack已安装	`wasm-pack --version`	显示0.14.0	☐	
Rust代码存在	`ls crates/hajimi-hnsw/src/lib.rs`	文件存在	☐	
WASM目标已安装	`ls $PREFIX/lib/rustlib/wasm32-unknown-unknown`	目录存在	☐	
磁盘空间充足	`df -h ~`	可用>500MB	☐	
P4自测表已读	`cat docs/task09-phase4-wasm-worker-robust/HAJIMI-PHASE4-自测表-v1.0.md`	已了解WASM-COMP测试项	☐	

未满足质量门禁→禁止开工→返回补充环境

---

工单矩阵（单Agent深度任务）

工单 C-01/01 WASM编译工程师 → DEBT-PHASE2-001最终清偿
目标：执行`wasm-pack build`，生成WASM产物，验证5x加速比，更新债务状态为"已清偿"
输入：
- `crates/hajimi-hnsw/Cargo.toml`（Rust配置）
- `crates/hajimi-hnsw/src/lib.rs`（193行HNSW核心代码）
- `tests/benchmark/wasm-vs-js.bench.js`（性能对比测试）
- `tests/benchmark/worker-blocking.bench.js`（Worker基准测试）
输出：
1. `crates/hajimi-hnsw/pkg/`目录（含`.wasm`文件+JS胶水代码+package.json）
2. `docs/task10-wasm-compile/HAJIMI-WASM-COMPILE-自测表-v1.0.md`（刀刃+P4）
3. `docs/task10-wasm-compile/HAJIMI-WASM-COMPILE-白皮书-v1.0.md`（编译过程+性能报告）
4. 债务更新：DEBT-PHASE2-001状态改为"✅已清偿"
自测点：
- WASM-COMP-001: wasm-pack编译成功
- WASM-COMP-002: pkg目录生成（含.wasm+.js）
- WASM-COMP-003: WASM文件大小<500KB
- WASM-COMP-004: 胶水代码加载正常
- WASM-COMP-005: 导出函数检查（insert/search）
- WASM-PERF-001: 查询加速比>5x（对比JS模式）
- WASM-PERF-002: 构建加速比>3x
- WASM-INT-001: HybridIndex自动切换到WASM模式
- WASM-NEG-001: WASM加载失败时降级到JS

---

刀刃风险自测表（Engineer必须全部手动勾选）

用例ID	类别	场景	验证命令（可复制）	通过标准	状态（Engineer填）	
WASM-COMP-001	FUNC	wasm-pack编译	`cd crates/hajimi-hnsw && wasm-pack build --target nodejs 2>&1`	Exit 0，无错误	[ ]	
WASM-COMP-002	FUNC	pkg目录生成	`ls crates/hajimi-hnsw/pkg/`	含.hajimi_hnsw_bg.wasm+.js+package.json	[ ]	
WASM-COMP-003	PERF	WASM文件大小	`ls -lh crates/hajimi-hnsw/pkg/*.wasm \| awk '{print $5}'`	<500KB	[ ]	
WASM-COMP-004	FUNC	胶水代码加载	`node -e "require('./crates/hajimi-hnsw/pkg/hajimi_hnsw.js')"`	Exit 0	[ ]	
WASM-COMP-005	FUNC	导出函数检查	`grep "export.*function\|module.exports" crates/hajimi-hnsw/pkg/hajimi_hnsw.js \| head -5`	看到insert/search等函数	[ ]	
WASM-PERF-001	PERF	查询加速比>5x	`node tests/benchmark/wasm-vs-js.bench.js 2>&1 \| grep "加速比\|speedup"`	显示>5x	[ ]	
WASM-PERF-002	PERF	构建加速比>3x	`node tests/benchmark/wasm-vs-js.bench.js 2>&1 \| grep "构建"`	显示>3x	[ ]	
WASM-INT-001	FUNC	Hybrid自动切换WASM	`node -e "const {HybridHNSWIndex}=require('./src/vector/hnsw-index-hybrid'); const i=new HybridHNSWIndex({dimension:128}); i.init().then(()=>console.log('Mode:',i.getMode()))"`	输出`Mode: wasm`（编译后）	[ ]	
WASM-INT-002	FUNC	无拷贝传递	检查`hnsw-index-hybrid.js`是否使用`new Uint8Array`直接传递	代码中无JSON序列化	[ ]	
WASM-NEG-001	NEG	WASM降级到JS	临时重命名pkg/目录，运行上述命令	输出`Mode: javascript`	[ ]	
WASM-DEBT-001	RG	债务状态更新	`grep "DEBT-PHASE2-001" docs/task10-wasm-compile/HAJIMI-WASM-COMPILE-白皮书-v1.0.md`	显示"✅已清偿"	[ ]	
WASM-E2E-001	E2E	完整WASM工作流	`node tests/e2e/phase4-integration.test.js 2>&1 \| grep "WASM"`	通过	[ ]	

---

P4自测轻量检查表（Engineer逐行勾选）

检查点	自检问题	覆盖情况（Engineer填）	
CF-001	WASM编译是否有≥1条CF用例覆盖？	[ ]	
CF-002	性能加速比是否有CF验证？	[ ]	
RG-001	债务清偿状态是否有RG用例验证？	[ ]	
NG-001	WASM加载失败降级是否有NG用例？	[ ]	
NG-002	文件缺失/损坏降级是否有NG用例？	[ ]	
UX-001	自动模式切换（WASM/JS）是否有UX场景？	[ ]	
E2E-001	端到端WASM工作流是否有E2E用例？	[ ]	
High-001	内存占用（WASM vs JS）是否有High风险用例？	[ ]	
字段完整性	所有用例是否填写完整（前置条件/验证命令/通过标准）？	[ ]	
范围边界	若WASM编译失败，是否标注"降级到JS"？	[ ]	

---

交付物清单（3件套强制）

1. WASM编译产物（强制）

```
crates/hajimi-hnsw/pkg/
├── hajimi_hnsw_bg.wasm        # WASM二进制（<500KB）
├── hajimi_hnsw_bg.wasm.d.ts   # TypeScript定义
├── hajimi_hnsw.js             # JS胶水代码
├── hajimi_hnsw.d.ts           # 类型定义
└── package.json               # NPM配置
```

2. HAJIMI-WASM-COMPILE-自测表-v1.0.md（强制）
- 包含上述12项刀刃自测（全部手动[x]）
- 包含上述10项P4检查（全部手动[x]）
- 债务更新：DEBT-PHASE2-001 ✅已清偿

3. HAJIMI-WASM-COMPILE-白皮书-v1.0.md（强制，4章结构）
- 第1章：编译过程 - 执行的命令、耗时、遇到的警告/错误及解决方案
- 第2章：性能验证 - wasm-vs-js.bench.js运行结果，加速比数据（目标>5x）
- 第3章：债务清偿 - DEBT-PHASE2-001从"🔄框架完成"改为"✅已清偿"的证据
- 第4章：已知限制 - Termux环境特定问题（如有）、生产环境建议

---

执行命令序列（参考）

```bash
# 1. 进入目录
cd ~/storage/downloads/A.Hajimi\ 算法研究院/workspace/crates/hajimi-hnsw

# 2. 执行编译（如内存不足加--jobs 1）
wasm-pack build --target nodejs

# 3. 验证产物
ls -lh pkg/

# 4. 运行性能测试
cd ../../
node tests/benchmark/wasm-vs-js.bench.js

# 5. 验证HybridIndex自动切换
node -e "const {HybridHNSWIndex}=require('./src/vector/hnsw-index-hybrid'); const i=new HybridHNSWIndex({dimension:128}); i.init().then(()=>console.log('Mode:',i.getMode()))"
```

---

D级红线（触发即永久返工）

1. WASM编译失败无记录 → 必须记录错误日志和尝试的解决方案
2. 未验证加速比 → 必须运行benchmark并记录结果（即使未达5x也要诚实记录）
3. 债务状态虚假更新 → 若WASM编译实际失败却标记"已清偿"→D级
4. 未提供编译产物 → pkg/目录必须存在且含.wasm文件
5. 自测表自动生成/预填 → 所有[x]必须Engineer手动勾选，发现复制粘贴→D级
6. 超过2小时未完成 → 超时必须提交进度报告，否则D级

---

验收标准（数值化）

验收项	验收命令	通过标准	失败标准	
WASM产物存在	`ls crates/hajimi-hnsw/pkg/*.wasm`	文件存在且>0B	文件不存在	
加速比验证	`node tests/benchmark/wasm-vs-js.bench.js`	有输出结果（无论是否达5x）	报错退出	
自动切换WASM	HybridHNSWIndex.init()	输出`Mode: wasm`	仍为`javascript`或报错	
降级机制仍可用	删除pkg/后测试	降级到`javascript`	崩溃或报错	
债务更新诚实	检查白皮书	DEBT-PHASE2-001状态与实际一致	声称清偿但未编译	
自测表完整	12项刀刃+10项P4	全部手动[x]	有⬜或预填[x]	

---

战术金句

"wasm-pack已就绪，193行Rust代码等待涅槃成WASM。