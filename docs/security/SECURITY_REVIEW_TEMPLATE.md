# Security Review Report Template

> Use this template for local Hajimi security reviews. Do not mark inferred
> issues as confirmed without validation evidence.

## 1. Scope

- Repository:
- Branch:
- Commit:
- Reviewer / Agent:
- Scan commands:
- Generated at:
- In scope:
- Out of scope:

## 2. Executive Summary

| Severity | Confirmed | Unverified | Accepted Risk | Fixed |
|---|---:|---:|---:|---:|
| Critical | 0 | 0 | 0 | 0 |
| High | 0 | 0 | 0 | 0 |
| Medium | 0 | 0 | 0 | 0 |
| Low | 0 | 0 | 0 | 0 |

Summary:

- Overall status:
- Gate status:
- Main residual risk:

## 3. Threat Model

| Area | Notes |
|---|---|
| Assets | |
| Entry points | |
| Trust boundaries | |
| High-risk operations | |
| Assumptions | |

## 4. Findings

### FINDING-001: Title

- Severity:
- Status: `candidate | unverified | confirmed | fixed | accepted_risk | false_positive`
- Category:
- Rule ID:
- Confidence:
- File:
- Line:
- Evidence:
- Attack path narrative:
- Recommendation:
- Regression test:
- Human review required:

Evidence table:

| Kind | File | Line | Command | Summary |
|---|---|---:|---|---|
| code | | | | |

## 5. Validation Receipts

| Command | Exit Code | Status | Output Summary |
|---|---:|---|---|
| | | `pass | fail | not_run | pending` | |

## 6. Fix / Revalidation

- Patch plan:
- Dry-run:
- Files affected:
- Rollback plan:
- Revalidation commands:
- Revalidation status:

## 7. Residual Risk

- PENDING:
- NOT_FOUND:
- Accepted risk:
- Manual WebView / click validation debt:

## 8. Safety Boundaries

- 不自动执行真实 exploit。
- 不做外部联网扫描。
- 高危修复必须人工确认。
- 未验证 finding 不得标记 confirmed。
- 未复测 fix 不得标记 fixed。
- Report snippets must use safe text rendering in UI; do not inject finding
  snippets through unsafe `innerHTML`.
