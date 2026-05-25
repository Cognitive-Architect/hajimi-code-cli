# Skill: Project Handoff (项目交接归档)

> **Version**: 0.1.0  
> **Status**: Template / Skeleton (Disabled by default)  

## 1. 技能定位与核心目标 (Target & Core Goals)
旨在当智能体完成大规模重构、多模块协作开发或阶段性任务闭环时，生成结构化、高可读性、完全确定的 `=== PROJECT HANDOFF ===` 交接小票。

## 2. 核心版式要求 (Standard Template Format)
生成归档时，必须在输出末尾追加以下标准格式块：

```text
=== PROJECT HANDOFF ===
目标:
- [简述本阶段的核心交付目标]

状态:
- [列出当前已完成的功能点与验证通过情况]

风险:
- [诚实列出当前残留的未决缺陷、技术债务或环境依赖]

下一步:
- [清晰指出下一阶段建议的开发方向与动作]
=======================
```

## 3. 技术约束与红线 (Technical Constraints & Redlines)
- **数据真实**: 目标与状态中的测试结果、SHA 必须来自实机命令，禁止伪造。
- **杜绝占位符**: 绝不能出现 `<请填写目标>` 或类似 placeholder 标记。
- **杜绝 mock**: 禁止在交付结果中出现任何欺骗性的 simulation 或 mock 字样。
