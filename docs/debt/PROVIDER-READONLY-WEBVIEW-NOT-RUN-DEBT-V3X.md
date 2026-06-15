# Provider Read-only WebView Verification Debt V3X

Date: 2026-06-15

## Summary

Day05 added Node-only Provider read-only DOM smoke and a read-only UI helper split. It did not run a real Tauri WebView Provider click smoke and did not wire the helper into `index.html` or `app.js`.

Human summary: we made and tested a safe display label maker for Provider rows, but we did not install it into the real desktop window yet.

## Why This Is Debt

Provider UI is keyring-adjacent and contains save/delete/test/probe/backup controls. Day05 explicitly forbids exercising those write paths.

Because of that, this task cannot honestly claim:

- real WebView Provider page PASS
- keyring behavior PASS
- save/delete/probe/backup safety PASS
- production Provider list fully migrated

## Current Evidence

```text
node tests/frontend/day36_provider_readonly_dom_smoke.js: PASS
node tests/frontend/day19_settings_smoke.js: PASS
node tests/frontend/day34_model_picker_smoke.js: PASS
```

The new helper source does not call:

```text
invokeTauri
saveProviderConfig
deleteProviderConfig
probe_provider_context_capacity
validate_provider
keyring
```

## Remaining Work

Future task should add a read-only WebView receipt that:

- loads Provider view/controller/service scripts
- opens the Provider settings tab
- proves Provider list display can render
- does not click save/delete/test/probe/backup
- verifies no provider write/keyring command is invoked

## Boundary

This debt must not be resolved by testing or modifying Provider save/delete/probe/backup/keyring paths in the same task.
