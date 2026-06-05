# STONE-AUDIT-V2B-APPJS-HOTSPOT-SAMPLING

Date: 2026-06-05
Branch: feature/toolfix-deepseek-schema
Baseline HEAD: 25c641ba17b2e24b73e5be5d0bd43e335597cf33

## Scope

Readonly sampling of four `src/interface/web/app.js` hotspot functions:

- `setupProviderSettings`
- `handleChatCommand`
- `streamChat`
- `sendChatMessage`

This report does not modify production code, does not fix any issue, does not add smoke coverage, and does not prove WebView click behavior.

## Function Facts

### `setupProviderSettings()`

- Location: `src/interface/web/app.js:3759`
- Responsibility: binds Provider settings UI controls, Provider modal controls, backup modal controls, long-context preset controls, API-key visibility toggles, and context-capacity probe controls.
- Entry point: called during app initialization at `src/interface/web/app.js:71`.
- Calls:
  - `this.openProviderModal()`
  - `this.closeProviderModal()`
  - `this.saveProviderConfig()`
  - `this.openBackupModal('export')`
  - `this.openBackupModal('import')`
  - `this.closeBackupModal()`
  - `this.confirmBackup()`
  - `this.showErrorToast()`
  - `this.isTauriAvailable()`
  - `this.invokeTauri('probe_provider_context_capacity', ...)`
  - `this.loadProviders()`
  - `this.updateCapabilityStatusDisplay()`
- Related DOM:
  - `addProviderBtnTab`
  - `cancelProvider`
  - `saveProvider`
  - `providerModalClose`
  - `providerModal`
  - `providerModalBaseUrlPreset`
  - `providerBaseUrl`
  - `providerModalToggleKey`
  - `providerApiKey`
  - `exportProviderBtnTab`
  - `importProviderBtnTab`
  - `providerLongContextPreset`
  - `providerMaxContext`
  - `providerMaxOutput`
  - `providerReserveOutput`
  - `providerSafetyMargin`
  - `providerRetrievalBudget`
  - `providerLongContextMode`
  - `backupModal`
  - `backupModalClose`
  - `cancelBackup`
  - `confirmBackup`
  - `backupTogglePassword`
  - `backupPassword`
  - `testContextCapacityBtn`
  - `cancelContextCapacityBtn`
  - `providerProbeLevelSelect`
  - `providerProbeStatusDetails`
  - `probeDetailStatus`
  - `probeDetailTested`
  - `probeDetailDeclared`
  - `probeDetailLatency`
  - `probeDetailErrorWrap`
  - `probeDetailError`
  - `providerId`
  - `providerModel`
- Global state touched or read:
  - `activeContextProbeCancelled`
  - provider state indirectly through called methods such as `loadProviders()`.

### `handleChatCommand(text)`

- Location: `src/interface/web/app.js:2572`
- Responsibility: maps Chinese slash-command aliases to English commands, parses slash commands, and dispatches command behavior.
- Entry point: called by `sendChatMessage()` when the submitted text starts with `/`, at `src/interface/web/app.js:2476`.
- Command branches observed:
  - `/agent`
  - `/tools`
  - `/providers`
  - `/tool`
  - `/chat`
  - `/mcp list`
  - `/mcp init`
  - `/mcp invoke`
  - `/search`
  - `/git status`
  - `/git diff`
  - `/git commit`
  - `/extensions`
  - `/compact`
  - unknown command fallback
- Calls:
  - `this.addChatMessage()`
  - `this.invokeAgentTask()`
  - `this.isTauriAvailable()`
  - `this.invokeTauri('list_tools')`
  - `this.executeToolWithConfirmationRetry()`
  - `this.streamChat()`
  - `this.mcpInit()`
  - `this.saveMcpServers()`
  - `this.renderMcpServers()`
  - `this.mcpInvoke()`
  - `this.executeTool()`
  - `this.getActiveProviderConfig()`
  - `this.addThinking()`
  - `this.removeThinking()`
  - `this.updateTokenDisplay()`
  - `this.escapeHtml()`
- Related DOM:
  - No direct `document.getElementById()` calls were observed inside the sampled function body. DOM changes are performed through called methods such as `addChatMessage()` and `streamChat()`.
- Global state touched or read:
  - `providerConfigs`
  - `mcpServers`
  - `extensions`
  - `installedExtensions`
  - `chatMessages`
  - `activeProviderId`

### `streamChat(provider, prompt, config, messages)`

- Location: `src/interface/web/app.js:3273`
- Responsibility: invokes backend `stream_chat`, receives model streaming events through a Tauri channel, parses streaming/thinking payloads, updates assistant-turn UI, records diagnostics, and returns the visible response.
- Entry points:
  - `sendChatMessage()` normal chat path at `src/interface/web/app.js:2522`
  - `/chat` branch in `handleChatCommand()` at `src/interface/web/app.js:2687`
- Calls:
  - `this.isTauriAvailable()`
  - `this.getTauriInvoke()`
  - `this.getTauriChannel()`
  - `this.createAssistantTurn()`
  - `this.updateTurnResponse()`
  - `this.recordStreamDiagnostic()`
  - `this.updateTurnThinking()`
  - `this.generateDemoResponse()`
  - `this.parseStreamEvent()`
  - `this.scheduleDomUpdate()`
  - `this.updateTokenDisplay()`
  - backend command `stream_chat`
- Related DOM:
  - `aiChatMessages`
- Global state touched or read:
  - `_lastAssistantTurn`
  - `_streamDiagnosticCounter`
  - `tokenStats`
  - `cumulativeStats`

### `sendChatMessage()`

- Location: `src/interface/web/app.js:2443`
- Responsibility: reads chat input, appends user message, handles slash commands, validates selected provider, calls `streamChat()` in Tauri mode, falls back to local demo in non-Tauri mode, and resets UI processing state.
- Entry points:
  - slash palette direct low-risk selection calls `this.sendChatMessage()` at `src/interface/web/app.js:2358`
  - Enter key handler calls `this.sendChatMessage()` at `src/interface/web/app.js:2379`
  - chat send button click calls `this.sendChatMessage()` at `src/interface/web/app.js:2389`
- Calls:
  - `this.buildContextPrompt()`
  - `this.addChatMessage()`
  - `this.updateTokenDisplay()`
  - `this.showStatusIndicator()`
  - `this.safeUpdateTaskDetails()`
  - `this.renderLiveShellState()`
  - `this.safeRenderContextFiles()`
  - `this.safeRenderModelInfo()`
  - `this.handleChatCommand()`
  - `this.saveChatSessions()`
  - `this.renderSessionList()`
  - `this.hideStatusIndicator()`
  - `this.isTauriAvailable()`
  - `this.streamChat()`
  - `this.createAssistantSessionMessage()`
  - `this.createAssistantTurn()`
  - `this.updateTurnResponse()`
  - `this.generateDemoResponse()`
  - `this.saveCumulativeToLocalStorage()`
  - `this.checkAutoCompact()`
- Related DOM:
  - `aiChatInput`
  - `aiChatSendBtn`
  - `aiChatModelSelect`
- Global state touched or read:
  - `chatContextFiles`
  - `chatMessages`
  - `tokenStats`
  - `isProcessing`
  - `activeProviderId`
  - `providerConfigs`

## Readonly Findings

1. `handleChatCommand()` `/chat`, `/search`, and `/compact` branches appear to reference `invoke`, but no local declaration was observed inside the sampled function body.
2. `setupProviderSettings()` is Provider high-risk adjacent. It was sampled only and not extracted, changed, or fixed.
3. `streamChat()` is part of Agent streaming / model streaming display. It was sampled only and not extracted, changed, or fixed.

## Validation Commands

```powershell
git diff --name-only -- src/interface/web/app.js src/interface/web/index.html src/interface/web/style.css src/interface/web/modules src/interface/desktop/src/main.rs src/interface/desktop/tauri.conf.json src/engine/tool-system/src/shell.rs
git status --short
```

Observed validation summary:

- Production/high-risk path diff command returned no output before this report was written.
- `git status --short` showed pre-existing dirty files only before this report was written.

## Next Recommended Step

Open a separate readonly smoke task to verify `handleChatCommand()` `/chat`, `/search`, and `/compact` branches.

## Explicit Non-Claims

- This report does not fix any issue.
- This report does not prove WebView click behavior.
- This report does not prove runtime behavior for the sampled command branches.
