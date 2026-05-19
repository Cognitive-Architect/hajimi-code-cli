# Hajimi Debt Documentation Index

> Updated: 2026-05-17
> Current source of truth: `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md`

## Active Summary

Use this file first:

- `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md`

## Active Debt Documents

| Document | Current status |
|---|---|
| `DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED.md` | Manual Tauri/WebView UI smoke remains blocked. |
| `DEBT-P0-UI-INTERACTION-REMEDIATION.md` | Frontend modularization and global Tauri API migration remain partial. |
| `DEBT-THINKING-UI.md` | Thinking UI and checkpoint V1 exist; richer diff, transaction restore, malformed stream tests, provider-token integration, and WebView smoke remain. |
| `DEBT-UX-AGENT-001.md` | Startup/filetree/session fixes are code-level complete but need real GUI verification. |
| `DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md` | Tauri dev smoke blocker remains active. |
| `SHELL-FEATURE-DEBT-002.md` | Complex shell features remain intentionally downgraded by design. |

## Archived In Latest Pass

Moved to `archive/05/debt-history`:

- `02-slash-command-palette.md`
- `DEBT-B16-SLASH-SAFETY-REMEDIATION.md`
- `ACTIVE-DEBT-STATUS-2026-05-17-B16-D06-SUGGESTED.md`
- `DEBT-AGENT-PROMPT-001.md`

## Archive

Cleared, inactive, or superseded debt documents are stored in:

- `archive/05/debt-history`

Notable archived groups:

- Token/context tracking history.
- Agent Core historical debt declarations.
- Signaling PSK inactive debt record.
- Slash Palette V1 and B16 Safety Gate receipts.
- Agent Prompt baseline debt superseded by persona/tool-manifest/golden work.
- Day 1-15 debt-remediation receipts.
- DOM/CSP/file-ops/checkpoint/frontend-module verification receipts.
- Superseded 2026-05-15 current-status snapshot.

## Maintenance Rule

When a debt changes status, update `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md` first. If the debt is truly cleared or superseded, move the old debt or receipt document to `archive/05/debt-history` and record the archive reason in the active summary.
