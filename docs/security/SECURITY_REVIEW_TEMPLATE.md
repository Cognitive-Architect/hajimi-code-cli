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
- Contract version:
- Feature gates:

## 3. Threat Model Summary

| Area | Notes |
|---|---|
| Assets | |
| Entry points | |
| Trust boundaries | |
| High-risk operations | |
| Assumptions | |

This template supports V1 local gate/report rendering. It must not be presented
as a complete security audit or full SAST result.

## 4. Findings

### FINDING-001: Title

- Finding ID:
- Rule ID:
- Severity:
- Status: `candidate | unverified | confirmed | fixed | accepted_risk | false_positive`
- Category:
- Confidence:
- Type:
- File:
- Line:
- Snippet:
- Evidence:
- Attack path narrative:
- Recommendation:
- Regression test:
- Human review required:
- Validation receipts:
- Residual risk:

Evidence table:

| Kind | File | Line | Snippet | Command | Output Hash | Note |
|---|---|---:|---|---|---|---|
| code | | | | | | |

## 5. Validation Receipts

| Command | Exit Code | Status | Stdout Summary | Stderr Summary |
|---|---:|---|---|---|
| | | `pass | fail | not_run | pending` | | |

## 6. Fix / Revalidation

- Patch plan:
- Dry-run:
- Files affected:
- Risk level:
- Human review required:
- Rollback plan:
- Revalidation commands:
- Revalidation status:

## 7. Residual Risk

- PENDING:
- NOT_FOUND:
- Accepted risk:
- Manual WebView / click validation debt:
- Findings without evidence:
- Findings without revalidation:

## 8. Workflow Contract Mapping

| Contract Field | Report Location | Notes |
|---|---|---|
| `finding_id` | Findings | Stable report-local ID |
| `rule_id` | Findings | Rule or detector ID |
| `severity` | Summary and Findings | `low`, `medium`, `high`, `critical` |
| `status` | Summary and Findings | `candidate`, `unverified`, `confirmed`, `fixed`, `accepted_risk`, `false_positive` |
| `confidence` | Findings | `0.0-1.0`; no evidence caps at `0.4` |
| `evidence` | Evidence table | Supports `kind/file/line/snippet/command/output_hash/note` |
| `recommendation` | Findings | Concrete mitigation |
| `regression_test` | Findings and Validation Receipts | Local command only |
| `human_review_required` | Findings and Fix / Revalidation | Required for High/Critical |
| `validation_receipts` | Validation Receipts | Required before `confirmed` / `fixed` |

## 9. Safety Boundaries

- 不自动执行真实 exploit。
- 不做外部联网扫描。
- 高危修复必须人工确认。
- 未验证 finding 不得标记 confirmed。
- 未复测 fix 不得标记 fixed。
- Report snippets must use safe text rendering in UI; do not inject finding
  snippets through unsafe `innerHTML`.
