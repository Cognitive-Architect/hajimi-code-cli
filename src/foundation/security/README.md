# Security Module

HAJIMI IDE 地基层的安全基础设施，提供限流、审计日志与安全响应头等防护能力。

## 职责

- **Token Bucket 限流**：
  - `rate-limiter.js` — 基于内存的 Token Bucket 算法实现，默认配置 burst=20、refillRate=100/60 req/s；支持单 IP 独立计数、`retryAfter` 计算、过期 bucket 清理（默认 1 小时）。
  - `rate-limiter-sqlite-luxury.js` — 豪华版 SQLite 持久化限流器（`LuxurySQLiteRateLimiter`），清偿 DEBT-SEC-001：
    - WAL 模式读写并发
    - 批量写入队列（默认 100 条事务提交）
    - 预编译语句缓存（`getBucket` / `updateBucket` / `deleteBucket`）
    - 异步后台刷盘（`db.export()` + `fs.writeFile`）
    - 定期 WAL checkpoint（5 分钟）与 SIGINT 强制刷盘
- **限流器工厂**：`rate-limiter-factory.js` 提供统一接口，支持内存版与 SQLite 版切换；`rate-limiter-redis.js` / `rate-limiter-redis-v2.js` 提供 Redis 分布式限流备选。
- **安全响应头**：`headers.js` 提供轻量级 Helmet.js 替代中间件，设置 `X-Frame-Options: DENY`、`X-Content-Type-Options: nosniff`、`Referrer-Policy`、`Permissions-Policy` 等基础安全头。
- **熔断器支持**：当失败率达到 50% 时触发熔断，30 秒恢复期（与限流器协同工作）。

## 关键文件

- `rate-limiter.js` — 内存 Token Bucket 限流器
- `rate-limiter-sqlite-luxury.js` — SQLite 持久化豪华版限流器（sql.js）
- `rate-limiter-factory.js` — 限流器工厂与多后端切换
- `headers.js` — 安全响应头中间件
- `rate-limiter.test.js` — Token Bucket 单元测试（RATE-001 ~ RATE-007）

## 快速开始

```javascript
const { LuxurySQLiteRateLimiter } = require('./rate-limiter-sqlite-luxury');

const limiter = new LuxurySQLiteRateLimiter({ capacity: 20, refillRate: 100 / 60 });
await limiter.init();

const result = await limiter.checkLimit('192.168.1.1');
console.log(result.allowed, result.remaining);
```

## 测试

```bash
# 运行 Token Bucket 限流器单元测试
node src/foundation/security/rate-limiter.test.js
```

## 依赖

- `sql.js` — 纯 JavaScript SQLite 实现（豪华版限流器，零原生编译）
- Node.js 内置模块：`fs`（`fs.promises`）、`path` — 数据库文件读写与路径处理
- Redis 客户端（可选）— `rate-limiter-redis.js` / `rate-limiter-redis-v2.js` 分布式场景
