# Slash Palette V1.5 Slice Plan

> Status: PLAN / READ-ONLY-SAMPLED
> Date: 2026-06-04
> Branch: `feature/toolfix-deepseek-schema`
> Baseline HEAD: `72d26203`
> Scope: static sampling only. No production code changed.

## 0. Scope Guard

This slice is a planning document for the existing Slash Palette surface. It does
not mark any debt as cleared.

Hard exclusions:

- Do not modify `src/interface/web/app.js` in this slice.
- Do not modify `src/interface/web/index.html` in this slice.
- Do not modify `src/interface/web/style.css` in this slice.
- Do not modify `src/interface/desktop/src/main.rs`, `src/engine/tool-system/src/shell.rs`, or `src/interface/desktop/tauri.conf.json`.
- Do not touch Agent streaming, checkpoint restore, provider keyring, shell execution, CSP, or `withGlobalTauri`.

Plain-language summary: this is only a drawer map. It labels where the Slash
Palette drawer sits, which handles open it, and which larger appliances it can
reach. It does not move the drawer.

## 1. Slash-Palette Related Functions

### `src/interface/web/app.js`

| Function / area | Lines sampled | Role | V1.5 note |
|---|---:|---|---|
| `init()` | 51-75 | Calls `setupChat()` during app startup. | Caller only; do not alter startup order in V1.5. |
| `setupChat()` | 2340-2421 | Mounts Slash Palette, wires input/keydown/blur/send events. | Candidate boundary for a small adapter wrapper, but not in this read-only slice. |
| `getSlashCommands()` | 2423-2436 | Returns Slash command catalog. | Best first extraction target: pure data, low risk. |
| `sendChatMessage()` | 2438-2565 | Sends normal chat and routes slash text to `handleChatCommand()`. | Do not extract in V1.5; too close to chat processing and session state. |
| `handleChatCommand(text)` | 2567-2866 | Executes slash commands and maps Chinese aliases. | Do not split execution branches in first V1.5 slice; downstream effects are broad. |
| `invokeAgentTask(goal)` | 3094-3163 | Downstream `/agent` executor. | Explicit no-touch zone. |
| `handleAgentEvent(...)` | 3165-3266 | Agent stream event handling. | Explicit no-touch zone. |
| `streamChat(...)` | 3268+ | Downstream `/chat` executor. | Explicit no-touch zone. |
| `mcpInit(...)` / `mcpInvoke(...)` | 4457-4471 | Downstream `/mcp` helpers. | Explicit no-touch zone. |

### `src/interface/web/modules/slash-palette.js`

| Function | Lines sampled | Role |
|---|---:|---|
| `normalizeItem(item)` | 6-23 | Normalizes command metadata, default risk/mode/enabled flags. |
| `itemMatches(item, query)` | 25-37 | Filters command metadata against trigger/title/description/category/risk/keywords. |
| `clearNode(node)` | 39-43 | Safe DOM clearing without `innerHTML`. |
| `createTextNode(className, text)` | 45-50 | Safe text rendering via `textContent`. |
| `createSlashPalette(options)` | 52-276 | Factory returning the palette API. |
| `loadItems()` | 76-78 | Pulls command list via `getCommands()`. |
| `renderEmpty()` | 80-86 | Renders empty state. |
| `renderItem(item, index)` | 88-132 | Renders one command row and row-level events. |
| `render()` | 134-147 | Rebuilds list DOM. |
| `filterItems(query)` | 149-155 | Filters and resets active item. |
| `getEnabledIndexes()` | 157-161 | Skips disabled rows for navigation. |
| `moveActive(delta)` | 163-173 | Arrow navigation. |
| `selectItem(item)` | 175-180 | Calls `onSelect(item)` then closes. |
| `selectActive()` | 182-186 | Selects current active item. |
| `handleKeyDown(event)` | 188-216 | Handles ArrowUp/ArrowDown/Escape/Enter. |
| `open(query)` | 218-224 | Loads items, shows panel, filters. |
| `close(reason)` | 226-235 | Clears DOM, hides panel, calls `onClose(reason)`. |
| `updateQuery(query)` | 237-243 | Opens or refilters. |
| `handleInput()` | 245-255 | Reads caret token and opens/closes slash panel. |
| `isOpen()` | 257-259 | Public state getter. |
| `destroy()` | 261-263 | Public cleanup. |

## 2. Related DOM IDs / Classes / Dataset

### DOM IDs

| ID | File | Role |
|---|---|---|
| `slashPalette` | `index.html:346` | Mount container passed as `containerEl`. |
| `aiChatInput` | `index.html:348` | Textarea passed as `inputEl`; source of slash token and caret. |
| `aiChatSendBtn` | `index.html:349` | Normal send button, shares `sendChatMessage()` path with slash commands after explicit submit. |

### CSS classes

| Class | Source | Role |
|---|---|---|
| `slash-palette` | `index.html`, `style.css`, module | Container positioning and module-added class. |
| `hidden` | `index.html`, module | Closed state. |
| `slash-palette-list` | module/style | List wrapper. |
| `slash-palette-item` | module/style | Command row button. |
| `active` | module/style | Keyboard-active row. |
| `disabled` | module/style | Disabled command row. |
| `slash-palette-main` | module/style | Trigger/title group. |
| `slash-palette-trigger` | module/style/tests | Trigger text. |
| `slash-palette-title` | module/style | Human-readable title. |
| `slash-palette-description` | module/style | Description text. |
| `slash-palette-meta` | module/style | Category/risk wrapper. |
| `slash-palette-category` | module/style | Category badge. |
| `slash-palette-risk` | module/style | Risk badge. |
| `slash-palette-empty` | module/style | Empty result state. |

### Dataset / ARIA

| Attribute | Location | Role |
|---|---|---|
| `data-command-id` / `row.dataset.commandId` | `slash-palette.js:92` | Stores normalized command id on each row. |
| `role="listbox"` | `slash-palette.js:73` | Container accessibility role. |
| `aria-label="Slash commands"` | `slash-palette.js:74` | Container accessible label. |
| `role="option"` | `slash-palette.js:93` | Row accessibility role. |
| `aria-selected` | `slash-palette.js:94` | Active row state. |
| `aria-disabled` | `slash-palette.js:103` | Disabled row state. |

Downstream note: `/agent` can update `.inspector-tab[data-inspector-tab="agent-trace"]`,
but that is not Slash Palette UI. It is execution fallout and should stay out of
the first V1.5 slice.

## 3. Event Binding Entrypoints

| Event | Bound in | Target | Effect |
|---|---|---|---|
| `input` | `app.js:2363` | `aiChatInput` | Auto-resizes textarea, then calls `slashPalette.handleInput()`. |
| `keydown` | `app.js:2371` | `aiChatInput` | If palette is open, delegates to `handleKeyDown(e)` before Enter send. |
| `blur` | `app.js:2382` | `aiChatInput` | Closes open palette with reason `blur`. |
| `click` | `app.js:2388` | `aiChatSendBtn` | Calls `sendChatMessage()`. |
| `click` | `slash-palette.js:122` | `.slash-palette-item` | Selects enabled row. |
| `mousedown` | `slash-palette.js:127` | `.slash-palette-item` | Prevents focus loss while clicking row. |

Feature flag:

- `window.__HAJIMI_FLAGS__?.slashPaletteEnabled !== false` gates palette creation.
- If `window.HajimiSlashPalette`, `aiChatInput`, or `slashPalette` is missing,
  `setupChat()` silently skips palette creation.

## 4. Call Chain

### Startup / Mount Chain

```text
app.init()
  -> setupChat()
    -> document.getElementById('aiChatInput')
    -> document.getElementById('slashPalette')
    -> window.HajimiSlashPalette.createSlashPalette({
         inputEl,
         containerEl,
         getCommands: () => this.getSlashCommands(),
         onSelect: (...)
       })
    -> this.slashPalette = returned API
```

### Input / Filter Chain

```text
aiChatInput input event
  -> setupChat input handler
    -> this.slashPalette.handleInput()
      -> updateQuery(token) or close('input')
        -> open(query) if closed
          -> loadItems()
            -> getCommands()
              -> app.getSlashCommands()
          -> filterItems(query)
            -> itemMatches(...)
            -> render()
```

### Keyboard Select Chain

```text
aiChatInput keydown event
  -> setupChat keydown handler
    -> if palette open: this.slashPalette.handleKeyDown(e)
      -> ArrowUp/ArrowDown: moveActive(...)
      -> Escape: close('escape')
      -> Enter: selectActive()
        -> selectItem(item)
          -> onSelect(item)
            -> chatInput.value = item.insertText || item.trigger || ''
            -> chatInput.focus()
            -> chatInput.dispatchEvent(new Event('input', { bubbles: true }))
            -> if direct + low risk: this.sendChatMessage()
```

### Explicit Submit / Execution Chain

```text
sendChatMessage()
  -> if text startsWith('/'): handleChatCommand(text)
    -> /agent: invokeAgentTask(goal)                 [no-touch]
    -> /tools: invokeTauri('list_tools')
    -> /providers: reads providerConfigs
    -> /tool: executeToolWithConfirmationRetry(...)  [no-touch in V1.5]
    -> /chat: streamChat(...)                        [no-touch]
    -> /mcp: mcpInit(...) / mcpInvoke(...)           [no-touch]
    -> /search: executeTool('grep', ...)
    -> /git: executeTool('git_status'|'git_diff'|'git_commit', ...)
    -> /extensions: reads extension state
    -> /compact: invoke('optimize_context', ...)
```

## 5. Possible Global State Impact

### Direct palette/mount state

- `slashPalette`: assigned in `setupChat()` and used for open/close/keyboard delegation.

### Chat/composer state touched by selection or slash submit

- `chatInput.value`, `chatInput.style.height`, focus state.
- `isProcessing`: set/reset by `sendChatMessage()`, `/agent`, and downstream flows.
- `chatMessages`: user slash text is pushed before command execution; `/compact` can replace it with summary state.
- `chatSessions` / `activeSessionId`: saved/rendered after command handling.
- `tokenStats`, `cumulativeStats`, `autoCompact`, `isAutoCompacting`: affected by chat/compact paths.
- `chatContextFiles`: context is skipped when text starts with `/`.

### Command-specific state

- `providerConfigs`, `activeProviderId`: `/providers`, `/chat`, `/compact`, `/agent`.
- `mcpServers`: `/mcp init` mutates and persists MCP server state.
- `extensions`, `installedExtensions`: `/extensions` reads extension state.
- `traceEvents`: `/agent` resets and appends trace events.

### High-risk downstream paths to avoid in first implementation slice

- Agent streaming: `/agent` -> `invokeAgentTask()` -> `run_agent_task`.
- Provider/key handling: `/chat` provider config and provider modal paths.
- Tool/shell capability: `/tool`, `/search`, `/git commit`, MCP invoke.
- CSP / `withGlobalTauri`: unrelated to Slash Palette extraction and must remain untouched.

## 6. Rollback File Lists

### This read-only sampling commit

If this report is wrong, rollback only:

```text
docs/frontend/SLASH-PALETTE-V1.5-SLICE-PLAN.md
```

### Future V1.5 implementation candidate

If a future coding agent extracts the pure command catalog or a tiny adapter,
expected rollback files should be limited to:

```text
src/interface/web/app.js
src/interface/web/modules/slash-palette.js
src/interface/web/modules/slash-command-catalog.js       # only if newly added
src/interface/web/index.html                             # only if script tag is added
tests/frontend/day16_slash_palette_smoke.js              # only if coverage is extended
tests/frontend/day21_slash_palette_app_integration_smoke.js # suggested new file
docs/frontend/SLASH-PALETTE-V1.5-SLICE-PLAN.md
docs/debt/STONE-AUDIT-V1-CLOSURE.md                      # only if status notes are updated
```

Do not include:

```text
src/interface/desktop/src/main.rs
src/engine/tool-system/src/shell.rs
src/interface/desktop/tauri.conf.json
```

## 7. Existing Smoke Coverage

Verified in this sampling run:

| Check | Result | Evidence |
|---|---|---|
| Slash Palette module syntax | PASS | `node --check src/interface/web/modules/slash-palette.js` |
| Slash Palette Node smoke | PASS | `node tests/frontend/day16_slash_palette_smoke.js` -> `PASS (8 scenarios)` |

Existing coverage from `tests/frontend/day16_slash_palette_smoke.js`:

1. `/` opens palette and renders commands.
2. `/c` filters to `/compact`.
3. ArrowDown/ArrowUp navigation works and prevents default.
4. Enter selects active command and closes palette.
5. Escape closes palette and preserves input.
6. Disabled command cannot execute by Enter or click.
7. Malicious title/description render as text, not executable HTML.
8. Non-slash input keeps palette closed.

Existing gate/reference coverage:

- `tests/security/security_audit_gate.js` has a fail rule for `innerHTML` usage in `src/interface/web/modules/slash-palette.js`.
- `docs/debt/STONE-AUDIT-V1-CLOSURE.md` records Day 3 `day16_slash_palette_smoke.js` as PASS.

Coverage gaps:

- No app-level smoke currently proves `setupChat()` wires `window.HajimiSlashPalette.createSlashPalette()` correctly.
- No smoke proves `onSelect()` mutates `aiChatInput`, dispatches `input`, and triggers direct low-risk send only for direct+low commands.
- No real Tauri WebView click evidence for Slash Palette interaction in packaged/dev desktop window.
- No smoke covers `window.__HAJIMI_FLAGS__.slashPaletteEnabled === false` fallback.

Do not mark these gaps as cleared.

## 8. Suggested New / Reused Smoke List

### Reuse now

```powershell
node --check src/interface/web/modules/slash-palette.js
node tests/frontend/day16_slash_palette_smoke.js
npm run test:security-gate
```

### Add for V1.5 implementation

Suggested new file:

```text
tests/frontend/day21_slash_palette_app_integration_smoke.js
```

Suggested scenarios:

1. App mount: `setupChat()` creates `this.slashPalette` when module, input, and container exist.
2. Feature flag fallback: `slashPaletteEnabled === false` does not create palette and normal Enter send still works.
3. Selection adapter: selecting `/compact` fills `aiChatInput` with `/compact`, focuses input, dispatches `input`, and does not auto-send because risk is medium/fill.
4. Direct command guard: selecting `/tools` calls `sendChatMessage()` only because it is `executeMode: direct` and `riskLevel: low`.
5. High-risk command guard: selecting `/tool` or `/agent` fills input but does not auto-send.
6. Missing module fallback: if `window.HajimiSlashPalette` is absent, `setupChat()` does not throw.
7. Command catalog parity: extracted catalog still returns `/tools`, `/providers`, `/tool`, `/chat`, `/mcp`, `/search`, `/git`, `/extensions`, `/compact`, `/agent`.
8. Chinese alias remains execution-side only: command catalog extraction must not falsely claim `/代理` is a rendered trigger unless explicitly added and covered.

Optional manual smoke, still PENDING unless actually run:

```text
Tauri WebView: type `/`, filter `/c`, ArrowDown/ArrowUp, Enter, Escape, click row, blur input.
```

## 9. Recommended V1.5 Slice

Recommended first implementation slice:

```text
Extract only the pure Slash command catalog from app.js.
```

Candidate shape:

```text
src/interface/web/modules/slash-command-catalog.js
  - createSlashCommandCatalog()
  - exports CommonJS for Node smoke
  - attaches window.HajimiSlashCommandCatalog for browser usage
```

Minimal app.js change for a future implementation:

```text
getSlashCommands()
  -> if window.HajimiSlashCommandCatalog exists, return catalog from it
  -> otherwise keep current inline fallback during transition
```

Stop before:

- Moving `handleChatCommand()`.
- Moving `/agent`, `/tool`, `/chat`, `/mcp`, `/git`, `/compact` execution branches.
- Changing DOM IDs or CSS.
- Changing security gate allowlists.

Acceptance for future implementation:

- Existing `day16_slash_palette_smoke.js` stays PASS.
- New app integration smoke passes.
- `node --check src/interface/web/app.js` passes.
- `npm run test:security-gate` passes.
- No production status is marked `CLEARED` unless real checks have run and are recorded.

## 10. Sampling Receipts

Commands run for this report:

```text
git status --short --branch
git log --oneline -n 8 --decorate
rg -n -i "slash|palette|command" src/interface/web/app.js
rg -n -i "slash|palette|command" src/interface/web/index.html src/interface/web/style.css src/interface/web/modules tests docs/frontend docs/debt
node --check src/interface/web/modules/slash-palette.js
node tests/frontend/day16_slash_palette_smoke.js
```

Observed baseline:

- Current branch: `feature/toolfix-deepseek-schema`.
- Latest Day 1-3 related commits:
  - `2998c550` Day 1 evidence.
  - `0db0cb5a` Day 2 DOM contract and freeze rules.
  - `fb6d900f` Day 3 closure and verification.
  - `72d26203` closure commit-reference fix.
- Working tree had unrelated existing changes before this report; only this report should be staged for the commit.

## 11. V1.5-B Implementation Record

> Date: 2026-06-04
> Task: `STONE-AUDIT-V1.5-B: Extract Slash Command Catalog`

Implemented scope:

- Added `src/interface/web/modules/slash-command-catalog.js`.
- Added `createSlashCommandCatalog()`.
- Added Node/CommonJS export for smoke tests.
- Added browser mount: `window.HajimiSlashCommandCatalog`.
- Updated `src/interface/web/app.js` so `getSlashCommands()` reads the new module when present and keeps the inline fallback when absent.
- Added `src/interface/web/index.html` script tag for the catalog module before `modules/slash-palette.js` and `app.js`.
- Added `tests/frontend/day21_slash_palette_app_integration_smoke.js`.

Preserved boundaries:

- `handleChatCommand()` was not moved.
- `/agent`, `/tool`, `/chat`, `/mcp`, `/git`, `/compact` execution branches were not moved.
- DOM IDs were not changed.
- CSS was not changed.
- Security gate allowlist was not changed.
- Shell, CSP, `withGlobalTauri`, provider keyring, checkpoint restore, and Agent streaming were not changed.

Validation status:

- Required validation commands are expected to be recorded in the final handoff and commit receipt.
- Tauri WebView slash interaction remains `PENDING-MANUAL-SMOKE`; do not mark it `CLEARED` from this Node smoke.
