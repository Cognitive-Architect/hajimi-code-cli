# HAJIMI Master Plan - 豪华镶钻版开发路线图

> **版本**: v1.0  
> **状态**: 规划中  
> **创建时间**: 2026-04-03  
> **核心原则**: 全都要 + 模块化 + 可裁剪

---

## 🎯 核心理念

不是追赶 Claude Code，而是打造**属于自己的、终极定制的 AI 编程助手**。

- ✅ **功能全面**: 瑞士军刀 + 镶钻
- ✅ **模块化**: 每个功能可独立开关
- ✅ **长期积累**: 越用越懂你的个人知识库
- ❌ **语音功能**: 已砍掉（确实用不到）

---

## 💎 架构总览

```
┌─────────────────────────────────────────────────────────────────┐
│                    HAJIMI CLI - 豪华镶钻版                        │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐   │
│  │  💎 核心底座 (Core Kernel) - 永不裁剪                     │   │
│  │  • QueryEngine (Rust) - 查询引擎                         │   │
│  │  • ToolRegistry - 工具注册中心                           │   │
│  │  • EventLoop - 事件循环                                  │   │
│  │  • ConfigManager - 配置管理                              │   │
│  └─────────────────────────────────────────────────────────┘   │
│                              │                                   │
│  ┌───────────────────────────┼───────────────────────────────┐  │
│  │                           ▼                               │  │
│  │  💎 插件层 (Plugin Layer) - 可插拔的钻石                   │  │
│  │                                                           │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐         │  │
│  │  │ 工具插件    │ │ UI插件      │ │ 记忆插件    │         │  │
│  │  │ • FileTool  │ │ • Ink UI    │ │ • Session   │         │  │
│  │  │ • BashTool  │ │ • Web UI    │ │ • AutoMem   │         │  │
│  │  │ • GitTool   │ │ • VS Code   │ │ • Dream     │         │  │
│  │  │ • AgentTool │ │             │ │             │         │  │
│  │  │ • ... (40+) │ │             │ │             │         │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘         │  │
│  │                                                           │  │
│  │  ┌─────────────┐ ┌─────────────┐ ┌─────────────┐         │  │
│  │  │ LLM插件     │ │ 同步插件    │ │ 安全插件    │         │  │
│  │  │ • Claude    │ │ • P2P Sync  │ │ • Sandbox   │         │  │
│  │  │ • OpenAI    │ │ • Cloud     │ │ • Permission│         │  │
│  │  │ • Ollama    │ │ • Git       │ │ • Audit     │         │  │
│  │  │ • Local     │ │             │ │             │         │  │
│  │  └─────────────┘ └─────────────┘ └─────────────┘         │  │
│  │                                                           │  │
│  └───────────────────────────────────────────────────────────┘  │
│                              │                                   │
│  ┌───────────────────────────┼───────────────────────────────┐  │
│  │                           ▼                               │  │
│  │  💎 个性化层 (Personalization) - 你的专属钻石              │  │
│  │  • 个人代码库索引 (HNSW + 全文)                            │  │
│  │  • 编码风格学习 (本地微调模型)                              │  │
│  │  • 习惯分析引擎 (时间/模式/偏好)                            │  │
│  │  • 知识图谱 (实体关系网络)                                  │  │
│  └───────────────────────────────────────────────────────────┘  │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 📦 功能模块详解

### 1. 核心底座 (Core Kernel)
**性质**: 永不裁剪，所有功能的基础

| 组件 | 技术 | 说明 |
|:---|:---|:---|
| QueryEngine | Rust + async | 查询编排、流式响应、工具调度 |
| ToolRegistry | Rust | 插件注册、热加载、依赖管理 |
| EventLoop | Tokio | 异步事件处理 |
| ConfigManager | Rust + TOML/JSON | 模块化配置、Feature Flag |

---

### 2. 工具系统 (40+ 工具，全部可开关)

```rust
pub trait Tool: Send + Sync {
    fn name(&self) -> &str;
    fn description(&self) -> &str;
    fn execute(&self, input: ToolInput) -> ToolOutput;
    fn permissions(&self) -> Vec<Permission>;
    fn is_enabled(&self) -> bool;  // ⭐ 关键：可开关
}
```

#### 2.1 文件操作工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `read_file` | 读取文件内容 | ✅ |
| `write_file` | 写入文件 | ✅ |
| `edit_file` | 编辑文件（diff应用） | ✅ |
| `delete_file` | 删除文件 | ✅ |
| `view_image` | 查看图片（终端渲染） | ❌ |
| `list_directory` | 列出目录 | ✅ |
| `glob` | 文件模式匹配 | ✅ |
| `find` | 递归查找 | ✅ |

#### 2.2 搜索工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `grep_files` | 文本搜索 | ✅ |
| `grep_code` | 代码语义搜索 | ✅ |
| `semantic_search` | 向量相似搜索 | ✅ |
| `symbol_search` | 符号搜索（LSP） | ❌ |

#### 2.3 终端工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `bash` | Bash命令执行 | ✅ |
| `powershell` | PowerShell执行 | ✅ |
| `exec` | 通用执行 | ✅ |
| `script` | 脚本执行（多行） | ✅ |

#### 2.4 Git工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `git_status` | 状态查看 | ✅ |
| `git_diff` | 差异比较 | ✅ |
| `git_log` | 日志查看 | ✅ |
| `git_commit` | 自动提交 | ✅ |
| `git_branch` | 分支管理 | ✅ |

#### 2.5 代码智能工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `lsp_query` | LSP查询 | ❌ |
| `go_to_definition` | 跳转到定义 | ❌ |
| `find_references` | 查找引用 | ❌ |
| `type_check` | 类型检查 | ❌ |

#### 2.6 网络工具（可选）
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `web_search` | 网页搜索 | ❌ |
| `fetch_url` | 获取URL内容 | ❌ |
| `api_request` | API调用 | ❌ |

#### 2.7 代理工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `spawn_agent` | 创建子代理 | ❌ |
| `close_agent` | 关闭子代理 | ❌ |
| `send_input` | 向代理发送输入 | ❌ |

#### 2.8 MCP工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `mcp_invoke` | 调用MCP工具 | ✅ |
| `mcp_resource` | 获取MCP资源 | ✅ |
| `mcp_tool` | MCP工具包装 | ✅ |

#### 2.9 构建/测试工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `npm_run` | npm脚本 | ✅ |
| `cargo_build` | Cargo构建 | ✅ |
| `make` | Make构建 | ❌ |
| `cmake` | CMake构建 | ❌ |
| `run_tests` | 运行测试 | ✅ |
| `coverage` | 覆盖率分析 | ❌ |
| `benchmark` | 性能基准 | ❌ |

#### 2.10 文档工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `generate_docs` | 生成文档 | ❌ |
| `update_readme` | 更新README | ❌ |

#### 2.11 编辑工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `apply_patch` | 应用补丁 | ✅ |
| `multi_file_edit` | 多文件编辑 | ✅ |
| `refactor` | 自动重构 | ❌ |

#### 2.12 分析工具
| 工具名 | 功能 | 默认开启 |
|:---|:---|:---:|
| `complexity` | 复杂度分析 | ❌ |
| `dependency_graph` | 依赖图生成 | ❌ |
| `security_audit` | 安全审计 | ❌ |

---

### 3. UI系统（多界面，想用哪个用哪个）

| UI模式 | 技术 | 场景 | 默认开启 |
|:---|:---|:---|:---:|
| **终端UI** | Ink + React | 日常使用 | ✅ |
| **Web UI** | React + WebSocket | 大屏幕/远程 | ❌ |
| **VS Code插件** | TypeScript + LSP | IDE内嵌 | ❌ |
| **API模式** | HTTP/REST | 脚本调用 | ❌ |

---

### 4. 记忆系统（5层豪华版）

```
Layer 1: Session Memory (内存)
   └── 当前对话，无限长度（靠压缩）
   └── 自动维护，重启清空

Layer 2: Auto Memory (本地文件)
   └── ~/.hajimi/memory/{project}/
   └── 自动提取的偏好、决策、工作流
   └── 跨session持久化

Layer 3: Dream Consolidation (本地后台)
   └── 定期整理、去重、总结
   └── 解决"记忆越来越乱"问题
   └── 可配置后台进程

Layer 4: Knowledge Graph (图数据库)
   └── 实体关系：函数、模块、概念之间的关联
   └── "这个函数被哪些地方调用过？"
   └── 可选：SurrealDB 或嵌入式

Layer 5: External Memory (可选云端)
   └── 可选同步到私有云
   └── 多设备共享（端到端加密）
   └── 默认关闭，需手动开启
```

---

### 5. 上下文压缩（3层 + Cascade可选）

| 层级 | 策略 | 技术实现 | 触发条件 | 可开关 |
|:---|:---|:---|:---|:---:|
| **micro** | 标记替换 | 简单字符串替换 | 每轮自动 | ✅ 强制开启 |
| **auto** | LLM摘要 | Claude API / 本地模型 | Token > 50k | ✅ |
| **compact** | 完整压缩 | LLM摘要 | 用户命令 /compact | ✅ |
| **cascade** | CDC增强 | hajimi-cascade (可选) | 手动开启 | ❌ |

**Cascade说明**: 作为可选功能，需要时在配置中开启。

---

### 6. LLM 插件系统

| 提供商 | 模型 | 用途 | 优先级 |
|:---|:---|:---|:---:|
| **Anthropic** | Claude Sonnet/Opus | 主力对话 | P0 |
| **OpenAI** | GPT-4/GPT-4o | 备选 | P1 |
| **本地** | Ollama (Llama3/Mistral) | 离线模式 | P1 |
| **混合路由** | 自动选择 | 成本优化 | P2 |

---

### 7. 同步系统

| 方式 | 技术 | 场景 | 默认开启 |
|:---|:---|:---|:---:|
| **P2P同步** | WebRTC | 多设备本地同步 | ✅ |
| **Git同步** | Git仓库 | 配置同步 | ❌ |
| **云端同步** | 私有云 | 异地备份 | ❌ |

---

### 8. 安全系统

| 层级 | 技术 | 平台 | 默认开启 |
|:---|:---|:---:|:---:|
| **进程隔离** | Landlock | Linux | ✅ |
| **进程隔离** | Seatbelt | macOS | ✅ |
| **进程隔离** | Windows Sandbox | Windows | ❌ |
| **权限系统** | deny/ask/allow | 全平台 | ✅ |
| **审计日志** | 本地SQLite | 全平台 | ✅ |
| **代码扫描** | 内置规则 | 全平台 | ❌ |

---

### 9. 个人知识库（终极功能）

```rust
pub struct PersonalKnowledgeBase {
    // 代码索引
    code_index: HNSWIndex<CodeSnippet>,      // 语义搜索
    fulltext_index: TantivyIndex,            // 全文搜索
    
    // 经验记录
    debug_logs: Vec<DebugExperience>,        // 调试经验
    adr_records: Vec<ArchitectureDecision>,  // 架构决策
    
    // 习惯学习
    coding_style: CodingStyleModel,          // 编码风格
    time_patterns: TimePatternAnalysis,      // 时间模式
    
    // 知识图谱
    entity_graph: Graph<Entity, Relation>,   // 实体关系
}
```

**功能**:
- "我以前怎么解决过类似问题？"
- "这个函数都在哪里被调用？"
- "我最近都在写什么类型的代码？"
- "根据我的习惯，这个函数应该怎么命名？"

---

## 📅 实施路线图（18个月）

### Phase 1: 核心底座（3个月）
```
目标: 可扩展的插件架构
├── Week 1-2: QueryEngine 核心流程
├── Week 3-4: ToolRegistry + 插件系统
├── Week 5-6: ConfigManager + Feature Flag
├── Week 7-8: 5个核心工具（文件/终端/搜索）
├── Week 9-10: Ink UI 基础
├── Week 11-12: 集成测试 + 文档
```

### Phase 2: 工具全家桶（4个月）
```
目标: 40+工具全部实现
├── Month 1: 文件/目录/搜索工具 (8个)
├── Month 2: Git/终端/构建工具 (10个)
├── Month 3: LSP/代码智能工具 (8个)
├── Month 4: MCP/代理/高级工具 (15个)
```

### Phase 3: UI全家桶（3个月）
```
目标: 多界面支持
├── Month 1: Ink终端UI完善
├── Month 2: Web UI (React)
├── Month 3: VS Code插件 + API服务器
```

### Phase 4: 记忆与知识（4个月）
```
目标: 个人知识库
├── Month 1: 5层记忆系统
├── Month 2: 上下文压缩 (3层 + Cascade)
├── Month 3: 代码库索引 (HNSW)
├── Month 4: 经验记录 + ADR系统
```

### Phase 5: 智能化（4个月）
```
目标: 深度学习
├── Month 1: 编码风格学习
├── Month 2: 习惯分析引擎
├── Month 3: 预测性建议
├── Month 4: 个人模型微调 (LoRA)
```

---

## ⚙️ 配置示例

### 最小配置（轻量版）
```json
{
  "version": "2.0",
  "enabled_features": [
    "ui:terminal",
    "tools:file",
    "tools:bash",
    "tools:git",
    "tools:search",
    "memory:session",
    "llm:anthropic",
    "compression:micro"
  ]
}
```

### 完整配置（豪华版）
```json
{
  "version": "2.0",
  "enabled_features": [
    "ui:terminal",
    "ui:web",
    "ui:vscode",
    "tools:all",
    "memory:all",
    "llm:anthropic",
    "llm:openai",
    "llm:local",
    "compression:all",
    "sync:p2p",
    "sync:git",
    "security:sandbox",
    "personalization:knowledge_base"
  ]
}
```

### 你的专属配置
```json
{
  "version": "2.0",
  "enabled_features": [
    // 根据你的需求定制
  ]
}
```

---

## 📁 本目录文件说明

| 文件 | 说明 |
|:---|:---|
| `README.md` | 本文件，总览文档 |
| `phase-*.md` | 各Phase详细设计文档（待创建） |
| `architecture/` | 架构设计图（待创建） |
| `checklist/` | 开发检查清单（待创建） |

---

*本文档为 HAJIMI CLI 豪华镶钻版开发路线图，随项目进展持续更新。*
