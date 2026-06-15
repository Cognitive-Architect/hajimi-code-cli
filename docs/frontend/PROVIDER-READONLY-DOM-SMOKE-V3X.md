# STONE-AUDIT-V3X-DAY05 Provider Read-only DOM Smoke

Date: 2026-06-15

## Scope

Add a Node-only Provider read-only DOM smoke that proves Provider display shell and read-only list rendering can be exercised without save, delete, probe, backup, validation, keyring, or backend calls.

Human summary: this test only looks at the outside label of the Provider drawer. It does not try keys, save keys, delete keys, test keys, or back up keys.

## Branch / Baseline

- Branch: `stone-audit-v3x-controlled-demolition`
- HEAD before: `2bef3da82b9e9663dcda974c326d34b1a13a57ad`
- Phase A result: PASS

## Files

- `tests/frontend/day36_provider_readonly_dom_smoke.js`
- `docs/frontend/PROVIDER-READONLY-DOM-SMOKE-V3X.md`

## DOM Shell Evidence

`day36_provider_readonly_dom_smoke.js` reads `src/interface/web/index.html` and asserts these Provider shell IDs exist:

```text
providerListTab
providerModal
providerForm
providerName
providerApiKey
saveProvider
testProviderBtn
backupModal
```

These IDs are only checked as DOM shell evidence. The smoke does not click or execute save/test/backup behavior.

## Read-only Assertions

The smoke verifies:

- Provider list display renders mock Provider name/model/base URL.
- Empty Provider state renders readable text.
- Workspace/global source label renders.
- Malicious Provider name/model strings do not execute.
- API key secret text does not render.
- Edit/delete buttons are not created by the read-only render path.
- Missing `providerListTab` safely no-ops.

## Forbidden Provider Calls

The smoke installs throwing spies for:

```text
saveProviderConfig
deleteProviderConfig
exportProviderBackup
importProviderBackup
openBackupModal
invokeTauri
openProviderModal
```

Result:

```text
forbiddenCalls.length: 0
```

It also scans the new Provider helper source for forbidden strings:

```text
invokeTauri
saveProviderConfig
deleteProviderConfig
probe_provider_context_capacity
validate_provider
keyring
```

Result: no forbidden string in the new production helper source.

## Validation

```text
node --check tests/frontend/day36_provider_readonly_dom_smoke.js: PASS
node tests/frontend/day36_provider_readonly_dom_smoke.js: PASS
```

Observed output:

```text
day36 provider readonly dom smoke: PASS (DOM shell + read-only view/controller/service)
```

Regression checks:

```text
node tests/frontend/day19_settings_smoke.js: PASS
node tests/frontend/day34_model_picker_smoke.js: PASS
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

Result: FAIL, known pre-existing baseline. No new Provider failure was introduced.

Existing failures:

```text
src/interface/web/views/command-palette-view.js:36
src/interface/web/views/session-list-view.js:22
src/interface/web/views/session-list-view.js:26
```

## WebView

WebView Provider click smoke: NOT RUN.

Reason: Day05 intentionally uses Node-only read-only DOM smoke plus helper extraction. Provider real UI still contains save/delete/probe/backup/keyring-adjacent controls, and this task forbids exercising those write paths.

## Forbidden Diff

Command:

```powershell
git diff --name-only -- src/interface/desktop src/engine/tool-system src/interface/web/app.js src/interface/web/index.html src/interface/desktop/tauri.conf.json
```

Result:

```text
no output
```

## Old Dirty Files

Old dirty files remain in the worktree and were not staged for this task.
