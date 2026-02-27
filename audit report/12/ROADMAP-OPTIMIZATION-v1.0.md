# ROADMAP-OPTIMIZATION-v1.0

> **项目**: HAJIMI V3 扩展优化路线图  
> **基于审计**: 12-AUDIT-EXPLORATION-建设性审计报告  
> **版本**: v1.0  
> **日期**: 2026-02-26

---

## 路线图概览

```
时间轴 ───────────────────────────────────────────────►
         │         │         │         │
        v2.0      v2.1      v2.2      v3.0
         │         │         │         │
    ┌────┴────┐   │    ┌────┴────┐   │
    │ 债务清偿 │   │    │ 可观测性 │   │
    │ P1/P2   │   │    │ 增强    │   │
    └────┬────┘   │    └────┬────┘   │
         │   ┌────┴────┐    │   ┌────┴────┐
         │   │ 安全加固 │    │   │ P2P同步 │
         │   │ &限流   │    │   │ 完整版  │
         │   └────┬────┘    │   └────┬────┘
         │        │   ┌────┴────┐    │
         │        │   │ 配置热  │    │
         │        │   │ 重载    │    │
         │        │   └────┬────┘    │
         │        │        │   ┌────┴────┐
         │        │        │   │ WASM    │
         │        │        │   │ 运行时  │
         │        │        │   │ 完善    │
         │        │        │   └─────────┘
```

---

## Phase 1: 债务清偿 (v2.0)

**目标**: 修复审计发现的P1/P2问题，提升系统健壮性

**周期**: 2周

### 任务清单

| ID | 任务 | 优先级 | 工时 | 交付物 |
|:---|:-----|:------:|:----:|:-------|
| 2.0-001 | 添加.gitignore | P2 | 10min | .gitignore文件 |
| 2.0-002 | CORS配置可配置化 | P2 | 30min | server.js更新 |
| 2.0-003 | 处理TODO标记 | P2 | 30min | 代码清理 |
| 2.0-004 | 添加请求ID追踪 | P2 | 1h | middleware/request-id.js |
| 2.0-005 | 配置校验增强 | P2 | 30min | server.js更新 |

### 验收标准
- [ ] .gitignore包含.env*, node_modules/, *.log
- [ ] CORS可通过构造函数参数配置
- [ ] 所有TODO已处理或转化为正式债务卡片
- [ ] 每个请求有唯一requestId，记录于日志

---

## Phase 2: 安全加固与限流 (v2.1)

**目标**: 增强API安全性，防止滥用

**周期**: 2周

### 任务清单

| ID | 任务 | 优先级 | 工时 | 交付物 |
|:---|:-----|:------:|:----:|:-------|
| 2.1-001 | 实现Token Bucket限流器 | P2 | 2h | middleware/rate-limiter.js |
| 2.1-002 | 添加API超时控制 | P2 | 1h | server.js + middleware |
| 2.1-003 | 错误日志增强 | P2 | 1h | 统一错误日志格式 |
| 2.1-004 | 添加安全响应头 | P2 | 30min | security-headers中间件 |

### 限流器设计

```javascript
// 目标API
class RateLimiter {
  constructor(options = {}) {
    this.maxRequests = options.maxRequests || 100;  // 每窗口请求数
    this.windowMs = options.windowMs || 60000;      // 窗口大小（1分钟）
    this.keyGenerator = options.keyGenerator || (req => req.ip);
  }
  
  async check(req) {
    const key = this.keyGenerator(req);
    const now = Date.now();
    // Token Bucket算法实现
    // 返回 { allowed: boolean, remaining: number, resetTime: number }
  }
}
```

### 验收标准
- [ ] 默认限流：100 req/min/IP
- [ ] 超限时返回429状态码
- [ ] 响应包含X-RateLimit-*头部
- [ ] API调用超时默认30秒

---

## Phase 3: 可观测性增强 (v2.2)

**目标**: 提升系统可观测性，支持生产环境监控

**周期**: 3周

### 任务清单

| ID | 任务 | 优先级 | 工时 | 交付物 |
|:---|:-----|:------:|:----:|:-------|
| 2.2-001 | Prometheus指标暴露 | P3 | 4h | /metrics端点 + 指标收集 |
| 2.2-002 | 健康检查增强 | P3 | 2h | /health/detailed端点 |
| 2.2-003 | 性能剖析支持 | P3 | 3h | CPU/内存剖析接口 |
| 2.2-004 | 配置热重载 | P3 | 6h | 配置文件监听+动态更新 |

### 指标设计

```
# 暴露的Prometheus指标
hajimi_requests_total{method, route, status}
hajimi_request_duration_seconds{method, route}
hajimi_vector_index_size	hajimi_vector_search_latency_seconds
hajimi_storage_chunks_total
hajimi_worker_pool_size
hajimi_worker_tasks_completed
hajimi_disk_free_bytes
hajimi_disk_emergency_mode
```

### 验收标准
- [ ] /metrics返回Prometheus格式指标
- [ ] 支持请求量、延迟、错误率统计
- [ ] 支持向量索引规模监控
- [ ] 配置文件修改后自动重载（无需重启）

---

## Phase 4: WASM运行时完善 (v3.0)

**目标**: 完善WASM支持，实现5x加速比

**周期**: 4周

### 前提条件
- wasm-bindgen-cli在Termux环境可用
- 或找到替代加载方案

### 任务清单

| ID | 任务 | 优先级 | 工时 | 交付物 |
|:---|:-----|:------:|:----:|:-------|
| 3.0-001 | WASM加载器完善 | P1 | 8h | src/wasm/loader.js重构 |
| 3.0-002 | WASM/JS自动切换优化 | P2 | 4h | hybrid-index增强 |
| 3.0-003 | WASM内存管理 | P2 | 6h | WASM内存池管理 |
| 3.0-004 | 性能基准测试 | P3 | 4h | 完整benchmark套件 |

### 技术方案

```javascript
// WASM加载策略
class WASMLoader {
  async load() {
    // 尝试1: 直接加载.wasm文件
    // 尝试2: 加载wasm-bindgen生成的.js包装器
    // 尝试3: 使用WebAssembly.instantiateStreaming
    // 降级: 返回null，使用JS实现
  }
}
```

### 验收标准
- [ ] Termux环境WASM可正常加载
- [ ] 向量搜索性能提升≥3x（目标5x）
- [ ] WASM加载失败自动降级到JS
- [ ] 内存使用稳定，无泄漏

---

## Phase 5: P2P同步完整实现 (v3.0+)

**目标**: 实现去中心化的数据同步

**周期**: 6周

### 任务清单

| ID | 任务 | 优先级 | 工时 | 交付物 |
|:---|:-----|:------:|:----:|:-------|
| 3.0+-001 | WebRTC信令服务 | P2 | 8h | signaling服务器 |
| 3.0+-002 | NAT穿透优化 | P2 | 12h | STUN/TURN集成 |
| 3.0+-003 | 冲突解决算法 | P1 | 16h | CRDT或向量时钟 |
| 3.0+-004 | 同步协议设计 | P1 | 12h | sync-protocol.md |
| 3.0+-005 | 端到端加密 | P2 | 12h | E2E加密实现 |

### 架构图

```
┌─────────────┐         ┌─────────────┐
│   Device A  │◄───────►│   Device B  │
│  (Initiator)│  WebRTC │  (Receiver) │
└──────┬──────┘         └──────┬──────┘
       │                       │
       ▼                       ▼
┌─────────────┐         ┌─────────────┐
│  DataChannel │◄───────►│  DataChannel│
└──────┬──────┘         └──────┬──────┘
       │                       │
       ▼                       ▼
┌─────────────┐         ┌─────────────┐
│  SyncEngine │◄───────►│  SyncEngine │
│  (CRDT)     │         │  (CRDT)     │
└─────────────┘         └─────────────┘
```

---

## 优先级矩阵

```
      影响面
   高 │ P2P同步   可观测性
      │  WASM完善  配置热重载
      │
   中 │ 限流      安全加固
      │  请求ID    债务清偿
      │
   低 │           安全响应头
      │
      └───────────────────────►
           低          高   紧急度
```

---

## 资源估算

| 阶段 | 周期 | 工时 | 角色 |
|:-----|:-----|:----:|:-----|
| v2.0 债务清偿 | 2周 | 16h | Engineer |
| v2.1 安全加固 | 2周 | 20h | Engineer |
| v2.2 可观测性 | 3周 | 40h | Engineer + SRE |
| v3.0 WASM完善 | 4周 | 60h | Engineer + Rust Dev |
| v3.0+ P2P同步 | 6周 | 80h | Architect + Engineer |

**总计**: ~216工时（约13周，1人全职）

---

## 风险评估

| 风险 | 影响 | 可能性 | 缓解措施 |
|:-----|:----:|:------:|:---------|
| WASM在Termux无法运行 | 高 | 中 | 保持JS降级方案，性能优化转向算法层面 |
| WebRTC NAT穿透困难 | 中 | 高 | 设计FILE_EXPORT可靠降级 |
| 配置热重载引入状态不一致 | 中 | 中 | 严格的配置校验，支持原子更新 |

---

## 收卷确认

本路线图基于12号建设性审计报告制定，涵盖：
- P1/P2债务清偿
- 安全加固
- 可观测性增强
- WASM完善
- P2P同步

**建议启动时间**: v2.0可立即启动，v3.0待WASM技术条件成熟

---

*制定者：Mike（审计官模式）*  
*日期：2026-02-26*
