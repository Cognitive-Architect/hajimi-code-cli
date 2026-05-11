# CONTEXT_WINDOW_POLICY.md

> Task: AGENT-PROMPT-CORE-001  
> Status: v0.2 revised / docs-only deliverable  
> Scope: Defines token budget, context selection, memory/tool injection, and truncation policy.  
> Non-scope: Does not implement a `ContextWindowManager` or change retrieval code.

## 1. Purpose

This policy defines how Hajimi Agent Core should assemble context for each LLM call without flooding the model or starving it of necessary information.

人话版：模型上下文像背包，容量有限。这份文件规定先装什么、后装什么、装不下先丢什么。

## 2. Context Assembly Goals

A good context window must:

1. Preserve stable behavior rules.
2. Include the current task and output schema.
3. Include only task-relevant tools.
4. Include enough code/memory context for the Agent to reason correctly.
5. Avoid full-history dumps and full-tool dumps.
6. Keep enough response budget for valid structured output.

人话版：不要搬家式塞东西。今天去买菜，带钱包、清单、钥匙就够，不用把衣柜扛走。

## 3. Context Blocks and Priority

| Priority | Block | Injection rule | Notes |
|---:|---|---|---|
| P0 | Output Contract | Always include | JSON schema or exact response format. |
| P0 | Current Goal / Task | Always include | The approved objective and current step. |
| P0 | Safety / Governance Constraints | Always include compact form | Workspace, shell, approval, destructive-action rules. |
| P1 | Agent Persona | Always include or stable cached system prompt | Prefer stable system message, not repeated user text. |
| P1 | Relevant Tool Manifest | Include selected tools only | Pick by task type and previous failures. |
| P1 | Focus Memory | Always include if available | Active facts needed for current task. |
| P2 | Working Memory Summary | Include summarized | Current session / plan state. |
| P2 | Relevant Code Context | Include targeted snippets | Prefer symbols/files over broad dumps. |
| P3 | Archive Memory Hits | Include only top-ranked snippets | Use RAG-like retrieval and compact citations. |
| P4 | Long Logs / Full History | Exclude by default | Include only summarized excerpts. |

人话版：证件、任务单、安全规矩优先；旧聊天记录和超长日志最后再说。

## 4. Default Token Budget

For a target context window `N`, reserve budget like this:

| Bucket | Target share | Hard guidance |
|---|---:|---|
| System / Persona / Safety | 20% | Keep stable and compact. |
| Current Goal / Output Schema | 20% | Never omit. |
| Relevant Tool Manifest | 20% | Select tools; do not inject all tools. |
| Memory / Code Context | 30% | Focus first, then working summary, then archive snippets. |
| Response Reserve | 10% | Keep free for valid JSON or user response. |

For smaller models or low token limits, compress Persona and Tool Manifest before reducing Current Goal or Output Schema.

人话版：背包再小，身份证和任务单不能丢。可以少带零食，不能少带钥匙。

## 5. Memory Tiers

### 5.1 Focus Memory

Definition: facts, decisions, file paths, active constraints, or current failures that are directly needed for the next step.

Injection rule: always include while relevant.

Examples:

- Current approved goal.
- Owner-approved non-scope.
- Last failing command and error category.
- Active file path under edit.

人话版：Focus 是手上正在用的锅铲，必须放手边。

### 5.2 Working Memory

Definition: session-level state that helps continuity but can be summarized.

Injection rule: include as a compact summary.

Examples:

- Recent plan steps completed.
- Known module boundaries.
- Last validation result.

人话版：Working 是今天的备忘录，不必逐字背，知道重点就行。

### 5.3 Archive Memory

Definition: older records, long documents, prior sessions, or background knowledge that may be useful but is not always needed.

Injection rule: retrieve top relevant snippets only; do not dump full archive.

Examples:

- Prior architecture decisions.
- Historical bug-fix patterns.
- Deprecated debt notes.

人话版：Archive 是仓库旧档案，查到有用页再拿出来，不要整箱搬到灶台上。

## 6. Tool Manifest Selection

Do not inject every available tool. Select tools by task intent:

| Task intent | Preferred tool categories |
|---|---|
| Understand code | search, file read, symbol/LSP, AST/context tools |
| Edit code | file read, diff/edit, validation, Git status |
| Debug failure | command output, search, file read, test runner, logs |
| Validate change | build, test, lint, typecheck, targeted command |
| Git task | Git status, diff, commit tools; require approval for risky ops |
| User-facing explanation | receipts, trace, summary tools |

Selection algorithm:

1. Infer task intent from current goal/subgoal.
2. Include read-only discovery tools first.
3. Include edit tools only if the current task can edit.
4. Include validation tools that match the project type.
5. Add recovery hints for recently failing tools.
6. Cap tool count unless the task genuinely requires more.

人话版：修灯就带梯子和螺丝刀；别把电锯、油漆桶、混凝土机全拖上楼。

## 7. Code Context Selection

When code context is needed, prefer this order:

1. Directly referenced file/function/symbol.
2. Callers/callees or import dependencies.
3. Related tests.
4. Configuration/build files.
5. Broader module summaries.

Avoid dumping whole files when a symbol-level snippet is enough. Keep file paths and line references where possible.

人话版：医生看病先看症状和检查报告，不会一上来翻你从出生到现在所有病历。

## 8. Truncation Strategy

When context exceeds budget, remove or compress in this order:

1. Long logs → summarize to error lines and key stack frames.
2. Archive memory → keep top-ranked hits only.
3. Working memory → compress to bullet summary.
4. Tool manifest → keep only tools relevant to current next action.
5. Code context → keep exact symbol/snippet, remove unrelated surrounding text.
6. Persona → use compact version only if runtime already has stable system message.
7. Current goal / output schema → never remove.

人话版：背包装不下时，先丢大包装盒和零食，不能丢身份证、票和钥匙。

## 9. Recommended Context Object

```json
{
  "schema_version": "ContextBundleV1",
  "budget": {
    "max_tokens": 8000,
    "reserved_response_tokens": 800,
    "estimated_input_tokens": 0
  },
  "blocks": [
    {
      "name": "agent_persona",
      "priority": "P1",
      "content_type": "system_prompt",
      "token_estimate": 0,
      "truncatable": true
    },
    {
      "name": "current_goal",
      "priority": "P0",
      "content_type": "json",
      "token_estimate": 0,
      "truncatable": false
    },
    {
      "name": "tool_manifest",
      "priority": "P1",
      "content_type": "json",
      "token_estimate": 0,
      "truncatable": true
    }
  ],
  "omitted": [
    {
      "source": "string",
      "reason": "budget|irrelevant|sensitive|duplicate"
    }
  ]
}
```

人话版：ContextBundle 就是打包清单，里面还要写哪些东西被省略了、为什么省略。

## 10. Sensitive Data and Safety

Context assembly must avoid unnecessary exposure of:

- API keys or secrets.
- Full environment dumps.
- Private credentials.
- Unrelated user files.
- Large logs with sensitive payloads.

When sensitive material is necessary for debugging, include only redacted summaries.

人话版：查账可以看金额，别把银行卡密码贴到白板上。

## 11. Acceptance Criteria

This policy is accepted when:

1. It defines context block priority.
2. It defines memory tiers and injection behavior.
3. It defines tool selection and truncation strategy.
4. It preserves current goal and output schema as non-removable blocks.
5. It can guide a future `ContextWindowManager` implementation.

人话版：这份规则合格的标准是：以后写代码的人能照着它做一个“上下文打包器”。

---

## v0.2 Addendum: Token Accounting

Every context block MUST be estimated before injection. The assembler should reserve output budget first, then admit blocks by priority.

```rust
pub struct TokenAccount {
    pub max_tokens: usize,
    pub reserved_response: usize,
    pub used: usize,
    pub remaining: usize,
}

pub enum TokenEstimateMethod {
    ExactProviderCount,
    HeuristicFallback,
}

pub struct ContextBlockEstimate {
    pub block_id: String,
    pub priority: ContextPriority,
    pub estimated_tokens: usize,
    pub method: TokenEstimateMethod,
    pub can_omit: bool,
    pub can_compact: bool,
}

impl TokenAccount {
    pub fn can_fit(&self, block_tokens: usize) -> bool {
        self.used + block_tokens <= self.max_tokens.saturating_sub(self.reserved_response)
    }
}
```

Recommended estimator order:

1. Prefer `LlmClient::count_tokens(messages, model)` when available.
2. If exact token counting is not available, use `heuristic_token_count`.
3. For mixed Chinese/English prompts, apply the existing heuristic rather than naive `chars / 4`.
4. Record the method used in trace metadata.

人话版：装背包前先称重。能用电子秤就用电子秤，没电子秤也别闭眼乱塞。

### Admission Order

1. Reserve response budget, default 10%.
2. Add P0 blocks. If P0 cannot fit, fail fast with `ContextOverflow(P0)`.
3. Add P1 blocks. If P1 cannot fit, compact first; omit only if marked omittable.
4. Add P2 blocks by relevance score.
5. Add P3 blocks only when budget remains.
6. Exclude P4 by default unless explicitly requested.
7. Record `omitted_blocks[]` with reason.

```json
{
  "context_accounting": {
    "max_tokens": 8000,
    "reserved_response": 800,
    "used": 6420,
    "remaining": 780,
    "omitted_blocks": [
      {
        "block_id": "archive-memory-hit-4",
        "priority": "P3",
        "reason": "budget_exceeded"
      }
    ]
  }
}
```

人话版：身份证和任务单塞不进去，那这趟别出门；旧聊天记录塞不进去，可以先不带，但要记账。

### Block Compaction Rules

| Block | First compaction | Last resort |
|---|---|---|
| Agent Persona | Use compact persona | Fail if still impossible |
| Tool Manifest | Drop low-score tools, then shorten descriptions | Keep minimum current tool candidate set |
| Working Memory | Summarize to decisions + active constraints | Omit non-critical history |
| Code Context | Keep symbols and relevant excerpts | Replace with file path + line summary |
| Logs | Keep error lines and command | Omit full logs |

人话版：背包装不下，先把大衣压缩袋，不是先扔身份证。
