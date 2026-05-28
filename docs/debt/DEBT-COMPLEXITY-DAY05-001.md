# Technical Debt: DEBT-COMPLEXITY-DAY05-001

## 1. Debt Declaration
- **Debt ID**: `DEBT-COMPLEXITY-DAY05-001`
- **Component**: `src/intelligence/agent-core/agent_loop.rs`
- **Method**: `bootstrap_first_tool_call`
- **Status**: 🟢 Active (Documented & Justified)
- **Size**: ~145 Lines of Code (Exceeds the 80-line constraint)

---

## 2. Justification & Necessity
The `bootstrap_first_tool_call` method was developed during the **DAY-05 — LLM Bootstrap Mechanism Implementation** to address the critical `DEBT-AGENT-LOOP-LLM-NO-OP` issue. The method is required to be robust, secure, and fully self-contained, which necessitated an implementation size exceeding 80 lines due to the following requirements:

1. **Multi-Step Async Coordination**:
   - Safely locking the asynchronous `planner` via `self.planner.lock().await` to query the task queue state.
   - Intelligently sequencing a two-stage expansion: calling `.decompose()` to generate subgoals, followed by `.expand()` for the first subgoal to build the initial task sequence.

2. **Rule-Based Safe Fallback Mapping**:
   - When the LLM client fails or no tool calls are generated (`task.tool_calls.is_empty()`), the method safely maps task descriptions into rule-based tool calls (such as mapping compilation to `powershell`, reading files to `read_file`, and editing to `write_file`).

3. **Double-Ended Validation**:
   - Before writing any tool call to the blackboard, the method acquires the asynchronous lock of the `tool_registry` to verify that the chosen tool actually exists in the runtime environment.
   - If the desired tool is missing, it dynamically falls back to alternate safe tools (e.g., `analyze` or `ls`).

4. **UX & Telemetry Tracing**:
   - Writing the serialized DTO to the blackboard under `BB_NEXT_TOOL`.
   - Emitting trace events with precise metadata containing the bootstrapped tool name to keep the UI completely synchronized and visually responsive.

---

## 3. Remediation Strategy
No remediation is immediately required, as the complexity is inherent to the business requirements of safe and resilient bootstrap execution. 
If structural simplification is desired in the future, the mapping logic and DTO translation can be refactored into a separate builder or strategy pattern crate/module inside the `engine` or `intelligence` layers.
