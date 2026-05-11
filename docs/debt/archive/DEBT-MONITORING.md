# 债务监控体系

## 概述
防止技术债务回归的自动化监控体系

## CI门禁
- 文件: `.github/workflows/debt-gate.yml`
- 触发: PR/push到main分支
- 检查:
  1. 生产代码unwrap = 0
  2. 生产代码expect = 0
  3. unsafe有SAFETY注释

## 本地检查
```bash
./scripts/debt-scan.sh
```

## 预提交钩子
```bash
cp .githooks/pre-commit .git/hooks/
chmod +x .git/hooks/pre-commit
```

## 债务评级
| 评级 | unwrap+expect | unsafe | 状态 |
|:---|:---:|:---:|:---|
| A | 0 | 文档化 | 优秀 |
| B | 0 | 有但未全文档 | 良好 |
| C | <5 | <5 | 可接受 |
| D | ≥5 | ≥5 | 危险 |

## 当前状态
- 评级: B级（清偿后目标）
- 最后更新: 2026-04-10
