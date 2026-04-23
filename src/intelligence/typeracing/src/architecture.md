# TypeRacing 架构设计文档

## 1. 概述

TypeRacing 是一个基于 LSP 工具的类型预测引擎，通过复用 `tool-system` 的 LSP 客户端能力，实现智能代码补全和类型推断。

## 2. 算法流程

### 2.1 初始化流程
1. 接收 LSP 连接配置（stdio/tcp）
2. 调用 `tool_system::lsp` 初始化 LSP 客户端
3. 建立类型预测树根节点

### 2.2 类型预测流程
1. 解析当前光标位置和上下文
2. 调用 `lsp.hover` 获取类型信息
3. 调用 `lsp.definition` 解析类型定义
4. 构建类型预测树分支
5. 返回排序后的类型候选列表

### 2.3 引用分析流程
1. 调用 `lsp.references` 获取符号引用
2. 分析引用模式推断类型约束
3. 更新预测树权重

## 3. LSP 工具集成路径

```
TypeRacing Engine
    ├── lsp.init()      → tool_system::LspInitTool
    ├── lsp.hover()     → tool_system::LspHoverTool
    ├── lsp.definition() → tool_system::LspDefinitionTool
    └── lsp.references() → tool_system::LspReferencesTool
```

## 4. 类型预测树设计

### 4.1 TypeTree 结构
- 根节点：当前编辑位置
- 分支节点：可能的类型路径
- 叶子节点：最终类型候选

### 4.2 PredictionNode 属性
- 类型名称
- 置信度分数
- 来源标记（LSP/启发式/历史）
- 子节点列表

## 5. 约束合规说明

- ✅ 复用 `engine/tool-system/src/lsp.rs` 的 LSP 工具
- ❌ 禁止自建语言服务
- ❌ 禁止独立 LSP 客户端实现
- ❌ 禁止 `std::process::Command` 启动语言服务
