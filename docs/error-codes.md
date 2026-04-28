# Hajimi IDE 错误码索引

> **用途**：帮助用户根据错误提示快速自诊断问题。  
> **更新日期**：2026-04-28  
> **覆盖范围**：Interface 层、Engine 层、Foundation & Intelligence 通用层。  
> **说明**：本文档所有错误均来自真实源码，可通过 `Select-String` / `grep` 在对应文件中定位。

---

## 目录

1. [Interface Layer（界面层）](#interface-layer界面层)
2. [Engine Layer（引擎层）](#engine-layer引擎层)
3. [General（通用层：Foundation & Intelligence）](#general通用层)
4. [调试指南](#调试指南)

---

## Interface Layer（界面层）

> 对应源码：`src/interface/desktop/src/main.rs`

| 错误码 | 错误提示 / 症状 | 原因 | 解决方案 | 代码出处 |
|--------|----------------|------|----------|----------|
| **E-I001** | `命令 '{}' 不在白名单中` | 用户或 AI 调用了 `run_command` 时传入的命令不在 `ALLOWED_COMMANDS` 白名单内（如 `rm`、`sudo` 等被显式禁止）。 | ① 确认命令属于白名单：`git`、`cargo`、`npm`、`node`、`rustc`、`bash`、`pwsh`、`curl` 等；<br>② 若需新命令，联系维护者扩展 `ALLOWED_COMMANDS` 常量。 | `src/interface/desktop/src/main.rs:212` |
| **E-I002** | `路径越界: {} 不在工作目录 {} 内` | `validate_path_within_workspace` 检测到请求路径解析后超出了 `hajimi-workspace` 沙箱根目录，或路径中包含 `..` 遍历。 | ① 确保访问的文件位于 `Documents/hajimi-workspace/` 下；<br>② 不要在路径中使用 `..` 向上跳转；<br>③ 使用绝对路径时，确保其落在工作区内。 | `src/interface/desktop/src/main.rs:168` |
| **E-I003** | `无法访问密钥存储: {}` | `delete_api_key_with_profile` 调用 OS Keyring（Windows Credential Manager / macOS Keychain / Linux Secret Service）失败，通常因权限不足或密钥服务未运行。 | ① Windows：以当前用户身份运行，确认 `Credential Manager` 服务正常；<br>② macOS：在「钥匙串访问」中检查权限；<br>③ Linux：确保 `gnome-keyring` 或 `kwallet` 已解锁。 | `src/interface/desktop/src/main.rs:445` |
| **E-I004** | `删除密钥失败: {}` | 删除密钥时 Keyring 返回了除 `NoEntry` 以外的错误（如文件被占用、权限拒绝）。 | ① 重启 IDE 后重试；<br>② 手动在系统密钥管理器中删除对应 `provider:*` 条目；<br>③ 检查杀毒软件是否拦截了密钥存储访问。 | `src/interface/desktop/src/main.rs:449` |
| **E-I005** | `参数包含非法字符: {}` | `run_command` 的参数中含有 `..` 或元字符 `;`、`&`、`|`、`` ` ``、`$`、`(`、`)`、`{`、`}`、`<`、`>`，被安全过滤器拦截。 | ① 移除参数中的 Shell 元字符；<br>② 如需传递复杂参数，改用文件或环境变量方式；<br>③ 参考 `docs/debt/SHELL-FEATURE-DEBT-002.md` 了解受限功能。 | `src/interface/desktop/src/main.rs:218` |
| **E-I006** | `Provider '{}' already exists` | 添加新 Provider 时，`add_provider_config` 发现全局或工作区配置中已存在相同 `id`。 | ① 更换 Provider ID；<br>② 或先删除旧配置再重新添加；<br>③ 检查工作区级与全局配置是否冲突。 | `src/interface/desktop/src/main.rs:603` |
| **E-I007** | `Provider '{}' not found` | 更新 Provider 配置时找不到对应 ID。 | ① 确认 Provider ID 拼写正确；<br>② 使用 `get_provider_configs` 查看当前可用列表；<br>③ 若目标配置位于工作区级，确保 `workspace_path` 参数正确。 | `src/interface/desktop/src/main.rs:630` |
| **E-I008** | `Old string not found in file` | `preview_edit` 或内联编辑时，传入的 `old_string` 在目标文件中未找到完全匹配。 | ① 确认 `old_string` 与文件实际内容一致（包括空格、换行）；<br>② 使用 `preview_edit` 接口先验证 diff；<br>③ 若文件已变更，先重新读取最新内容。 | `src/interface/desktop/src/main.rs:1274` |
| **E-I009** | `Invalid approval level` | `set_approval_level` 接收到的级别不在合法列表 `Auto / Advisory / Required / Critical / Override` 中。 | ① 检查传入值的大小写和拼写；<br>② 使用 UI 下拉框选择，避免手动输入错误。 | `src/interface/desktop/src/main.rs:1134` |

---

## Engine Layer（引擎层）

> 对应源码：`src/engine/*`

| 错误码 | 错误提示 / 症状 | 原因 | 解决方案 | 代码出处 |
|--------|----------------|------|----------|----------|
| **E-E001** | `Tool not found: {0}` | `ToolRegistry` 中未注册该工具，或工具名拼写错误。 | ① 使用 `list_tools` 查看已注册工具列表；<br>② 确认工具名大小写（如 `edit_file` 而非 `EditFile`）；<br>③ 检查 `build_registry()` 中是否已注册该工具。 | `src/engine/tool-system/src/error.rs:6`<br>`src/engine/llm-core/src/error.rs:5`<br>`src/engine/worker/src/error.rs:6` |
| **E-E002** | `Execution timeout after {0}ms` | 工具执行耗时超过设定阈值（Shell 默认 30s，Worker/LLM 按各自配置）。 | ① 优化命令或查询以减少耗时；<br>② 对于长任务，使用后台 Agent 模式（`@agent continue-background`）；<br>③ 检查网络连接是否导致 LLM 流式响应卡顿。 | `src/engine/llm-core/src/error.rs:8`<br>`src/engine/tool-system/src/error.rs:9` |
| **E-E003** | `Retry exhausted after {attempts} attempts: {source}` | 带重试策略的操作（如 LLM 调用、网络请求）在最大重试次数后仍失败。 | ① 查看底层 `source` 错误定位根因；<br>② 检查 API Key 是否有效、网络是否稳定；<br>③ 若服务端限流，稍后重试或切换 Provider。 | `src/engine/llm-core/src/error.rs:11` |
| **E-E004** | `Permission denied: {0}` | 工具权限级别为 `Deny`，或 Shell 白名单 / 路径沙箱拦截。 | ① 在设置中将对应工具权限调整为 `Ask` 或 `Allow`；<br>② 对于 Shell 工具，确认命令在白名单内且不含元字符；<br>③ 文件操作需确保路径在 `hajimi-workspace` 内。 | `src/engine/tool-system/src/error.rs:27` |
| **E-E005** | `Git failed (exit: {}): {}` | `GitCli::exec` 执行 `git` 子进程返回非零退出码（如合并冲突、无提交权限、分支保护）。 | ① 查看 `stderr` 中的具体 Git 错误信息；<br>② 检查当前目录是否为有效 Git 仓库（`.git` 存在）；<br>③ 处理合并冲突或配置 `user.email` / `user.name`。 | `src/engine/tool-system/src/git_cli.rs:22` |
| **E-E006** | `Invalid git repository` | `GitCli::new` 发现指定路径不存在或缺少 `.git` 目录。 | ① 确认路径正确且为 Git 仓库根目录；<br>② 若未初始化，先执行 `git init`；<br>③ 检查文件系统权限。 | `src/engine/tool-system/src/git_cli.rs:23` |
| **E-E007** | `Command '{}' not in strict allow-list...` | `shell.rs` 的 `BashExecutor` / `PowerShellExecutor` 检测到首命令 token 不在 `ALLOWED_COMMANDS` 白名单。 | ① 使用白名单内的命令（见 E-I001 列表）；<br>② 禁止通过绝对路径调用白名单外二进制文件；<br>③ 查阅 `docs/debt/SHELL-FEATURE-DEBT-002.md`。 | `src/engine/tool-system/src/shell.rs:49` |
| **E-E008** | `Metacharacters (; & | \` $ etc.) not permitted...` | Shell 命令中包含分号、管道、重定向、反引号等危险元字符，被安全策略拒绝。 | ① 删除元字符，改用安全的参数化调用；<br>② 复杂管道/重定向功能当前已降级，需等待后续版本支持；<br>③ 如需批量操作，使用 `MultiEditTransaction` 或脚本工具替代。 | `src/engine/tool-system/src/shell.rs:61` |
| **E-E009** | `LLM error: {0}` | LLM 客户端返回业务级错误（如认证失败、模型不存在、内容过滤）。 | ① 检查 API Key 是否过期或被撤销；<br>② 确认所选模型在当前 Provider 可用；<br>③ 查看审计日志 `get_audit_logs` 获取详细状态。 | `src/engine/llm-core/src/error.rs:23` |
| **E-E010** | `Streaming error: {0}` | 流式聊天过程中连接中断或 SSE/Chunk 解析失败。 | ① 检查本地网络与 VPN 设置；<br>② 对于 Ollama 本地模型，确认服务正在监听；<br>③ 增大超时或切换为非流式模式（如支持）。 | `src/engine/llm-core/src/error.rs:26` |

---

## General（通用层）

> 覆盖 Foundation（网络、WASM）与 Intelligence（Agent、Memory、Knowledge、Codex-Twist、Chimera-REPL）模块。

| 错误码 | 错误提示 / 症状 | 原因 | 解决方案 | 代码出处 |
|--------|----------------|------|----------|----------|
| **E-G001** | `JSON parse error: {0}` | WebSocket / JSON-RPC 通信中收到的 payload 不是合法 JSON。 | ① 检查网络是否被代理篡改；<br>② 确认客户端发送的数据为 UTF-8 编码；<br>③ 查看后端日志中的原始 payload 定位截断位置。 | `src/foundation/network/src/protocol.rs:167` |
| **E-G002** | `Method not found: {0}` | JSON-RPC 请求了服务端未实现的方法。 | ① 对照 API 文档确认方法名；<br>② 检查前后端版本是否一致；<br>③ MCP 服务器场景下，确认工具是否已在 `ToolRegistry` 注册。 | `src/foundation/network/src/protocol.rs:171` |
| **E-G003** | `WasmMemory pointer is null` | WASM HNSW 向量计算模块接收到空指针。 | ① 确认 JS 侧传递的 `SharedArrayBuffer` 已正确初始化；<br>② 检查 WASM 内存是否因页面刷新而被释放；<br>③ 重新加载页面或重启索引任务。 | `src/foundation/wasm/src/memory.rs:18` |
| **E-G004** | `Memory access out of bounds: ptr=0x{:x}, len={}, max={}` | WASM 内存访问越界，通常因向量维度与分配长度不匹配导致。 | ① 确认向量维度为 384（与 `EMBEDDING_DIM` 一致）；<br>② 检查 `num_vectors * dim` 是否溢出；<br>③ 重新初始化 WASM 实例并分配足够内存。 | `src/foundation/wasm/src/memory.rs:23` |
| **E-G005** | `BLAKE3 checksum mismatch` | `.hctx` 存档文件在读取时校验和失败，说明数据被篡改或损坏。 | ① 检查磁盘健康状况；<br>② 若文件来自传输/同步，重新获取完整副本；<br>③ 无法修复时删除损坏存档，依赖上层自动重建。 | `src/intelligence/chimera/chimera-repl/src/archive_writer.rs:29` |
| **E-G006** | `Invalid HCTX magic bytes` | `.hctx` 文件头魔数不匹配，可能文件格式错误或版本不兼容。 | ① 确认文件为 Hajimi 生成的 `.hctx` 格式；<br>② 若版本升级导致，使用兼容模式读取或迁移工具转换；<br>③ 不要手动编辑二进制存档文件。 | `src/intelligence/chimera/chimera-repl/src/archive_writer.rs:30` |
| **E-G007** | `Plan not initialized` | Agent Core 在计划未初始化时尝试执行后续步骤。 | ① 确保 Agent 启动时正确设置了初始目标（goal）；<br>② 检查 `AgentLoop` 状态机是否按 `Observe → Retrieve → Plan → Act` 顺序执行；<br>③ 重启 Agent 会话。 | `src/intelligence/agent-core/ports.rs:21` |
| **E-G008** | `Governance rejected: {}` | 可插拔治理模块（Governance）拒绝了当前操作（如超出权限、风险评分过高）。 | ① 查看拒绝原因描述，调整输入或参数；<br>② 在设置中临时提升审批级别（如从 `Required` 降为 `Advisory`）；<br>③ 检查是否触发了自动安全规则（如路径越界、危险命令）。 | `src/intelligence/agent-core/ports.rs:23` |
| **E-G009** | `Embedding timeout: exceeded 500ms` | Dream 层调用 ONNX 嵌入模型时超过 500ms 超时。 | ① 首次加载模型可能较慢，可重试；<br>② 检查本地 ONNX Runtime 环境是否正常；<br>③ 若使用 CPU 推理，关闭其他占用资源的程序；<br>④ 确认 `embedding_model.onnx` 文件存在且未损坏。 | `src/intelligence/memory/src/dream.rs:29` |
| **E-G010** | `Invalid dimension: expected 384, got {actual}` | 传入的向量维度不是 384，与 HNSW / Dream 层的固定维度不匹配。 | ① 确认嵌入模型输出维度为 384；<br>② 检查向量序列化/反序列化过程中是否有字节丢失；<br>③ 重新生成 embedding 并确保长度一致。 | `src/intelligence/memory/src/dream.rs:30` |
| **E-G011** | `Tier not available: {0:?}` | `SyncMemoryGateway` 尝试访问的内存层级（Session/Auto/Dream/Graph/Cloud）未初始化或不可用。 | ① 确认对应层级已在 `MemoryGateway` 中初始化；<br>② Cloud 层需要提前配置 `CloudIdentity`；<br>③ 检查 SQLite/LevelDB 等底层存储是否打开成功。 | `src/intelligence/memory/src/sync_gateway.rs:92` |
| **E-G012** | `Depth limit exceeded (max 2 hops)` | 知识图谱遍历或路径查询时 `max_depth > 2`，超出硬编码安全限制。 | ① 将查询深度调整为 1 或 2；<br>② 若业务需要更深遍历，通过多次查询拼接结果；<br>③ 联系维护者评估是否需要放宽限制。 | `src/intelligence/memory/src/graph_query.rs:12` |
| **E-G013** | `Invalid ADR format` | `build_from_adr_dir` 扫描 ADR 目录时路径不存在或文件不符合 Markdown + Frontmatter 格式。 | ① 确认 `docs/adr/` 目录存在且包含 `.md` 文件；<br>② 检查 ADR 文件是否包含 `title:` 和 `id:` 前置元数据；<br>③ 修复或重新生成 ADR 文件。 | `src/intelligence/knowledge/src/adr_index.rs:30` |
| **E-G014** | `Encryption: {0}` / `Decryption: {0}` | Cloud 记忆层 E2EE 加密/解密失败（Age/X25519 或 Argon2id KDF 错误）。 | ① 确认密码正确且未混淆大小写；<br>② 检查密文是否完整传输（未截断）；<br>③ 若使用旧版本备份，尝试 `decrypt_legacy` 兼容路径。 | `src/intelligence/memory/src/cloud.rs:45`<br>`src/intelligence/memory/src/cloud.rs:46` |
| **E-G015** | `输入为空` / `缺少元数据块` | Codex-Twist LCR 适配器解析 `.hctx` 或线程数据时缺少必要字段。 | ① 确认输入数据非空；<br>② 检查 `.hctx` 文件头是否包含完整的元数据块（`---` 包围的 Frontmatter）；<br>③ 重新导出或手动补全缺失字段。 | `src/intelligence/codex-twist/src/lcr_adapter.rs:175` |

---

## 调试指南

### 1. 查看后端日志

Hajimi IDE 后端基于 Tauri v2 + Rust，日志默认输出到以下位置：

- **Windows**：`%APPDATA%\Hajimi\logs\` 或直接从开发终端查看 `cargo tauri dev` 输出。
- **macOS**：`~/Library/Application Support/Hajimi/logs/`。
- **Linux**：`~/.config/Hajimi/logs/`。

在 `cargo tauri dev` 模式下，所有 `eprintln!`、`println!` 以及 `tracing` 日志都会实时打印到启动终端。建议：

1. 复现问题时保持终端可见；
2. 使用 `RUST_LOG=debug cargo tauri dev` 开启详细日志（若项目已配置 `tracing-subscriber`）。

### 2. 使用 DevTools 调试前端

前端为纯 HTML/CSS/JS，无框架打包：

1. **打开 DevTools**：在 Hajimi IDE 窗口内按 `Ctrl + Shift + I`（Windows/Linux）或 `Cmd + Option + I`（macOS）。
2. **查看 Console**：所有前端错误、`window.__TAURI__.invoke` 调用异常都会显示在此处。
3. **Network 面板**：监控 WebSocket 或 HTTP 请求（如 LLM 流式响应、Provider 验证请求）。
4. **Sources 面板**：前端源码位于 `src/interface/web/`，可直接断点调试。

### 3. 使用 Select-String / grep 定位源码

所有错误消息均可在源码中追溯。示例：

```powershell
# PowerShell (Windows)
Select-String -Path src\interface\desktop\src\main.rs -Pattern "不在白名单中"
Select-String -Path src\engine\tool-system\src\shell.rs -Pattern "allow-list"
Select-String -Path src\intelligence\memory\src\dream.rs -Pattern "timeout"

# Bash / Git Bash
grep -rn "不在白名单中" src/interface/desktop/src/main.rs
grep -rn "BLAKE3 checksum mismatch" src/intelligence/chimera/
grep -rn "Depth limit exceeded" src/intelligence/memory/src/
```

### 4. 常见排查清单

| 现象 | 优先检查项 |
|------|-----------|
| 任何命令都提示「不在白名单」 | `ALLOWED_COMMANDS` 是否被意外修改；确认不是通过 `bash -c` 拼接。 |
| 文件读写失败 | 路径沙箱验证；确认文件在 `hajimi-workspace` 内；检查磁盘权限。 |
| LLM 无响应或超时 | API Key 是否正确（Keyring / 环境变量）；网络连通性；Provider 配置 `base_url`。 |
| Agent 卡住或报 Plan 错误 | 查看 `subscribe_agent_trace` 事件流；检查 `approval_level` 是否设为 `FullDeny`。 |
| 内联编辑 diff 不匹配 | 使用 `preview_edit` 先验证；确认 `old_string` 与文件当前内容完全一致。 |
| WASM/HNSW 崩溃 | 确认 `SharedArrayBuffer` 跨域隔离头已配置；向量维度严格为 384。 |

---

*本文档随源码同步维护。新增错误类型时，请同步更新本表格并确保 `Select-String` 可追溯。*
