# 32-AUDIT-SPRINT4-ALPHA 建设性审计报告

## 审计结论
- **评级**: **B / 有条件Beta**
- **状态**: 有条件Go（需修复后进入Beta）
- **与31号自测一致性**: 部分一致（E2E测试使用Mock而非真实网络，密钥派生未按规范实现）

---

## 进度报告（分项评级）

| 维度 | 评级 | 说明 |
|:---|:---:|:---|
| B-31/01包名同步 | ✅ A | 10处`@koush/wrtc`，零旧包名残留，98行合规 |
| B-31/02功能完整性 | ✅ A | 4项功能全部实现（文件/文本/断点/拥塞），294行 |
| E2E测试真实性 | ⚠️ B | 强制`@koush/wrtc`✓，但使用MockChannel内存模拟而非真实网络 |
| 内存安全性 | ✅ A | 7处cleanup，无泄漏风险 |
| 加密安全性 | ⚠️ B | AES-256-GCM实现正确，但密钥派生与规范不符（randomBytes vs scryptSync）|
| 协议规范性 | ✅ A | Mermaid状态机+4消息格式完整 |

**整体评级 B**: 核心功能完整，但E2E测试真实性和密钥派生实现需补正。

---

## 关键疑问回答（Q1-Q4）

### Q1（E2E真实性）: 25 passed是否真实可复现？
**结论**: ⚠️ **可复现但非真实网络传输**

E2E测试确实产生25 passed/0 failed（验证通过），但存在以下情况：
- ✅ 强制使用`@koush/wrtc`（第6行无try-catch）
- ✅ 真实测试了DataChannelManager逻辑
- ⚠️ **使用MockChannel进行内存模拟**（第17-22行），而非真实WebRTC网络传输
- ⚠️ 测试的是本地内存消息传递，非真实ICE/STUN/TURN穿透

**证据**:
```javascript
// tests/datachannel-transfer.e2e.js:17-22
class MockChannel {
  constructor() { this.readyState = 'open'; this._peer = null; this._onmessage = null; }
  send(data) { if (this._peer?._onmessage) setImmediate(() => this._peer._onmessage({ data })); }
  // ...
}
```

**建议**: 补充真实网络E2E测试（两进程WebRTC握手+DataChannel传输）。

---

### Q2（代码可读性）: 294行是否为可读代码？
**结论**: ✅ **代码可读性良好**

- 代码结构清晰，有注释说明
- 使用标准JavaScript风格，非压缩代码
- 符合294行约束（≤300）

---

### Q3（加密安全）: AES-256-GCM密钥是否硬编码？
**结论**: ⚠️ **无硬编码，但与协议规范不符**

**实现**（datachannel-manager.js:21）:
```javascript
this.cryptoKey = crypto.randomBytes(32); // 随机生成256位密钥
```

**协议规范**（DATACHANNEL-PROTOCOL-v1.0.md:122）:
```javascript
const key = crypto.scryptSync(sharedSecret, 'salt', 32); // 从共享密钥派生
```

**差异分析**:
- ✅ 实现：每实例随机生成密钥（内存安全，但两peer无法互通）
- 📋 规范：从sharedSecret派生（支持跨peer密钥协商）
- ⚠️ 当前实现导致E2E测试中必须`dcmB.cryptoKey = dcmA.cryptoKey`（第28行hack）

**安全风险**: 无硬编码风险，但设计不符合协议规范，不支持真实P2P密钥交换。

---

### Q4（拥塞控制动态性）: 滑动窗口是否真实动态调整？
**结论**: ✅ **真实动态调整**

**实现验证**（datachannel-manager.js:227-235）:
```javascript
adjustWindow(transferId, event) {
  if (event === 'success' && tx.window < MAX_WINDOW) {
    tx.window = Math.min(tx.window + 1, MAX_WINDOW); // 成功+1
  } else if (event === 'loss' && tx.window > MIN_WINDOW) {
    tx.window = Math.max(Math.floor(tx.window / 2), MIN_WINDOW); // 丢包/2
  }
}
```

- ✅ RTT测量实现（第172-175行）
- ✅ 窗口范围1-32（MIN_WINDOW/MAX_WINDOW）
- ✅ 成功递增/丢包减半的动态算法
- ✅ 拥塞控制消息发送（第240-243行）

---

## 验证结果（V1-V6）

| 验证ID | 结果 | 证据 |
|:---|:---:|:---|
| V1-sh包名 | ✅ PASS | 10处`@koush/wrtc`（标准≥2） |
| V2-sh残留 | ✅ PASS | 0处旧包名残留 |
| V3-功能实现 | ✅ PASS | 11处功能命中（≥4） |
| V4-内存清理 | ✅ PASS | 7处cleanup（≥5） |
| V5-E2E强制 | ✅ PASS | 第6行强制`require('@koush/wrtc')` |
| V6-密钥派生 | ⚠️ PARTIAL | 使用`randomBytes`而非`scryptSync` |

---

## FIND清单

| FIND-ID | 严重度 | 描述 | 位置 |
|:---|:---:|:---|:---|
| FIND-032-01 | MEDIUM | E2E测试使用MockChannel模拟，非真实WebRTC网络传输 | `tests/datachannel-transfer.e2e.js:17-22` |
| FIND-032-02 | MEDIUM | 密钥派生使用`randomBytes`而非协议规范的`scryptSync` | `src/p2p/datachannel-manager.js:21` |
| FIND-032-03 | LOW | E2E测试需要hack共享密钥（`dcmB.cryptoKey = dcmA.cryptoKey`） | `tests/datachannel-transfer.e2e.js:28` |

---

## Sprint4阶段判定

- **当前阶段**: Alpha（功能实现完成，测试覆盖不完全真实）
- **下一步建议**: 
  1. 补充真实网络E2E测试（两进程WebRTC握手）
  2. 修复密钥派生实现（支持sharedSecret + scryptSync）
  3. 完成后进入Beta
- **预计工作量**: 4-6小时
- **到RC时间**: +2天（含Beta测试+性能验证）

---

## 落地可执行路径

### Beta-ready条件
1. **真实网络E2E测试**: 启动两个Node进程，真实WebRTC握手+DataChannel传输
2. **密钥派生修复**: 实现`deriveKey(sharedSecret)`函数，使用`crypto.scryptSync`
3. **回归验证**: `node tests/webrtc-handshake.e2e.js` + `node tests/datachannel-transfer.e2e.js` 全绿

---

## 审计喵评语

🥁 "无聊喵"（B级，minor债务需补正）

Sprint4 Alpha功能完整度不错喵~ 4项功能都真实实现了（非stub），代码可读性良好，内存清理到位。但是有两个问题需要补正喵：

1. **E2E测试用了MockChannel**（内存模拟），不是真实WebRTC网络传输，这叫"单元测试冒充E2E"喵
2. **密钥派生没按规范来**，用了randomBytes而不是scryptSync，导致两peer没法天然互通，还要hack共享密钥

建议补充真实网络测试和修复密钥派生，然后就能进Beta了喵。不是返工，是补正，半天就能搞定喵！

---

## 归档建议

- 审计报告归档: `audit report/32/32-AUDIT-SPRINT4-ALPHA.md`
- 关联状态: ID-191（v3.3.0-alpha里程碑）
- 审计链: 31→32（Sprint4 Alpha验证闭环）
- Sprint4阶段: **Alpha → 有条件Beta**（需完成FIND-032-01/02修复）

---

*审计员签名: Mike 🐱 | 2026-02-28*
