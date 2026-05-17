# Hajimi Debt Documentation Index

> Updated: 2026-05-17
> Current source of truth: `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md`

## Active Summary

Use this file first:

- `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md`

B16 update suggestion:

- `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17-B16-D06-SUGGESTED.md`

## Active Debt Documents

| Document | Current status |
|---|---|
| `02-slash-command-palette.md` | Slash command suggestion panel remains open. |
| `DEBT-B16-SLASH-SAFETY-REMEDIATION.md` | B16 Slash Palette and Security Audit Gate receipt; suggests `AD-007 IMPLEMENTED/PENDING-UI-SMOKE`, `AD-004 PARTIAL/IMPROVED`, and `AD-008 PARTIAL/GATED`. |
| `DEBT-AGENT-PROMPT-001.md` | Agent Prompt V2 is partial; contracts/golden exist, productization remains. |
| `DEBT-FRONTEND-B13-UI-SMOKE-BLOCKED.md` | Tauri/WebView UI smoke remains blocked. |
| `DEBT-P0-UI-INTERACTION-REMEDIATION.md` | Frontend modularization and global Tauri migration remain partial. |
| `DEBT-THINKING-UI.md` | Thinking UI and checkpoint V1 exist; richer diff, transaction restore, and WebView smoke remain. |
| `DEBT-UX-AGENT-001.md` | Startup/filetree/session behavior is verify-stage and needs GUI smoke. |
| `DEBT-UX-B07-001-TAURI-DEV-SMOKE-BLOCKED.md` | Tauri dev smoke blocker remains active. |
| `SHELL-FEATURE-DEBT-002.md` | Complex shell features remain intentionally downgraded. |

## Archive

Cleared, inactive, or superseded debt documents are stored in:

- `archive/05/debt-history`

Notable archived groups:

- Token/context tracking history.
- Agent Core historical debt declarations.
- Signaling PSK inactive debt record.
- Day 1-15 debt-remediation receipts.
- DOM/CSP/file-ops/checkpoint/frontend-module verification receipts.
- Superseded 2026-05-15 current-status snapshot.

## Maintenance Rule

When a debt changes status, update `docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md` first. If the debt is truly cleared, move the old debt or receipt document to `archive/05/debt-history` and record the archive reason in the active summary.
