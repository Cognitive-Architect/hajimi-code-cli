# Day 5 Chat, Model, and UI Feedback Full Sampling

This document provides a comprehensive read-only inventory and code-smell analysis of the Day 5 frontend domains within the Hajimi IDE codebase.

## 1. Git / Baseline Snapshot

- **Branch**: `stone-audit-v3x-controlled-demolition`
- **HEAD**: `6667d86e056bbf813866e735cadc1815783bf923`
- **Git Status (`git status --short`)**:
```
 M docs/debt/DEBT-AGENT-LLM-NATIVE-BUDGET-MELTDOWN.md
 M docs/debt/INDEX.md
 M docs/debt/active/ACTIVE-DEBT-STATUS-2026-05-17.md
 D "docs/roadmap/Hajimi Agent/debt/DEBT-DAY-07-CHECKPOINT-DIFF-UI.md"
 D "docs/roadmap/Hajimi Agent/plan/AGENT-UI-INTEGRATION-SAMPLING-NOTES.md"
 D "docs/roadmap/Hajimi LLM/plan/AGENT-LLM-NATIVE-DESIGN.md"
 D "docs/roadmap/Hajimi LLM/plan/LLM-NATIVE-AGENT-MIGRATION-ROADMAP.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-01/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-02/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-03/snapshot.md"
 D "docs/roadmap/Hajimi RealAgent/evidence/day-04/snapshot.md"
?? docs/debt/DEBT-AGENT-CHINESE-I18N.md
?? docs/debt/DEBT-AGENT-LLM-NATIVE-DRIVER-INJECTION.md
?? docs/debt/DEBT-AGENT-LLM-NATIVE-THINKING-LEAK.md
?? docs/debt/DEBT-AGENT-LOOP-LLM-NO-OP.md
?? docs/debt/DEBT-AGENT-UI-INTEGRATION.md
?? docs/debt/DEBT-TAURI-CHANNEL-ENVELOPE-REGRESSION.md
?? docs/debt/STONE-AUDIT-V1.5-PACKAGE-SMOKE.md
?? "docs/roadmap/Hajimi AgentFix/"
?? "docs/roadmap/Hajimi ToneFix/"
?? "docs/roadmap/hajimi template.7z"
?? "docs/roadmap/hajimi template/"
?? src/interface/desktop/native-smoke.txt
```
- **Old Dirty Files Staged**: `NO` (all modifications are unstaged, local untracked roadmap folders exist)
- **Production Code Changes**: `NO` (no changes made to `app.js`, `index.html` or any backend components)

---

## 2. Chat Render Inventory

This domain handles chat messages presentation, Markdown parsing, code-block copying, and rendering LLM thinking blocks.

| Function | Location (`app.js`) | Role |
| :--- | :--- | :--- |
| `createAssistantTurn` | L2832 - L2894 | Dynamically creates an AI turn DOM structure (`article.assistant-turn`) containing the avatar, collapsible thinking panel, and response container. Returns DOM handles and state. |
| `hasSessionThinking` | L2896 - L2901 | Checks if an assistant message contains thinking content or state. |
| `snapshotAssistantTurn` | L2903 - L2914 | Serializes the current state of an assistant turn (thinking and response) to save inside a session. |
| `createAssistantSessionMessage` | L2916 - L2932 | Packages text content and an assistant turn snapshot into a clean session message structure. |
| `renderChatMessageFromSession` | L2934 - L2963 | Decodes a session message and recreates the corresponding AI turn DOM, restoring the past thinking timeline and formatted response. |
| `updateTurnThinking` | L2965 - L2993 | Modifies thinking states (`thinking`, `done`, `error`), calculates elapsed time, and delegates DOM rendering to `window.HajimiThinkingUI`. |
| `updateTurnResponse` | L2995 - L3023 | Manages the assistant response text state, handling errors and rendering Markdown-rendered HTML. |
| `addChatMessage` | L3417 - L3444 | Appends standard user messages or raw assistant fallbacks to the message panel and sets up code block "📋" copy buttons. |
| `addThinking` | L3446 - L3448 | Wrapper helper for starting thinking UI tracking. Delegates to `window.HajimiThinkingUI.addThinking`. |
| `removeThinking` | L3450 - L3452 | Removes a thinking tracker. Delegates to `window.HajimiThinkingUI.removeThinking`. |
| `createThinkingBlock` | L3454 - L3456 | Instantiates a thinking block DOM element via `window.HajimiThinkingUI.createThinkingBlock`. |
| `toggleThinking` | L3458 - L3460 | Toggles open/close state of the thinking panel via `window.HajimiThinkingUI.toggleThinking`. |
| `updateThinkingContent` | L3462 - L3464 | Updates the text inside a thinking panel via `window.HajimiThinkingUI.updateThinkingContent`. |
| `formatText` | L4972 - L4982 | Performs basic regular expression replacements for simple markdown (bold, inline code, pre blocks, line breaks). |
| `renderMarkdown` | L4985 - L5016 | A comprehensive Markdown parser supporting headers, bold text, lists, code blocks, and sanitizing hyperlinks against XSS. |
| `sanitizeUrl` | L5019 - L5026 | Safety guard validating that URLs belong to approved schemes (`http`, `https`, `mailto`) to prevent `javascript:` XSS. |

---

## 3. Chat Orchestration Inventory

This domain orchestrates chat interactions, parses slash commands, and manages Tauri Event Channel stream distribution.

| Function | Location (`app.js`) | Role |
| :--- | :--- | :--- |
| `setupChat` | L2262 - L2311 | Attaches listeners to input boxes, send, clear, and abort buttons, and configures keybind triggers for sending. |
| `sendChatMessage` | L2313 - L2494 | Main entry point for user submissions. Manages loading states, intercepts slash commands, records tokens, and kicks off `streamChat` or local fallback demo response. |
| `handleChatCommand` | L2496 - L2797 | The main router for slash commands (both English and Chinese). Coordinates commands like `/agent`, `/tools`, `/providers`, `/tool`, `/chat`, `/mcp`, `/search`, `/git`, `/extensions`, `/compact`. |
| `streamChat` | L3199 - L3380 | Orchestrates stream response. Spawns Tauri Channel, routes chunk data through `parseStreamEvent`, invokes `scheduleDomUpdate` for non-blocking rendering, and records diagnostic stats. |
| `invokeAgentTask` | L3025 - L3094 | Kicks off autonomous agent task using `run_agent_task` RPC via Tauri Channel. Initializes the trace event collector. |
| `handleAgentEvent` | L3096 - L3197 | Receives state/result/error/trace logs from the active agent run. Strips `<thinking>` tags from final responses (`DEBT-AGENT-LLM-NATIVE-THINKING-LEAK` fix) and updates UI cards. |

---

## 4. Model Picker UI Inventory

This domain manages the active provider selection dropdown, custom providers settings forms, and long-context capacity probes.

| Function | Location (`app.js`) | Role |
| :--- | :--- | :--- |
| `loadProviders` | L3535 - L3548 | Fetches saved provider configurations from the backend. |
| `renderModelButton` | L3550 - L3556 | Updates the text of the model picker dropdown to reflect the active selection and refreshes the sidebar summary. |
| `setupModelPicker` | L3561 - L3575 | Attaches event listeners for opening, closing, and actions within the model picker modal. |
| `openModelPicker` | L3577 - L3580 | Displays the model picker modal overlay. |
| `closeModelPicker` | L3582 - L3584 | Dismisses the model picker modal overlay. |
| `renderModelPicker` | L3586 - L3634 | Renders current providers inside the picker modal, binding buttons for selecting, editing, or deleting models. |
| `selectProvider` | L3636 - L3646 | Switches active provider ID, updates the top bar model display, and updates corresponding settings. |
| `renderProviderList` | L3648 - L3683 | Renders the provider configuration list inside the Settings Tab panel. |
| `setupProviderSettings` | L3685 - L3784 | Binds click handlers for provider forms, presets, encryption backups, and wires up Capacity Probe test controls. |
| `testCapacityBtn Event Handlers` | L3787 - L3954 | Orchestrates context capacity testing. Confirms expensive runs, updates tested token progress bars via intervals, handles cancels, and triggers `probe_provider_context_capacity` RPC. |
| `openProviderModal` | L3956 - L4001 | Opens the detailed provider modal form. Presets fields and queries past capacity probe results. |
| `updateCapabilityStatusDisplay`| L4016 - L4055 | Highlights probe status (e.g., `Verified`, `Fallback`, `Stale`, `MockOnly`, `Cancelled`) with color codes inside the provider form. |
| `saveProviderConfig` | L4145 - L4202 | Gathers form inputs, maps parameters, and writes configuration to global/workspace via Tauri. Erases sensitive fields in RAM. |
| `editProviderConfig` | L4204 - L4207 | Pre-populates the editor modal with selected provider configuration. |
| `deleteProviderConfig` | L4209 - L4218 | Removes the specified provider from storage via Tauri. |
| `exportProviderBackup` | L4124 - L4132 | Exports encrypted credentials backup via Tauri. |
| `importProviderBackup` | L4134 - L4143 | Imports encrypted credentials backup via Tauri. |

---

## 5. UI Feedback Inventory

This domain manages notifications, system popups, confirmations, and in-context execution progress bars.

| Function | Location (`app.js`) | Role |
| :--- | :--- | :--- |
| `showErrorToast` | L4954 - L4965 | Dynamically creates or updates an bottom-aligned error notification card, sliding it out after 4 seconds. |
| `hideErrorToast` | L4967 - L4970 | Immediately removes the active error notification card. |
| `showApprovalModal` | L4705 - L4789 | Renders high-risk operation safety prompt overlays based on agent event `approval_request`. Color-codes threats using `risk_score` and returns user resolution to Tauri. |
| `createOperationSummaryBar` | L3472 - L3474 | Injects a tool execution progress receipt inside an active assistant message (delegated to `HajimiThinkingUI`). |
| `toggleDetails` | L3478 - L3480 | Handles accordion expands on tool progress details (delegated to `HajimiThinkingUI`). |
| `updateOperationSummary` | L3484 - L3486 | Modifies the metrics or text inside the active tool receipt bar (delegated to `HajimiThinkingUI`). |
| `generateOperationReason` | L3489 - L3491 | Formulates clear English action summaries based on tools used (delegated to `HajimiThinkingUI`). |
| `renderOperationDiffPreview` | L3495 - L3497 | Renders code differences returned in execution traces (delegated to `HajimiThinkingUI`). |
| `updateOperationProgress` | L3500 - L3502 | Updates status tags inside running operation receipts (delegated to `HajimiThinkingUI`). |

---

## 6. Topbar Status Inventory

This domain drives navigation status, breadcrumbs, task state displays, and session-level tokens dashboard.

| Function | Location (`app.js`) | Role |
| :--- | :--- | :--- |
| `renderTopBarWorkspace` | L236 - L248 | Renders the absolute path of the loaded folder, applying middle truncation if length exceeds bounds. |
| `updateGitBranch` | L780 - L791 | Parses branch name from git status command stdout and displays it. |
| `renderLiveShellState` | L226 - L234 | Modifies global task tags in the top header (`就绪`, `处理中...`, `running`, `completed`, `failed`). |
| `safeUpdateTaskDetails` | L504 - L506 | Updates inspector metrics (delegated to `HajimiInspector`). |
| `updateTokenDisplay` | L2158 - L2182 | Populates token counters on the status panel, showing session or lifetime metrics on click. |

---

## 7. Key Dependencies & Global Namespace Bindings

Multiple modular subsystems are exposed globally on the `window` object and interact with `app.js`:

1. `window.HajimiThinkingUI`
   - Handles the lifecycle of LLM thinking logs, timeline events parsing (`parseThinkingStream`, `parseStreamEvent`), progress bars, session replay rendering, and unified timeline generation.
2. `window.HajimiSessions`
   - Governs the creation, serialization, deletion, and DOM list rendering of chat sessions inside `localStorage`.
3. `window.HajimiWorkspace`
   - Initializes directory paths, maps project routing, and binds initial setups.
4. `window.HajimiInspector`
   - Updates statistics, draws Context Capacity receipts, and monitors memory allocations.
5. `window.HajimiAuditLog`
   - Renders security command history and authorization requests.
6. `window.HajimiResourceDashboard`
   - Updates CPU, RAM, and hardware metrics.
7. `window.HajimiCommandPaletteView` & `window.HajimiCommandController`
   - Intercepts keystrokes and controls command palette autocomplete dropdowns.

---

## 8. Analysis & Issues (Refactoring Targets)

As we move forward into the Day 5 migrations, several architectural bottlenecks are apparent:

### Code Smells & High Coupling
- **Giant File anti-pattern**: `app.js` is over 5500 lines long. Chat processing, modal handling, capacity checking, and DOM rendering are closely coupled.
- **Scattered DOM Mutations**: Direct queries like `document.getElementById('aiChatSendBtn').disabled = false;` are performed inside nested asynchronous promise blocks. If elements are removed from `index.html`, this will throw exceptions that crash the event loop.
- **State Synchronization Gaps**: The state of current providers is mirrored in multiple places (e.g., `this.activeProviderId`, `this.providerConfigs`, `localStorage`, and DOM elements like `statusModel`). There is no single source of truth, causing rendering inconsistencies.

### Refactoring Recommendations
1. **Extract Chat Services**:
   - Split `streamChat`, `handleAgentEvent`, and `sendChatMessage` into a dedicated controller/service file.
   - Encapsulate LLM/Agent communication channels away from DOM structure.
2. **Extract Model Picker Module**:
   - Move all UI modals and settings forms (including the `testContextCapacity` probe logic, preset dropdowns, and backup systems) to a standalone module. This will clean up over 600 lines from `app.js`.
3. **Isolate Toast & Confirmation Overlays**:
   - Transition `showErrorToast` and `showApprovalModal` into a generic notification/governance view component.
4. **Standardize Topbar Controller**:
   - Group `renderTopBarWorkspace`, `updateGitBranch`, and `renderLiveShellState` into a layout module that exposes simple event callbacks.

---

## 9. DOM Ownership Table

| DOM ID / Selector | Location | Current Owner | Candidate Owner | Risk |
| :--- | :--- | :--- | :--- | :--- |
| `#aiChatMessages` | `app.js` L2833 | `app.js` (`createAssistantTurn`, `addChatMessage`) | `ChatView` / `ChatRenderModule` | DOM reference collision; concurrent updates can corrupt scrolling position. |
| `#aiChatSendBtn` | `app.js` L2405 | `app.js` | `ChatController` / `ChatView` | Direct modification of `.disabled` state inside asynchronous business handlers may lead to frozen send states. |
| `#modelSelectBtn` | `app.js` L3551 | `app.js` (`renderModelButton`, `setupModelPicker`) | `ModelPickerController` / `ModelPickerView` | Clicks reset dropdown selection incorrectly if list states mismatch. |
| `#modelPickerModal` | `app.js` L3565 | `app.js` | `ModelPickerView` | Direct `classList` addition/removal causes layout transitions to misfire. |
| `#providerModal` | `app.js` L3598 | `app.js` | `ModelPickerView` | Directly bound forms can leak user credentials if DOM references are hijacked. |
| `#errorToast` | `app.js` L4955 | `app.js` | `NotificationView` / `FeedbackController` | Dynamic insertion of overlay in `document.body` conflicts with other overlay layers, leading to z-index fighting. |
| `#statusModel` | `app.js` L3641 | `app.js` (`selectProvider`) | `TopbarView` | Updating state doesn't sync with active settings dropdown labels if rendered out of order. |
| `#statusTokens` | `app.js` L5128 | `app.js` (`setupStatusBar`) | `TopbarView` / `StatusController` | Double-bound click listeners toggle cumulative token states repeatedly. |
| `#statusLang`, `#statusCursor` | `app.js` L173, L174 | `app.js` | `TopbarView` | Hardcoded status values bypass dynamic LSP check hooks. |

---

## 10. State Ownership Table

| State Field | Current Owner | Read Locations | Write Locations | Candidate Owner | Risk |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `chatMessages` | `app.js` | `sendChatMessage` (L2446), `handleChatCommand` (L2779), `streamChat` (L3337) | `sendChatMessage` (L2419, L2447, L2460, L2482), `handleChatCommand` (L2783) | `ChatSessionModel` / `SessionsService` | Switching active session during stream updates corrupts history log. |
| `activeProviderId` | `app.js` | `sendChatMessage` (L2416, L2433), `streamChat` (L3175), `saveProviderConfig` (L4155) | `selectProvider` (L3637) | `ModelPickerController` / `SettingsService` | Running an agent task under a decommissioned model configuration. |
| `providerConfigs` | `app.js` | `sendChatMessage` (L2433), `selectProvider` (L3639), `renderProviderList` (L3661), `editProviderConfig` (L4205) | `loadProviders` (L3540) | `SettingsService` / `ModelPickerController` | In-memory configuration state gets out of sync with workspace config file on disk. |
| `current model / selected model display` | `app.js` | `renderModelButton` (L3553) | `selectProvider` (L3641) | `TopbarView` | The display UI fails to update when custom providers are loaded asynchronously. |
| `isProcessing` (loading/streaming state) | `app.js` | `sendChatMessage` (L2314) | `sendChatMessage` (L2404, L2420, L2462, L2483), `invokeAgentTask` (L3067), `handleAgentEvent` (L3152, L3171) | `ChatController` / `AgentService` | Multiple network streams initiated concurrently if locking mechanisms fail. |
| `session title dependencies` | `app.js` | `renderSessionList` (L2410, L2471) | `saveChatSessions` (L2409, L2426, L2470) | `SessionsService` | Corrupted title string due to un-persisted memory states when switching pages. |
| `tokenStats` & `cumulativeStats` | `app.js` | `updateTokenDisplay` (L2158) | `streamChat` (L3322, L3327) | `HajimiInspector` / `StatsService` | Mismatch between local cache and true costs returned by backend stream channels. |

---

## 11. Cross-References Table

| Caller | Callee | Domain A | Domain B | Risk | Recommendation |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `app.js (sendChatMessage)` | `window.HajimiSessions (saveChatSessions, renderSessionList)` | Chat Orchestration | Sessions | Sessions render while a network stream is still writing to local storage | Extract session persistence into an asynchronous background database queue. |
| `app.js (sendChatMessage)` | `app.js (streamChat)` | Chat Orchestration | Chat Render | Network delays block the main UI thread during streaming initialization | Use a decoupled asynchronous worker or event listener for streaming data chunks. |
| `app.js (handleChatCommand)` | `app.js (executeToolWithConfirmationRetry)` | Chat Orchestration | UI Feedback | Inline confirmation modals block command parsing logic | Extract confirmation request to a global Notification / Dialog Service. |
| `app.js (handleAgentEvent)` | `app.js (parseThinkingStream)` | Chat Orchestration | Chat Render | Thinking blocks parsing relies on regex matchers inside UI thread | Move thinking tag parsing to an isolated transformer utility. |
| `app.js (selectProvider)` | `app.js (safeRenderModelInfo, renderLiveShellState)` | Model Picker UI | Topbar Status | Model changes trigger global UI recalculations, causing jitter | Dispatch custom events to notify topbar status instead of calling direct methods. |
| `app.js (setupStatusBar)` | `app.js (updateTokenDisplay)` | Topbar Status | Stats | Queries status bar DOM elements before window completes render | Wrap status bar in a View component that initializes after document load. |

---

## 12. Existing Smoke Coverage Matrix

| Smoke File | Covers | Direct or Indirect | Pre-Migration Gate | Post-Migration Gate | Gap |
| :--- | :--- | :--- | :--- | :--- | :--- |
| `day16_slash_palette_smoke.js` | Slash Command palette visibility and autocomplete triggers. | Direct | YES | YES | Does not execute real backend Tauri commands. |
| `day21_slash_palette_app_integration_smoke.js` | Keyboard shortcuts and palette invocation integration with `app.js`. | Direct | YES | YES | Relies on inline palette fallback when modules fail. |
| `day27_handle_chat_command_invoke_smoke.js` | Intercepting and executing slash commands on the message event loop. | Direct | YES | YES | Mocks Tauri RPC responses and cannot catch Rust-side serialization errors. |
| `day14_sessions_thinking_modules_smoke.js` | AI thinking streams, collapsible UI components, and session state restoring. | Direct | YES | YES | Relies heavily on Mock Tauri Channel envelopes. |
| `day29_session_list_dom_smoke.js` | Session item dynamic rendering and switching handlers. | Direct | YES | YES | Uses `innerHTML` directly, triggering security warnings. |
| `day19_settings_smoke.js` | Provider form validation, credential key bindings, and profile setups. | Direct | YES | YES | Does not test capacity probe cancellation or timing metrics. |
| `npm run test:security-gate` | Security constraints (no unsafe innerHTML usage, CSP, rates). | Direct | YES | YES | Command exits with 1 due to legacy `command-palette-view.js` and `session-list-view.js` unallowlisted entries. |

---

## 13. Migration Recommendation Table

| Batch | Name | Modules Included | Recommendation |
| :--- | :--- | :--- | :--- |
| **Batch 1** | **SAFE-NOW** | `showErrorToast`, `renderMarkdown`, `sanitizeUrl`, `formatText`, `renderTopBarWorkspace`, `updateGitBranch`, `renderLiveShellState` | Safe to move immediately. Group toast into `FeedbackView`, markdown parser into `MarkdownService`, and topbar triggers into `TopbarView`. |
| **Batch 2** | **NEEDS NODE SMOKE** | `handleChatCommand` (routing & commands registry), `createAssistantTurn`, `updateTurnThinking`, `updateTurnResponse` | Highly complex layout blocks. Needs dedicated Node.js smoke coverage verification before refactoring. |
| **Batch 3** | **NEEDS WEBVIEW RECEIPT** | `streamChat`, `handleAgentEvent`, `sendChatMessage` | Streaming event loops and Agent trace callbacks. Requires high-fidelity WebView smoke tests with backend channel mocks. |
| **Batch 4** | **KEEP-FOR-NOW** | `saveProviderConfig`, `deleteProviderConfig`, `loadProviders`, custom profile manager, backups, Capacity Probes | **Highly coupled to OS Keyring, workspace local file paths, and Tauri IPC. Maintain as-is** until structural dependencies are decoupled. |

---

## 14. Forbidden Boundary Receipt

We strictly adhere to all constraints. The following receipt confirms that forbidden boundaries remain unmodified:
- Provider / Keyring modified: **NO**
- Shell modified: **NO**
- Checkpoint modified: **NO**
- CSP / withGlobalTauri modified: **NO**
- Agent streaming modified: **NO**
- streamChat modified: **NO**
- sendChatMessage modified: **NO**
- handleAgentEvent modified: **NO**
- production files modified: **NO**
- old dirty files staged: **NO**
