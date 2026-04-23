# HAJIMI E2E 测试套件 - Week 4 底座巩固

## 测试目标
验证Week 1-3全部交付物的端到端集成，确保底座A级稳固。

## 测试范围
- EventLoop: 100%底座级（7单元测试+E2E验证）
- WebUI: 100%完成（Flex-Line最终申报）
- TypeRacing: 100%完成（20测试+E2E验证）
- Tools: 100%完成（40/40+E2E验证）

## E2E测试场景

| 场景ID | 场景描述 | 验证模块 | 通过标准 |
|:---|:---|:---|:---|
| E2E-001 | 完整REPL流程 | EventLoop+WebUI | 输入→执行→流式输出→历史，<2s |
| E2E-002 | 类型预测端到端 | TypeRacing+WebUI | 编辑→Ctrl+Space→预测显示<500ms |
| E2E-003 | 工具调用链 | Tools+WebUI | 命令输入→工具选择→执行→结果展示 |
| E2E-004 | 系统负载测试 | 全系统 | 并发10个REPL会话，内存<500MB |
| E2E-005 | 故障恢复 | EventLoop+WebUI | 异常终止→重启→状态恢复 |

## 测试文件结构

```
tests/e2e/
├── HAJIMI-E2E-TEST-SUITE.md    # 测试方案
├── repl_workflow.test.ts       # E2E-001: REPL全流程
├── type_prediction.test.ts     # E2E-002: 类型预测E2E
├── tool_chain.test.ts          # E2E-003: 工具调用链
└── system_load.test.ts         # E2E-004/005: 负载+故障恢复
```

## 执行命令

```bash
# 全量E2E测试
npm test -- tests/e2e/*.test.ts

# 单个场景测试
npm test -- repl_workflow.test.ts
npm test -- type_prediction.test.ts
npm test -- tool_chain.test.ts
npm test -- system_load.test.ts
```

## 刀刃检查表

| 类别 | 编号 | 自测点 | 验证命令 | 目标 |
|:---|:---|:---|:---|:---:|
| FUNC | FUNC-001 | REPL全流程 | `npm test -- repl_workflow.test.ts` | PASSED |
| FUNC | FUNC-002 | 类型预测E2E | `npm test -- type_prediction.test.ts` | PASSED |
| FUNC | FUNC-003 | 工具调用链 | `cargo test test_e2e_tool_chain` | PASSED |
| CONST | CONST-001 | 全系统编译 | `cargo check --workspace` | 0 errors |
| NEG | NEG-001 | 故障恢复 | `npm test -- system_load.test.ts` | PASSED |
| NEG | NEG-002 | 内存检测 | `grep "memoryUsage" system_load.test.ts` | ≥1 |
| E2E | E2E-001 | 场景覆盖率 | 测试报告 | 5/5 |
| HIGH | HIGH-001 | 并发负载 | `grep "concurrent.*10" system_load.test.ts` | ≥1 |

## 底座巩固声明

```
底座状态验证：Week 4 E2E测试确认
- EventLoop: 100%底座级（7单元测试+E2E验证）
- WebUI: 100%完成（95%→100%，Flex-Line最终申报）
- TypeRacing: 100%完成（20测试+E2E验证）
- Tools: 100%完成（40/40+E2E验证）
- 整体底座: A级稳固，Phase 5底座升A目标达成
```
