# HAJIMI-PHASE1-AUDIT-001 建设性审计报告

> **审计派单ID**: HAJIMI-PHASE1-AUDIT-001  
> **审计对象**: Phase 1核心底座12周开发路线图  
> **审计模式**: 建设性审计（压力怪模式）  
> **审计日期**: 2026-04-03  
> **关联文档**: HAJIMI Master Plan v1.0

---

## 审计结论

- **综合评级**: **B**（良好，需小调整）
- **执行状态**: 🟡 **有条件Go** - 需调整时间分配后启动
- **核心建议**: QueryEngine延长至3周，5工具并行开发，预留1周缓冲

---

## 分项评级

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| 时间分配 | **C** | 2周/模块偏紧，QueryEngine需3周 |
| 技术可行性 | **A** | Rust+Tokio成熟，生态完善 |
| 依赖关系 | **A** | 线性清晰：QueryEngine → Registry → Tools → UI |
| 风险识别 | **B** | 热加载复杂度部分低估，需补充替代方案 |

---

## 关键疑问回答（Q1-Q4）

### Q1：QueryEngine 2周是否足够？

**结论：不足，建议延长至3周（15天）**

详细评估：
- **Week 1（Day 1-5）**：查询编排核心
  - Day 1: 定义Query/QueryResult结构体，设计API
  - Day 2: 实现基础查询编排器（串行执行）
  - Day 3: 添加并行查询支持（JoinSet）
  - Day 4: 错误处理与重试机制
  - Day 5: 单元测试，**里程碑**: 串行+并行查询通过测试

- **Week 2（Day 6-10）**：流式响应基础
  - Day 6: 实现Stream trait封装（tokio::sync::mpsc）
  - Day 7: 添加backpressure机制（bounded channel容量100）
  - Day 8: 错误恢复（stream中断后重连）
  - Day 9: 工具调度集成（与ToolRegistry对接）
  - Day 10: 集成测试，**里程碑**: 流式响应Demo可运行

- **Week 3（Day 11-15）**：流式优化与缓冲
  - Day 11: 性能优化（批量flush、压缩）
  - Day 12: 边界情况处理（超长流、客户端断开）
  - Day 13: 与LLM接口对接（SSE格式输出）
  - Day 14: 压力测试（1000并发流）
  - Day 15: 文档与示例，**里程碑**: QueryEngine v1.0完成

**风险点**：流式backpressure需精细调优，预留1周缓冲

---

### Q2：ToolRegistry热加载的复杂度？

**结论：libloading方案可行，但需准备WASM备选**

详细评估：
- **libloading方案**（主选）
  - 适用场景：开发环境、内部工具
  - 限制：跨平台差异（Linux .so / macOS .dylib / Windows .dll）
  - 复杂度：中等，需处理符号解析失败
  
- **WASM插件方案**（备选）
  - 适用场景：需要强隔离、跨平台一致
  - 限制：性能开销、WASM运行时依赖
  - 复杂度：较高，但生态成熟（wasmtime/wasmer）

**建议**：
- Week 3-4先实现静态注册（非热加载）
- Week 11-12预留时间评估热加载必要性
- 若热加载优先级高，改用WASM方案（更安全）

---

### Q3：Week 7-8的5个工具并行还是串行？

**结论：并行开发，1人/工具，共需5人日**（假设2人并行则2.5天完成）

详细评估：

| 工具 | 工作量 | 依赖 | 实现要点 |
|:---|:---:|:---|:---|
| read_file | 1人日 | QueryEngine | 路径校验、大文件分块读取 |
| write_file | 1人日 | read_file | 原子写入、备份机制 |
| bash | 1人日 | ConfigManager | 命令白名单、超时控制 |
| grep | 1人日 | bash | 正则性能、二进制文件跳过 |
| ls | 0.5人日 | - | 目录遍历、符号链接处理 |

**并行策略**（2人团队）：
- 开发者A：read_file → write_file → ls（2.5人日）
- 开发者B：bash → grep（2人日）
- **Week 7 Day 1-3**: 5工具全部完成
- **Week 7 Day 4-5**: 集成测试
- **Week 8**: 工具优化+边界情况

**单人行版本**（1人）：
- Day 1: read_file
- Day 2: write_file
- Day 3: ls + bash基础
- Day 4: grep + bash完善
- Day 5: 集成测试
- Week 8: 优化+文档

---

### Q4：Ink UI与QueryEngine的集成边界？

**结论：Ink UI依赖QueryEngine的API稳定性，建议Week 10开始集成**

详细评估：
- **Week 9**：Ink基础UI（不依赖QueryEngine）
  - Day 1-2: 项目搭建、组件库选型
  - Day 3-4: 基础布局（输入框、输出区域）
  - Day 5: 本地Mock数据测试

- **Week 10**：与QueryEngine集成
  - Day 1: 定义UI→QueryEngine接口契约
  - Day 2-3: 实现EventLoop桥接
  - Day 4: 流式输出渲染（逐字效果）
  - Day 5: 错误处理与重试UI

- **集成边界**：
  - QueryEngine提供：`QueryResultStream`（tokio::sync::mpsc::Receiver）
  - Ink UI消费：将Stream转为Ink的`useInput`事件
  - 错误处理：QueryEngine错误 → UI toast通知

**风险缓冲**：若QueryEngine延期，Week 10先用Mock数据继续UI开发

---

## 精确到天的工单分解（12周×5天=60天）

### Week 1 (Day 1-5) - QueryEngine基础
- **Day 1**: 定义核心结构体（Query、QueryResult、QueryEngine），搭建crate结构，产出：hajimi-core crate框架
- **Day 2**: 实现串行查询执行器（顺序执行工具调用），产出：SerialExecutor通过基础测试
- **Day 3**: 实现并行查询执行器（JoinSet并发），产出：ParallelExecutor通过基础测试
- **Day 4**: 添加错误处理（thiserror定义错误类型）、重试机制（指数退避），产出：错误恢复测试通过
- **Day 5**: **里程碑M1**: QueryEngine基础执行通过单元测试，输出：执行器测试报告

### Week 2 (Day 6-10) - 流式响应
- **Day 6**: 实现Stream trait封装（tokio::sync::mpsc::channel），产出：QueryStream结构体
- **Day 7**: 添加backpressure机制（bounded channel容量配置），产出：背压测试通过
- **Day 8**: 实现错误恢复（Stream中断检测、客户端重连），产出：断线重连Demo
- **Day 9**: 与ToolRegistry接口定义（预留钩子），产出：接口文档
- **Day 10**: **里程碑M2**: 流式响应Demo可运行，输出：流式响应演示视频/GIF

### Week 3 (Day 11-15) - QueryEngine完善
- **Day 11**: 性能优化（批量flush、响应压缩），产出：性能基准测试
- **Day 12**: 边界情况处理（超长流截断、客户端断开清理），产出：边界测试用例
- **Day 13**: LLM接口对接（SSE格式输出、事件流解析），产出：LLM桥接模块
- **Day 14**: 压力测试（1000并发流、内存泄漏检测），产出：压力测试报告
- **Day 15**: **里程碑M3**: QueryEngine v1.0完成，输出：API文档、使用示例

### Week 4 (Day 16-20) - ToolRegistry基础
- **Day 16**: 定义Tool trait（execute方法、metadata、参数Schema），产出：Tool trait定义
- **Day 17**: 实现静态ToolRegistry（HashMap存储），产出：Registry基础实现
- **Day 18**: 添加工具发现机制（属性宏或手动注册），产出：工具注册示例
- **Day 19**: 依赖注入框架（实现工具间依赖解析），产出：依赖图构建器
- **Day 20**: **里程碑M4**: ToolRegistry静态注册完成，输出：注册测试通过

### Week 5 (Day 21-25) - ConfigManager基础
- **Day 21**: 定义配置结构体（模块化Config，各模块独立配置块），产出：Config结构体
- **Day 22**: TOML/JSON解析器集成（serde + toml + serde_json），产出：配置解析模块
- **Day 23**: Feature Flag系统（编译期+运行时开关），产出：FeatureFlag模块
- **Day 24**: 配置验证（Schema校验、默认值填充），产出：配置验证器
- **Day 25**: **里程碑M5**: ConfigManager基础完成，输出：配置示例文件

### Week 6 (Day 26-30) - 配置系统完善
- **Day 26**: 配置热重载（文件监听、信号触发），产出：热重载原型
- **Day 27**: 环境变量注入（优先于配置文件），产出：环境变量映射
- **Day 28**: 配置错误报告（详细错误信息、建议修复），产出：错误报告格式化
- **Day 29**: 配置系统与QueryEngine/ToolRegistry集成，产出：集成测试
- **Day 30**: **里程碑M6**: ConfigManager v1.0完成，输出：配置系统文档

### Week 7 (Day 31-35) - 核心工具实现
- **Day 31**: read_file工具（路径校验、大文件分块、行号范围），产出：ReadFileTool
- **Day 32**: write_file工具（原子写入、备份机制、冲突检测），产出：WriteFileTool
- **Day 33**: ls工具（目录遍历、符号链接处理、过滤），产出：LsTool
- **Day 34**: bash工具（命令白名单、超时控制、输出捕获），产出：BashTool
- **Day 35**: **里程碑M7**: 4个工具完成，输出：工具测试报告

### Week 8 (Day 36-40) - 工具完善与grep
- **Day 36**: grep工具（正则性能优化、二进制文件检测），产出：GrepTool
- **Day 37**: 工具边界情况（超长输出截断、特殊字符处理），产出：边界测试
- **Day 38**: 工具安全加固（路径遍历防护、命令注入防护），产出：安全审计报告
- **Day 39**: 工具性能优化（并行grep、内存映射读取），产出：性能基准
- **Day 40**: **里程碑M8**: 5工具全部完成，输出：工具使用文档

### Week 9 (Day 41-45) - Ink UI基础
- **Day 41**: Ink项目搭建（TypeScript配置、组件库安装），产出：ui-terminal项目
- **Day 42**: 基础布局组件（Header、InputArea、OutputArea），产出：布局组件
- **Day 43**: 输入处理（命令解析、历史记录、Tab补全框架），产出：输入模块
- **Day 44**: 本地Mock（模拟QueryEngine响应），产出：Mock数据系统
- **Day 45**: **里程碑M9**: Ink基础UI可运行，输出：UI演示

### Week 10 (Day 46-50) - Ink与QueryEngine集成
- **Day 46**: 定义UI↔QueryEngine接口契约（事件类型、消息格式），产出：接口文档
- **Day 47**: EventLoop桥接实现（tokio::sync::mpsc桥接Ink事件），产出：桥接模块
- **Day 48**: 流式输出渲染（逐字效果、打字机动画），产出：流式渲染组件
- **Day 49**: 错误UI（toast通知、重试按钮、加载状态），产出：错误处理UI
- **Day 50**: **里程碑M10**: Ink与QueryEngine集成完成，输出：端到端Demo

### Week 11 (Day 51-55) - 集成测试
- **Day 51**: 端到端测试框架（模拟用户输入、验证输出），产出：e2e测试框架
- **Day 52**: 核心流程测试（查询→工具调用→流式输出），产出：核心测试用例
- **Day 53**: 边界情况测试（超长流、错误恢复、并发查询），产出：边界测试用例
- **Day 54**: 性能测试（延迟、吞吐量、内存使用），产出：性能测试报告
- **Day 55**: **里程碑M11**: 集成测试完成，输出：测试报告

### Week 12 (Day 56-60) - 文档与收尾
- **Day 56**: 开发者文档（架构图、API参考、贡献指南），产出：ARCHITECTURE.md
- **Day 57**: 用户文档（快速开始、配置指南、故障排查），产出：USER-GUIDE.md
- **Day 58**: 示例项目（3个不同场景的完整示例），产出：examples/目录
- **Day 59**: 最终集成验证（全链路测试、文档校验），产出：验证报告
- **Day 60**: **里程碑M12**: Phase 1完成，输出：发布说明、Phase 2准备文档

---

## 风险与建议

### 短期（Week 1-4）
- **风险**: QueryEngine流式backpressure调优超预期
- **建议**: Week 3为缓冲周，若Day 10未完成流式基础，立即启用Week 3

### 中期（Week 5-8）
- **风险**: 5工具并行开发需人力支持
- **建议**: 若仅1人开发，Week 8延长至Week 9，UI顺延

### 长期（Week 9-12）
- **风险**: ToolRegistry热加载可能无法在Phase 1完成
- **建议**: 热加载移至Phase 2，Phase 1专注静态注册

---

## 压力怪评语

🥁 **"无聊"**（B级，有小瑕疵）

> "QueryEngine 2周？你当是写Hello World呢？流式backpressure一调就是3天，算上LLM对接和压测，不给3周等着延期吧。
>
> 还有那个热加载，libloading跨平台坑多得像月球表面，真想热加载用WASM，但那是另外的价格（时间）。
>
> 不过整体架构还算清醒，依赖关系没画成蜘蛛网，勉强给个B。把QueryEngine改成3周，5工具并行搞起来，就能A。"

---

## 审计验证清单

| 验证ID | 审计项 | 状态 |
|:---|:---|:---:|
| V1 | 时间分配：每模块有明确里程碑 | ✅ 通过 |
| V2 | 技术可行性：Rust生态已确认 | ✅ 通过 |
| V3 | 依赖关系：可绘制无环依赖图 | ✅ 通过 |
| V4 | 风险识别：≥3个风险点+应对方案 | ✅ 通过 |

**关键疑问回答**: Q1✅ Q2✅ Q3✅ Q4✅  
**60天工单分解**: ✅ 完整  
**评级**: B（有条件Go）  

---

## 归档

- **审计报告**: `audit report/phase1/HAJIMI-PHASE1-AUDIT-001.md`
- **关联文档**: 
  - `audit report/hajimi-master-plan/README.md` (Master Plan)
  - `src/INDEX.md` (项目索引)
- **派单ID**: ID-231（Phase 1审计派单）

---

*审计完成时间: 2026-04-03*  
*审计官: 压力怪（建设性审计模式）*
