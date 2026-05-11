# TOOL_MANIFEST_SPEC.md

> Task: AGENT-PROMPT-CORE-001  
> Status: v0.2 gap-fill / docs-only deliverable  
> Scope: Defines how Hajimi Agent Core should generate, enrich, filter, and inject task-relevant tool manifests for Planner and Act prompts.  
> Non-scope: Does not implement `ToolManifestGenerator`, change `ToolRegistry`, or add new tools.

## 1. Purpose

This spec closes the bridge between the existing `engine-tool-system` runtime and the prompt framework. Planner and Act prompts can only be tool-aware if they receive a compact, accurate, task-scoped `ToolManifest`.

人话版：工具箱里有 40+ 把工具，但 Agent 不知道哪把是螺丝刀、哪把是电钻，就会瞎猜。这份文件就是给工具贴标签，再按任务挑出该带的那几把。

## 2. Source Runtime Facts

The current tool runtime exposes:

- `Tool` trait with `name()`, `description()`, `permissions()`, `is_enabled(config)`, and `execute(args)`.
- `ToolRegistry` with `register`, `get`, and `list`.
- Permission model: `PermissionLevel::{Deny, Ask, Allow}` plus `ToolPermissions` with `requires_confirmation` and optional path allow-list.
- Tool error taxonomy including permission, execution failure, invalid args, timeout, patch conflict, git errors, network errors, parse errors, and not-found errors.

Runtime docs also describe the tool system as a 40+ tool engine with file, git, search, build, LSP, MCP, analysis, and shell tools.

人话版：代码里已经有工具仓库和登记表，但登记表还不够给 LLM 用。LLM 需要的是“什么时候用、风险多高、失败怎么救”。

## 3. ToolManifestEntryV1 Schema

Each injected tool entry MUST match this shape:

```json
{
  "schema_version": "ToolManifestEntryV1",
  "name": "string",
  "description": "string",
  "category": "FileRead|FileWrite|Search|Git|Build|Test|Lsp|Mcp|Shell|Analysis|Docs|Network|Image|Other",
  "available": true,
  "risk_level": "Low|Medium|High|Critical",
  "requires_confirmation": false,
  "parameters_schema": {},
  "when_to_use": ["string"],
  "do_not_use_when": ["string"],
  "recovery_hints": ["string"],
  "evidence_expected": ["string"],
  "known_failure_kinds": ["PermissionDenied|ExecutionFailed|InvalidArgs|Timeout|PatchConflict|GitError|NetworkError|ParseError|NotFound|Unknown"],
  "budget_hint": {
    "max_description_chars": 200,
    "max_total_chars": 900
  }
}
```

人话版：这就是每个工具的身份证：名字、用途、危险等级、怎么用、什么时候别用、失败了怎么办、做完能留下什么证据。

## 4. Field Mapping: Runtime Tool to Manifest

| Runtime source | Manifest field | Rule |
|---|---|---|
| `Tool::name()` | `name` | Direct mapping. Must be unique in the manifest. |
| `Tool::description()` | `description` | Direct mapping, compact to 200 chars for prompt injection. |
| `Tool::is_enabled(config)` | `available` | `true` only when enabled by runtime config. Disabled tools are omitted by default. |
| `Tool::permissions().default_level` | `risk_level` | `Allow -> Low`, `Ask -> Medium`, `Deny -> Critical/unavailable`. |
| `Tool::permissions().requires_confirmation` | `requires_confirmation` | Direct mapping; also raises risk by one level unless already Critical. |
| `Tool::permissions().allowed_paths` | safety note | Include only as compact path-scope note if relevant to the current task. |
| `ToolErrorKind` | `known_failure_kinds` | Map known error classes into prompt-friendly enum values. |
| Existing module / tool name | `category` | Deterministic name/category mapping. |
| Manual catalog | `when_to_use`, `do_not_use_when`, `recovery_hints`, `evidence_expected`, `parameters_schema` | Maintained by docs/runtime adapter; do not invent at call time. |

When a tool lacks a manual catalog entry, generate a minimal fallback entry and mark missing fields as `UNKNOWN` rather than hallucinating details.

人话版：能从代码拿的就从代码拿；代码没有的，比如“失败后怎么救”，要人工补目录。别让模型临场编工具说明书。

## 5. Manual Tool Catalog

Runtime `Tool` objects are not enough for prompt quality. A lightweight catalog SHOULD be maintained alongside prompt specs.

Recommended future path:

```text
docs/agent-prompt-core/tool-catalog/
├── file_tools.json
├── search_tools.json
├── git_tools.json
├── build_test_tools.json
├── lsp_tools.json
├── mcp_tools.json
├── shell_tools.json
└── analysis_docs_network_tools.json
```

Each catalog entry SHOULD include:

```json
{
  "name": "read_file",
  "category": "FileRead",
  "parameters_schema": {
    "type": "object",
    "required": ["path"],
    "properties": {
      "path": { "type": "string" }
    }
  },
  "when_to_use": ["Read a known file before planning or editing."],
  "do_not_use_when": ["The file path is unknown; use search or list tools first."],
  "recovery_hints": ["If file is not found, search by filename or inspect directory."],
  "evidence_expected": ["File path and relevant excerpt or summary."]
}
```

人话版：代码里的工具像商品条码，catalog 像商品详情页。只有条码不够，Agent 还得知道“这东西适合干嘛”。

## 6. Tool Categories and Intent Matching

The generator SHOULD classify the current step intent before selecting tools.

| Intent | Typical cues | Preferred categories |
|---|---|---|
| Inspect repository | inspect, understand, audit, analyze | Search, FileRead, Analysis, Lsp |
| Locate symbol or usage | definition, reference, symbol, call site | Lsp, Search, FileRead |
| Fix bug | bug, failing, error, panic, test failure | Search, FileRead, Test, Build, FileWrite |
| Implement feature | add, implement, create | Search, FileRead, FileWrite, Build, Test |
| Refactor | refactor, rename, split, cleanup | Lsp, Search, FileRead, FileWrite, Test |
| Validate | verify, test, check, build | Test, Build, Shell |
| Git workflow | commit, diff, status, branch, PR | Git |
| Documentation | docs, README, comment, explain | Docs, FileRead, FileWrite |
| External lookup | web, URL, API, fetch | Network |
| MCP operation | MCP, server, invoke, tool bridge | Mcp |

人话版：修 bug 先带搜索、读文件、测试；写文档不用带 LSP 全家桶。按菜买菜，别把超市搬回家。

## 7. Filtering Algorithm

Input:

```json
{
  "goal_description": "string",
  "step_type": "Observe|Retrieve|Plan|Act|Reflect|Store|Decide",
  "current_task": "string|null",
  "recently_failed_tools": ["string"],
  "available_budget_tokens": 1600,
  "max_tools": 15
}
```

Output: filtered `ToolManifestEntryV1[]`.

Algorithm:

1. Load all registered tools from `ToolRegistry.list()`.
2. For each name, fetch `ToolRegistry.get(name)` and skip unavailable tools.
3. Merge runtime fields with manual catalog fields.
4. Classify intent from goal + task + step type.
5. Score each tool:
   - `+4` category matches intent.
   - `+3` tool name or description matches goal keyword.
   - `+2` tool is commonly needed by current step type.
   - `+2` tool provides expected evidence required by current task.
   - `-3` tool appears in `recently_failed_tools` with identical params.
   - `-2` risk is High and lower-risk alternative exists.
   - `-99` unavailable or disallowed by governance.
6. Sort by score descending, then risk ascending: Low → Medium → High → Critical.
7. Keep required tools first, then best-scored optional tools.
8. Cap to `max_tools`, default `<= 15`.
9. Compact descriptions and hints to fit budget.
10. Record omitted tools and reason in trace metadata, not in the main prompt unless needed.

人话版：先从仓库拿工具名单，再按今天任务打分，危险工具靠后，坏过的工具别原样重试，最后只带前 15 个以内。

## 8. Step-Specific Selection Rules

| Step | Manifest rule |
|---|---|
| Observe | Include read-only repo inspection tools: directory, search, git status, LSP hover/definition if relevant. |
| Retrieve | Include memory/search/context tools; exclude write tools unless explicitly needed. |
| Plan | Include categories likely needed by the plan, but prefer descriptions over parameter-heavy schemas. |
| Act | Include only tools executable for the current task; include full parameters schema for candidate tools. |
| Reflect | Include failed tool entry, fallback tools, validation tools, and evidence inspection tools. |
| Store | Include memory/persistence tools only if runtime supports them. |
| Decide | Include no execution tools by default; include summary/evidence contracts. |

人话版：规划阶段看工具简介就够；执行阶段必须带参数说明。看菜单和开火需要的信息不一样。

## 9. Runtime Interface Draft

```rust
pub struct ToolManifestGenerator {
    registry: std::sync::Arc<engine_tool_system::ToolRegistry>,
    catalog: ToolCatalog,
    config: engine_tool_system::Config,
}

pub struct ToolManifestRequest {
    pub goal_description: String,
    pub step_type: StepType,
    pub current_task: Option<String>,
    pub recently_failed_tools: Vec<String>,
    pub available_budget_tokens: usize,
    pub max_tools: usize,
}

pub struct ToolManifestEntry {
    pub schema_version: String,
    pub name: String,
    pub description: String,
    pub category: ToolCategory,
    pub available: bool,
    pub risk_level: RiskLevel,
    pub requires_confirmation: bool,
    pub parameters_schema: serde_json::Value,
    pub when_to_use: Vec<String>,
    pub do_not_use_when: Vec<String>,
    pub recovery_hints: Vec<String>,
    pub evidence_expected: Vec<String>,
    pub known_failure_kinds: Vec<String>,
}

impl ToolManifestGenerator {
    pub fn generate(&self, request: ToolManifestRequest) -> Vec<ToolManifestEntry>;
    pub fn generate_with_history(
        &self,
        request: ToolManifestRequest,
        failures: &[ToolFailureRecord],
    ) -> Vec<ToolManifestEntry>;
}
```

人话版：这是未来代码接口草图：输入今天要干什么、在哪一步、哪些工具刚翻车，输出一小盒可用工具卡片。

## 10. Governance and Risk Rules

- `Critical` tools MUST NOT be injected as callable candidates unless governance pre-approval exists.
- `High` tools MAY be included but MUST carry confirmation and evidence notes.
- Destructive file or git operations MUST require explicit approval unless runtime policy already approved them.
- Shell tools MUST include allow-list and injection-safety warning in compact form.
- Path-sensitive tools MUST include workspace sandbox notes.

人话版：电锯不是不能给，但得先问老板、戴护具、划安全线。别把危险工具偷偷塞给模型。

## 11. Failure-Aware Filtering

For each recent failure, store:

```json
{
  "tool_name": "string",
  "args_fingerprint": "sha256-or-stable-hash",
  "failure_kind": "InvalidArgs|Timeout|PermissionDenied|ExecutionFailed|Unknown",
  "error_summary": "string",
  "attempt_count": 1
}
```

Rules:

- Same tool + same args + same failure kind MUST NOT be retried unchanged.
- Same tool with corrected args MAY be retried once.
- Same failure pattern twice SHOULD prefer alternative tool or `StopAndHandoff`.
- Permission failures SHOULD route to `AskUser` or governance, not blind retry.

人话版：同一把钥匙开不了门，别拧十次。换钥匙、找门卫，或者停下来报告。

## 12. Validation Checklist

- [ ] Every injected tool name exists in `ToolRegistry`.
- [ ] Every injected tool is enabled under current config.
- [ ] Every injected tool has a category and risk level.
- [ ] Act-step manifest includes parameter schema for candidate tools.
- [ ] Planner-step manifest is compact and does not exceed budget.
- [ ] Manifest contains <= 15 tools unless Owner/maintainer explicitly raises the limit.
- [ ] Recently failed tool+args pairs are excluded or marked with changed-args requirement.
- [ ] High/Critical tools carry governance notes.
- [ ] Omitted tools are recorded in trace metadata for debug.

人话版：验收就看：工具是真的、可用、够少、风险标了、失败记录没忘。这样 Agent 才不是盲人摸象。

## 13. Out of Scope for This Spec

- Building the actual Rust generator.
- Rewriting existing tool traits.
- Adding new tools.
- Guaranteeing exact provider-specific function-calling behavior.

人话版：这份文件只定工具卡怎么生成和怎么筛，不负责现场把货架重装一遍。
