# HAJIMI 配置示例集

本文档提供各种使用场景下的配置示例，方便快速启用/禁用功能。

---

## 🔰 最小配置（第一次用）

只保留最核心功能，快速上手。

```json
{
  "version": "2.0",
  "name": "minimal",
  "description": "最小功能集，快速上手",
  "enabled_features": [
    "ui:terminal",
    "tools:read_file",
    "tools:write_file",
    "tools:bash",
    "tools:list_directory",
    "tools:grep_files",
    "memory:session",
    "llm:anthropic",
    "compression:micro"
  ],
  "llm": {
    "default": "claude-sonnet-4.6",
    "api_key": "${ANTHROPIC_API_KEY}"
  },
  "ui": {
    "theme": "dark",
    "syntax_highlight": true
  }
}
```

---

## 🏠 日常使用配置（推荐）

适合大多数开发场景的平衡配置。

```json
{
  "version": "2.0",
  "name": "daily",
  "description": "日常开发配置，功能均衡",
  "enabled_features": [
    "ui:terminal",
    "tools:file",
    "tools:bash",
    "tools:git",
    "tools:search",
    "tools:build",
    "tools:test",
    "memory:session",
    "memory:auto",
    "llm:anthropic",
    "compression:micro",
    "compression:auto"
  ],
  "llm": {
    "default": "claude-sonnet-4.6",
    "fallback": "ollama-llama3",
    "api_key": "${ANTHROPIC_API_KEY}"
  },
  "memory": {
    "auto_extract": true,
    "max_session_history": 100
  },
  "tools": {
    "bash": {
      "timeout": 300000,
      "allowed_commands": ["*"]
    },
    "git": {
      "auto_commit_message": true
    }
  }
}
```

---

## 💎 完整豪华配置（全都要）

启用所有功能，体验完整能力。

```json
{
  "version": "2.0",
  "name": "luxury",
  "description": "豪华完整版，全部功能开启",
  "enabled_features": [
    // UI系统
    "ui:terminal",
    "ui:web",
    "ui:vscode",
    
    // 工具系统（全部）
    "tools:file",
    "tools:bash",
    "tools:git",
    "tools:search",
    "tools:build",
    "tools:test",
    "tools:doc",
    "tools:edit",
    "tools:analyze",
    "tools:network",
    "tools:agent",
    "tools:mcp",
    "tools:lsp",
    
    // 记忆系统（全部）
    "memory:session",
    "memory:auto",
    "memory:dream",
    "memory:graph",
    "memory:cloud",
    
    // LLM（全部）
    "llm:anthropic",
    "llm:openai",
    "llm:local",
    
    // 压缩（全部）
    "compression:micro",
    "compression:auto",
    "compression:compact",
    "compression:cascade",
    
    // 同步
    "sync:p2p",
    "sync:git",
    "sync:cloud",
    
    // 安全（全部）
    "security:sandbox",
    "security:audit",
    "security:scan",
    
    // 个性化
    "personalization:code_index",
    "personalization:style_learning",
    "personalization:habit_analysis",
    "personalization:knowledge_graph"
  ],
  
  "llm": {
    "default": "claude-opus-4.6",
    "fallback_chain": ["claude-sonnet-4.6", "gpt-4", "ollama-llama3"],
    "routing": "auto",
    "api_keys": {
      "anthropic": "${ANTHROPIC_API_KEY}",
      "openai": "${OPENAI_API_KEY}"
    }
  },
  
  "memory": {
    "auto_extract": true,
    "dream_consolidation": {
      "enabled": true,
      "schedule": "0 3 * * *"
    },
    "knowledge_graph": {
      "enabled": true,
      "backend": "surrealdb"
    }
  },
  
  "ui": {
    "theme": "auto",
    "animations": true,
    "transparency": 0.95,
    "web": {
      "enabled": true,
      "port": 8080
    },
    "vscode": {
      "enabled": true
    }
  },
  
  "sync": {
    "p2p": {
      "enabled": true,
      "discovery": "local_network"
    },
    "cloud": {
      "enabled": false,
      "provider": "self_hosted"
    }
  },
  
  "personalization": {
    "code_index": {
      "enabled": true,
      "index_paths": ["~/projects"],
      "exclude_patterns": ["node_modules", "target", ".git"]
    },
    "style_learning": {
      "enabled": true,
      "training_schedule": "weekly"
    }
  }
}
```

---

## ✈️ 离线模式配置

飞机上、咖啡馆网络差时使用。

```json
{
  "version": "2.0",
  "name": "offline",
  "description": "完全离线模式，本地LLM",
  "enabled_features": [
    "ui:terminal",
    "tools:file",
    "tools:bash",
    "tools:git",
    "tools:search",
    "memory:session",
    "memory:auto",
    "llm:local",
    "compression:micro"
  ],
  "llm": {
    "default": "ollama-llama3",
    "local": {
      "provider": "ollama",
      "model": "llama3:70b",
      "temperature": 0.7
    }
  },
  "sync": {
    "enabled": false
  },
  "tools": {
    "network": {
      "enabled": false
    }
  }
}
```

---

## 🔒 超安全模式配置

处理敏感代码时使用。

```json
{
  "version": "2.0",
  "name": "paranoid",
  "description": "最高安全级别，所有操作需确认",
  "enabled_features": [
    "ui:terminal",
    "tools:file",
    "tools:bash",
    "tools:git",
    "memory:session",
    "llm:anthropic",
    "compression:micro",
    "security:sandbox",
    "security:audit"
  ],
  "security": {
    "sandbox": {
      "enabled": true,
      "landlock": true,
      "allowed_paths": ["~/projects/safe"]
    },
    "permissions": {
      "default": "ask",
      "file_write": "ask",
      "bash_exec": "ask",
      "git_commit": "ask"
    },
    "audit": {
      "enabled": true,
      "log_all_operations": true
    }
  },
  "tools": {
    "bash": {
      "timeout": 60000,
      "blocked_commands": ["rm -rf", ">", "curl", "wget"]
    }
  }
}
```

---

## 🚀 高性能模式配置

追求速度，牺牲部分功能。

```json
{
  "version": "2.0",
  "name": "performance",
  "description": "高性能模式，快速响应",
  "enabled_features": [
    "ui:terminal",
    "tools:file",
    "tools:bash",
    "tools:git",
    "tools:search",
    "memory:session",
    "llm:anthropic",
    "compression:micro"
  ],
  "performance": {
    "tool_timeout": 30000,
    "parallel_execution": true,
    "cache_enabled": true,
    "index_on_startup": false
  },
  "llm": {
    "default": "claude-haiku",
    "max_tokens": 4000,
    "temperature": 0.0
  },
  "disabled_features": [
    "memory:auto",
    "memory:dream",
    "compression:auto",
    "personalization:style_learning"
  ]
}
```

---

## 🔧 前端开发专用配置

React/Vue/Angular开发优化。

```json
{
  "version": "2.0",
  "name": "frontend",
  "description": "前端开发专用",
  "enabled_features": [
    "ui:terminal",
    "tools:file",
    "tools:bash",
    "tools:git",
    "tools:search",
    "tools:build",
    "tools:test",
    "tools:lsp",
    "memory:session",
    "memory:auto",
    "llm:anthropic"
  ],
  "tools": {
    "build": {
      "package_managers": ["npm", "yarn", "pnpm"],
      "runners": ["vite", "webpack", "rollup"]
    },
    "lsp": {
      "enabled": true,
      "servers": {
        "typescript": "typescript-language-server",
        "css": "css-languageserver",
        "html": "html-languageserver"
      }
    }
  },
  "prompts": {
    "system": "You are a frontend development expert. Prefer modern ES6+, React hooks, and TypeScript. Always consider responsive design and accessibility."
  }
}
```

---

## ⚙️ 后端开发专用配置

Rust/Go/Node.js后端开发优化。

```json
{
  "version": "2.0",
  "name": "backend",
  "description": "后端开发专用",
  "enabled_features": [
    "ui:terminal",
    "tools:file",
    "tools:bash",
    "tools:git",
    "tools:search",
    "tools:build",
    "tools:test",
    "tools:docker",
    "memory:session",
    "memory:auto",
    "llm:anthropic"
  ],
  "tools": {
    "docker": {
      "enabled": true,
      "compose": true
    },
    "database": {
      "enabled": true,
      "clients": ["psql", "mysql", "redis-cli"]
    }
  },
  "prompts": {
    "system": "You are a backend development expert. Focus on API design, database optimization, and system architecture. Always consider security and scalability."
  }
}
```

---

## 🧪 实验性功能配置

尝鲜最新功能（可能不稳定）。

```json
{
  "version": "2.0",
  "name": "experimental",
  "description": "实验性功能，尝鲜",
  "enabled_features": [
    "ui:terminal",
    "ui:web",
    "tools:all",
    "memory:all",
    "llm:anthropic",
    "llm:local",
    "compression:all",
    "personalization:all"
  ],
  "experimental": {
    "agent_swarm": true,
    "auto_refactor": true,
    "predictive_edits": true,
    "voice_input": false
  }
}
```

---

## 💡 使用建议

1. **从最小配置开始**，熟悉后再逐步开启功能
2. **定期回顾配置**，关闭长期不用的功能
3. **为不同项目创建不同配置**，如 `frontend.json`, `backend.json`
4. **使用环境变量**，敏感信息（API Key）不要硬编码

---

## 📝 配置文件位置

```
~/.config/hajimi/
├── config.json              # 主配置
├── profiles/
│   ├── minimal.json         # 最小配置
│   ├── daily.json           # 日常配置
│   ├── luxury.json          # 豪华配置
│   ├── offline.json         # 离线配置
│   ├── paranoid.json        # 安全配置
│   ├── frontend.json        # 前端专用
│   └── backend.json         # 后端专用
└── local/                   # 本地覆盖（不提交Git）
    └── config.local.json
```

---

*配置文件支持热重载，修改后无需重启即可生效。*
