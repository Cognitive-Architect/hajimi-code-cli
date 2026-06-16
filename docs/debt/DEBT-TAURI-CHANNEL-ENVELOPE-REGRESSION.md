# DEBT-TAURI-CHANNEL-ENVELOPE-REGRESSION: Tauri Channel Envelope 回归防线

> 创建日期: 2026-05-26  
> 当前状态: `CLOSED` — 代码修复已验证，回归测试已补充，真实 WebView 聊天已恢复  
> 优先级: `P0`  
> 关联模块: `src/interface/web/modules/tauri-bridge.js`, `src/interface/web/app.js`, `src/interface/desktop/src/main.rs`  

---

## 1. 债务摘要

DeepSeek OpenAI-compatible provider 在真实 WebView 中出现“测试连接通过，但聊天回复显示模型没有返回内容”的问题。实机平台用量证明模型确实收到请求并产生输出，后端诊断日志也证明 SSE 被正常解析并通过 Tauri Channel 发送；最终根因是前端 Tauri Channel 适配层没有处理 Tauri v2 的回调 envelope。

这是一个架构边界债务，不是 DeepSeek 专属问题。凡是通过 `tauri::ipc::Channel<T>` 从 Rust 命令向前端推送事件的链路，都会受到同一协议契约影响，包括聊天流、Agent Trace、资源告警等订阅类功能。

---

## 2. 用户可见现象

1. provider 测试连接显示通过。
2. 真实聊天请求发送后，DeepSeek 开放平台用量增长，说明 API 调用成功。
3. Hajimi UI 中 assistant turn 显示：
   - `未返回显式思考过程`
   - `模型没有返回内容。`
4. 会话统计中的请求轮次增长，但 assistant 内容为空。

---

## 3. 根因

Tauri v2 对从 JavaScript 作为 command 参数传入的 Channel 回调，会把真实业务 payload 包装为 envelope：

```js
{ message: response, id: n }
```

历史前端桥接层将回调参数直接传给业务层：

```js
this._handler(message);
```

聊天业务层因此收到的不是：

```js
{ chunk, done, error, promptTokens, completionTokens }
```

而是：

```js
{ message: { chunk, done, error, promptTokens, completionTokens }, id: n }
```

于是 `event.chunk` 永远为空，`parseStreamEvent` 只能得到空 buffer，最终 `streamChat` 判断 `pending === true`，渲染“模型没有返回内容。”。

---

## 4. 诊断证据

诊断构建记录的关键字段如下：

```json
{
  "backend": {
    "outputEvents": 17,
    "outputChars": 47,
    "channelSendEvents": 18,
    "channelSendErrors": 0
  },
  "frontend": {
    "onmessageCount": 18,
    "chunkEvents": 0,
    "chunkChars": 0,
    "pending": true,
    "returnLen": 0
  }
}
```

结论：

- `outputChars > 0`：模型响应内容已被后端 parser 解析出来。
- `channelSendEvents > 0` 且 `channelSendErrors = 0`：Rust 后端向 Tauri Channel 发送成功。
- `onmessageCount > 0`：前端确实收到 Channel 回调。
- `chunkChars = 0`：业务层没有从回调参数上读取到 `chunk`。

该组合直接指向 Channel envelope 未解包，而不是 provider、API key、DeepSeek schema、SSE parser 或 CSS 渲染问题。

---

## 5. 修复策略

修复点必须放在统一 Tauri 桥接层，而不是每个业务订阅点各自兼容。

目标文件：

```text
src/interface/web/modules/tauri-bridge.js
```

正确行为：

```js
const payload = message
  && typeof message === 'object'
  && Object.prototype.hasOwnProperty.call(message, 'message')
  ? message.message
  : message;
this._handler(payload);
```

这样聊天流、Agent Trace、资源告警等所有 Channel 使用方都能收到业务 payload，而不是 Tauri envelope。

---

## 6. 回归风险

高风险改法：

1. 在 `app.js` 的 `streamChat` 内只兼容 `event.message.chunk`。
2. 在某一个订阅功能里手写 envelope 解包。
3. 将 Tauri Channel 与普通 browser `MessageEvent` 混用，假设 payload 一定在 `event.data`。
4. 只用 mock Channel 做前端测试，mock 直接传业务 payload，绕过真实 Tauri envelope。

这些做法会让聊天流暂时恢复，但留下其他 Channel 订阅的同类回归。

---

## 7. 关闭条件

该债务可以标记为关闭，但必须保留回归测试或人工验证记录。关闭条件：

| # | 条件 | 状态 | 证据 |
|---|------|------|------|
| 1 | `src/interface/web/modules/tauri-bridge.js` 在统一入口解包 `{ message, id }` envelope | ✅ | 第 42–47 行 envelope 解包逻辑 |
| 2 | `node --check src/interface/web/modules/tauri-bridge.js` 通过 | ✅ | 2026-05-25 运行通过 |
| 3 | `node --check src/interface/web/app.js` 通过 | ✅ | 2026-05-25 运行通过 |
| 4 | `cargo check -p hajimi-desktop --features custom-protocol` 通过 | ✅ | 2026-05-25 运行通过 |
| 5 | 真实 Tauri WebView 流式聊天 UI 正常显示模型正文 | ✅ | DeepSeek provider 实机验证，UI 正常渲染 assistant 回复 |
| 6 | 前端模块测试覆盖 envelope + legacy payload | ✅ | `tests/frontend/day20_tauri_channel_envelope_regression.test.js` 通过 |
| 7 | 诊断日志不记录 API key 或完整敏感提示词 | ✅ | `preview_for_diagnostic` 仅截取前 120 字符；日志仅含统计字段（outputEvents/outputChars 等），不含完整消息或密钥 |

---

## 8. 后续建议

建议新增一个轻量 frontend test，专门验证 `HajimiTauri.Channel` 回调会把 Tauri envelope 解包成业务 payload。测试应直接针对 `src/interface/web/modules/tauri-bridge.js`，不要只测 `streamChat`，因为根因位于桥接层。

建议后续任何新增 Channel 使用方遵循规则：

- 业务代码只接收业务 payload。
- Tauri envelope 只允许在 `tauri-bridge.js` 统一处理。
- mock Channel 必须覆盖真实 envelope 形态。

---

## 9. 签名

- **根因定位**: Tauri v2 Channel command-argument callback envelope 未解包。
- **修复位置**: `src/interface/web/modules/tauri-bridge.js`。
- **验证证据**: DeepSeek 实机聊天已恢复，UI 正常渲染 `pong`。
