# 12-DEBT-DISCOVERY-债务遗漏清单

> **审计日期**: 2026-02-26  
> **审计范围**: Termux环境代码全量扫描  
> **发现方式**: 建设性审计探索（task-audit/10.md）

---

## 概要

本次审计发现 **3项未声明债务**，均为 **P2级别（非阻塞）**，不影响当前版本发布，建议纳入v2.0排期。

| 分类 | 数量 | 最高级别 |
|:-----|:----:|:--------:|
| 配置管理 | 1 | P2 |
| 访问控制 | 1 | P2 |
| 代码债务 | 1 | P2 |
| **总计** | **3** | **P2** |

---

## 债务详情

### DEBT-AUDIT-001: 缺少.gitignore版本控制保护

| 属性 | 内容 |
|:-----|:-----|
| **级别** | P2 |
| **类型** | 配置管理 |
| **位置** | 项目根目录 |
| **发现方式** | `ls -la` 检查 |

**问题描述**：
项目根目录缺少 `.gitignore` 文件，以下敏感文件/目录可能意外提交至版本控制：
- `.env` / `.env.local` / `.env.*.local` - 环境变量与凭据
- `node_modules/` - 依赖目录
- `*.log` / `logs/` - 日志文件
- `temp/` - 临时文件
- `.hajimi/` - 本地存储数据

**风险**：
- 敏感凭据（如有）可能泄露至Git历史
- 仓库体积膨胀（node_modules可能很大）

**修复方案**：
```gitignore
# Dependencies
node_modules/
package-lock.json

# Environment
.env
.env.local
.env.*.local

# Logs
*.log
logs/

# Runtime
temp/
*.tmp

# Storage
.hajimi/
*.db

# OS
.DS_Store
Thumbs.db
```

**工时**: 10分钟  
**排期建议**: v2.0

---

### DEBT-AUDIT-002: HTTP CORS默认配置过宽

| 属性 | 内容 |
|:-----|:-----|
| **级别** | P2 |
| **类型** | 访问控制 |
| **位置** | src/api/server.js:153 |
| **发现方式** | 代码审查 |

**问题代码**：
```javascript
// src/api/server.js:152-155
res.setHeader('Access-Control-Allow-Origin', '*');
res.setHeader('Access-Control-Allow-Methods', 'GET, POST, PUT, DELETE, OPTIONS');
res.setHeader('Access-Control-Allow-Headers', 'Content-Type');
```

**风险描述**：
- `Access-Control-Allow-Origin: *` 允许任意网页通过浏览器访问API
- 在Termux局域网场景下（手机热点/WiFi），API可能暴露给同一网络的其他设备
- 恶意网页可能利用此配置进行CSRF攻击（虽受限于Termux使用场景，风险较低）

**修复方案**：
```javascript
// 推荐方案：可配置化
constructor(options = {}) {
  // ...
  this.corsOrigin = options.corsOrigin || 'http://localhost:3000';
  // 生产环境可配置为特定域名
}

_handleRequest(req, res) {
  // ...
  const origin = req.headers.origin;
  const allowedOrigins = Array.isArray(this.corsOrigin) 
    ? this.corsOrigin 
    : [this.corsOrigin];
    
  if (allowedOrigins.includes('*') || allowedOrigins.includes(origin)) {
    res.setHeader('Access-Control-Allow-Origin', origin || '*');
  }
  // ...
}
```

**工时**: 30分钟  
**排期建议**: v2.0

---

### DEBT-AUDIT-003: TODO标记待处理

| 属性 | 内容 |
|:-----|:-----|
| **级别** | P2 |
| **类型** | 代码债务 |
| **位置** | 2处 |
| **发现方式** | `grep -r "TODO" src/` |

**TODO列表**：

| 文件 | 行号 | TODO内容 | 建议处理 |
|:-----|:----:|:---------|:---------|
| src/vector/hnsw-core.js | 278 | `// TODO: 实现简单的多样性启发式` | 评估是否影响召回率，如影响则v2.0实现 |
| src/vector/hybrid-retriever.js | 待确认 | `// TODO: 从持久化存储重新加载` | 确认是否已实现，如已实现则删除TODO |

**修复方案**：
1. 评估两处TODO的实际影响
2. 如影响功能，创建正式债务卡片并排期
3. 如已实现或无需实现，删除TODO标记并添加注释说明

**工时**: 30分钟（评估+处理）  
**排期建议**: v2.0

---

## 债务矩阵

```
严重性
   ▲
P0 │ 
   │ 
P1 │ 
   │ █ DEBT-AUDIT-001 (.gitignore)
P2 │ █ DEBT-AUDIT-002 (CORS)
   │ █ DEBT-AUDIT-003 (TODO)
   └──────────────────────────────►
      配置    访问    代码
      管理    控制    债务
```

---

## 与已知债务对比

| 已知债务ID | 描述 | 状态 | 本审计是否重复 |
|:-----------|:-----|:----:|:-------------:|
| DEBT-PHASE2-001 | WASM优化 | ⚠️ 85% | 否 |
| DEBT-PHASE2-002 | SimHash→Dense编码损失 | ✅ 已缓解 | 否 |
| DEBT-PHASE2-003 | Termux内存限制 | ✅ 已缓解 | 否 |
| DEBT-PHASE2-004 | Worker Thread | ✅ 已清偿 | 否 |

**结论**：本次发现的3项债务均为新增，无重复。

---

## 建议排期

| 债务ID | 建议版本 | 负责人 | 优先级 |
|:-------|:---------|:-------|:------:|
| DEBT-AUDIT-001 | v2.0 | Engineer | P2-高 |
| DEBT-AUDIT-002 | v2.0 | Engineer | P2-中 |
| DEBT-AUDIT-003 | v2.0 | Engineer | P2-低 |

---

*审计官：Mike*  
*日期：2026-02-26*
