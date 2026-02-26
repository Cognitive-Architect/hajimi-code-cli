# DEBT-SEC-001: Rate Limiting 内存存储限制

> **债务ID**: DEBT-SEC-001  
> **分类**: 技术债务 - 安全组件  
> **优先级**: P2  
> **声明日期**: 2026-02-27  
> **预计清偿版本**: Phase 3

---

## 问题描述

当前实现的 Token Bucket 限流器使用**内存 Map**存储每个 IP 的 bucket 状态：

```javascript
this.buckets = new Map(); // IP -> {tokens, lastRefill}
```

这带来以下限制：

### 限制 1: 进程重启清零
- 服务器重启后，所有限流计数器重置
- 恶意用户可通过触发重启绕过限流

### 限制 2: 单机限流
- 无法在分布式环境下共享限流状态
- 多实例部署时，每个实例独立计数

### 限制 3: 内存占用
- 每个 IP 占用约 50 bytes
- 100K 个 IP = ~5MB 内存
- 虽可接受，但无上限增长存在风险

---

## 临时缓解措施

1. **定期清理**: `cleanup()` 方法清理过期 bucket（1小时无活动）
2. **容量上限**: 后续考虑添加最大 bucket 数量限制（LRU 淘汰）
3. **监控告警**: 监控内存使用，超限报警

---

## 清偿方案

### 方案 A: Redis 存储（推荐）

```javascript
// 使用 Redis 替代内存 Map
class RedisRateLimiter {
  constructor(redisClient) {
    this.redis = redisClient;
  }
  
  async consume(ip, tokens = 1) {
    // 使用 Redis Lua 脚本实现原子操作
    const script = `
      local key = KEYS[1]
      local capacity = tonumber(ARGV[1])
      local refillRate = tonumber(ARGV[2])
      local now = tonumber(ARGV[3])
      local requested = tonumber(ARGV[4])
      
      local bucket = redis.call('HMGET', key, 'tokens', 'lastRefill')
      local tokens = tonumber(bucket[1]) or capacity
      local lastRefill = tonumber(bucket[2]) or now
      
      -- 补充 token
      local elapsed = now - lastRefill
      tokens = math.min(capacity, tokens + elapsed * refillRate)
      
      if tokens >= requested then
        tokens = tokens - requested
        redis.call('HMSET', key, 'tokens', tokens, 'lastRefill', now)
        redis.call('EXPIRE', key, 3600) -- 1小时过期
        return {1, tokens}
      else
        redis.call('HMSET', key, 'tokens', tokens, 'lastRefill', now)
        return {0, tokens}
      end
    `;
    
    return this.redis.eval(script, 1, ip, ...);
  }
}
```

**优点**:
- 进程重启后数据不丢失
- 支持分布式部署
- Redis 性能足够（单节点 100K QPS）

**缺点**:
- 引入外部依赖（Redis）
- 增加网络延迟（~1ms）

### 方案 B: 本地文件存储

使用 LevelDB/BoltDB 本地存储限流状态。

**优点**:
- 无外部依赖
- 进程重启数据保留

**缺点**:
- 磁盘 I/O 性能较低
- 分布式仍需解决

---

## 清偿计划

| 阶段 | 时间 | 内容 |
|:---|:---|:---|
| Phase 2 | 当前 | 内存实现 + DEBT声明 |
| Phase 3 | 后续 | 评估 Redis 方案，可选实现 |
| Phase 4 | 后续 | 完全清偿，Redis 成为默认 |

---

## 验收标准

- [ ] Redis 限流器实现
- [ ] 配置切换（memory/redis）
- [ ] 性能测试（Redis vs Memory）
- [ ] 集成测试（多实例共享状态）

---

> 💡 **使用建议**: 当前阶段建议单机部署，如需集群，请在前置负载均衡层做限流。
