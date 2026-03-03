# TURN服务器集成架构 v1.0

> DEBT-P2P-002 清偿方案 | 黄瓜睦 Architect | v3.5.0-sprint7

## 1. 架构目标

实现NAT穿透失败时的relay候选fallback，确保P2P连接在各种网络环境下可用。

## 2. 开源TURN方案选型

| 方案 | 协议支持 | 特点 | 适用场景 |
|------|----------|------|----------|
| **coturn** | RFC 5766/5780/7065 | 成熟稳定，高并发 | 生产部署 |
| **pion/turn** | RFC 5766 | Go编写，轻量嵌入 | 嵌入式/边缘 |

**推荐**: 服务端使用coturn，客户端SDK参考pion/turn设计。

## 3. 网络策略：LAN优先，WAN Fallback

```
┌─────────────────────────────────────────────────────────────┐
│                     连接策略决策树                            │
├─────────────────────────────────────────────────────────────┤
│  1. mDNS发现同网段peer ──→ 尝试host候选（LAN直连）            │
│        ↓ 失败                                                │
│  2. STUN获取srflx候选 ──→ 尝试P2P直连（WAN直连）              │
│        ↓ 失败（NAT对称型/严格防火墙）                          │
│  3. TURN获取relay候选 ──→ 中继转发（Relay模式）               │
└─────────────────────────────────────────────────────────────┘
```

## 4. ICE候选管理设计

```typescript
interface ICECandidate {
  type: 'host' | 'srflx' | 'relay';
  priority: number;     // host > srflx > relay
  address: string;
  port: number;
  foundation: string;
}

enum ConnectionState {
  LAN = 'lan',       // host候选成功
  DIRECT = 'direct', // srflx候选成功
  RELAY = 'relay',   // relay候选成功
  FAILED = 'failed'  // 所有候选失败
}
```

## 5. TURN认证流程

```
Client ──Allocate Request──────────────────→ TURN Server
         ←──401 Unauthorized (nonce realm)──
         ──Allocate + Authorization────────→
         ←──200 OK (relay address)──────────
```

- 长期凭证机制（Long-term credentials）
- HMAC-SHA1计算MESSAGE-INTEGRITY
- 401挑战响应，403不重试防暴力破解

## 6. 降级策略

| 场景 | 行为 |
|------|------|
| TURN服务器不可用 | 回退到STUN-only，UI提示"仅局域网可用" |
| 401认证失败 | 指数退避重试（1s→2s→4s→8s，max 3次） |
| 403禁止访问 | 立即停止，不重试 |
| 无TURN配置 | 向后兼容，保持现有mDNS+STUN行为 |

## 7. DEBT-P2P-002 清偿声明

- [x] TURN服务器架构设计
- [x] relay候选自动fallback机制
- [x] 401/403认证错误处理
- [x] 连接状态UI反馈

---
*Ouroboros衔尾蛇：穿透一切网络边界* ☝️🐍♾️
