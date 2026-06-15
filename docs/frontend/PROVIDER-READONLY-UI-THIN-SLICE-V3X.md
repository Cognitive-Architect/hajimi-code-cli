# STONE-AUDIT-V3X-DAY05 Provider Read-only UI Thin Slice

Date: 2026-06-15

## Scope

After Phase A passed, add a read-only Provider display helper split across service/view/controller files.

Human summary: this creates a small display-only helper for Provider rows. It is like printing labels for jars on a shelf; it does not open the jars, change the contents, or throw anything away.

## Branch / Baseline

- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `2bef3da82b9e9663dcda974c326d34b1a13a57ad`
- Phase A result: PASS
- Phase B result: PASS for Node helper coverage

## Modified Files

- `src/interface/web/controllers/provider-controller.js`
- `src/interface/web/services/provider-service.js`
- `src/interface/web/views/provider-view.js`
- `tests/frontend/day36_provider_readonly_dom_smoke.js`
- `docs/frontend/PROVIDER-READONLY-DOM-SMOKE-V3X.md`
- `docs/frontend/PROVIDER-READONLY-UI-THIN-SLICE-V3X.md`

## Extracted Read-only Helpers

### `HajimiProviderService`

Exposes:

```text
normalizeProviderConfig(config)
normalizeProviderConfigs(configs)
formatProviderMeta(config)
getProviderSource(workspacePath)
```

Behavior:

- normalizes display fields only
- supports `providerType` / `provider_type`
- supports `baseUrl` / `base_url`
- supports `hasSavedKey` / `hasApiKey` / `has_api_key`
- never includes `apiKey`
- never calls Tauri
- never touches keyring

### `HajimiProviderView`

Exposes:

```text
renderProviderListReadOnly(app, options)
```

Behavior:

- uses `document.createElement`
- uses `textContent`
- uses `appendChild`
- does not use `innerHTML`
- renders `.provider-item`, `.provider-item-info`, `.provider-item-name`, `.provider-item-meta`, `.provider-source-tag`, `.provider-source-hint`
- renders no edit/delete buttons
- safely no-ops when `providerListTab` is missing

### `HajimiProviderController`

Exposes:

```text
renderProviderListReadOnly(app, options)
```

Behavior:

- delegates to `HajimiProviderView`
- returns `{ rendered, count, readonly: true }`
- does not perform Provider writes
- does not invoke backend commands

## Browser Wiring Status

Current browser wiring: NOT WIRED.

This task does not modify `index.html` and does not modify `app.js`. Existing production Provider flows remain in `app.js` and still own save/delete/probe/backup/modal behavior.

Reason: the task allowed a read-only UI thin slice, but explicitly forbids Provider write/keyring/probe/backup behavior. Browser wiring should be a separate task after a dedicated WebView Provider read-only receipt.

## Validation

```text
node --check src/interface/web/controllers/provider-controller.js: PASS
node --check src/interface/web/services/provider-service.js: PASS
node --check src/interface/web/views/provider-view.js: PASS
node --check tests/frontend/day36_provider_readonly_dom_smoke.js: PASS
node tests/frontend/day36_provider_readonly_dom_smoke.js: PASS
node tests/frontend/day19_settings_smoke.js: PASS
node tests/frontend/day34_model_picker_smoke.js: PASS
```

Forbidden diff command:

```powershell
git diff --name-only -- src/interface/desktop src/engine/tool-system src/interface/web/app.js src/interface/web/index.html src/interface/desktop/tauri.conf.json
```

Result:

```text
no output
```

## Security Gate

```text
npm run test:security-gate
Security Audit Gate V1 summary
findings: 97
failures: 3
warnings: 94
allowlisted: 94
```

Result: FAIL, known pre-existing baseline. No Provider helper file appears in the three failures.

Existing failures:

```text
src/interface/web/views/command-palette-view.js:36
src/interface/web/views/session-list-view.js:22
src/interface/web/views/session-list-view.js:26
```

## Read-only / Write-risk Classification

| Area | Status | Notes |
|---|---|---|
| Provider display formatting | READ-ONLY | Moved into `provider-service.js`. |
| Provider list read-only render | READ-ONLY | New `provider-view.js`, safe DOM only. |
| Provider controller delegation | READ-ONLY | New `renderProviderListReadOnly`. |
| Provider save/update | FORBIDDEN | Not moved, not called. |
| Provider delete | FORBIDDEN | Not moved, not called. |
| Provider validation/test | FORBIDDEN | Not moved, not called. |
| Provider probe | FORBIDDEN | Not moved, not called. |
| Provider backup import/export | FORBIDDEN | Not moved, not called. |
| Keyring | FORBIDDEN | Not touched. |
| Agent provider binding | FORBIDDEN | Not touched. |

## WebView

WebView Provider UI smoke: NOT RUN.

This is not a real browser wiring proof and not a keyring safety proof. It is a Node-only helper and DOM smoke receipt.

## Old Dirty Files

Old dirty files remain in the worktree and were not staged for this task.

## Next

Recommended next task:

```text
Provider read-only browser wiring sampling: prove provider-view/controller scripts can be loaded without invoking save/delete/probe/backup/keyring, then decide whether to wire read-only list only.
```
