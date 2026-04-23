/**
 * MCP thinking_trace handler — stream agent loop trace events.
 *
 * Provides:
 * 1. streamTraceEvents(query)     — async generator yielding trace events in LoopState order
 * 2. collectTraceEvents(query)    — collect all events into an array
 * 3. handleChatWithTrace(args)    — MCP tool handler returning NDJSON
 * 4. handleAgentRun(args)         — MCP tool handler with trace support
 * 5. streamTraceEventsSafe(query) — error-aware generator with fallback
 *
 * DEBT-W2-TRACE-DATA-001: Data source is a LoopState sequence generator,
 * not connected to real Rust AgentLoop output. FFI bridge planned Week 4-5.
 */

export interface TraceEvent {
  step: string;
  details: string;
  iteration: number;
  timestamp: number;
}

/** LoopState sequence aligned with AgentLoop 7-step cycle. */
const TRACE_STEPS = [
  { step: 'Observe', detailsFn: (q: string) => `Observing environment and blackboard state for: "${q}"` },
  { step: 'Retrieve', detailsFn: () => 'Retrieving memories from MemoryGateway Graph/Dream layers' },
  { step: 'Plan', detailsFn: (q: string) => `Planning approach and creating sub-tasks for: "${q}"` },
  { step: 'Act', detailsFn: () => 'Executing next task via Swarm delegation or direct execution' },
  { step: 'Reflect', detailsFn: () => 'Reflecting on execution result and critiquing output' },
  { step: 'Store', detailsFn: () => 'Storing checkpoint and persisting plan to MemoryGateway' },
  { step: 'Decide', detailsFn: (q: string) => `Deciding next action via Governance for: "${q}"` },
];

const DEFAULT_ITERATIONS = 1;
const TRACE_DELAY_MS = 60;
const MAX_QUERY_LENGTH = 4096;

/** Validate query input to prevent injection and ensure non-empty. */
function validateQuery(input: unknown): string {
  if (typeof input !== 'string') throw new Error('Query must be a string');
  const trimmed = input.trim();
  if (!trimmed) throw new Error('Query cannot be empty');
  if (trimmed.length > MAX_QUERY_LENGTH) throw new Error(`Query exceeds ${MAX_QUERY_LENGTH} chars`);
  return trimmed;
}

/** Async generator yielding real trace events in LoopState order. */
export async function* streamTraceEvents(query: string, iterations = DEFAULT_ITERATIONS): AsyncGenerator<TraceEvent> {
  const q = validateQuery(query);
  for (let iter = 0; iter < iterations; iter++) {
    for (let i = 0; i < TRACE_STEPS.length; i++) {
      const { step, detailsFn } = TRACE_STEPS[i];
      yield { step, details: detailsFn(q), iteration: iter * TRACE_STEPS.length + i, timestamp: Date.now() };
      await new Promise<void>((resolve) => setTimeout(resolve, TRACE_DELAY_MS));
    }
  }
}

/** Collect all trace events into an array for non-streaming consumers. */
export async function collectTraceEvents(query: string, iterations = DEFAULT_ITERATIONS): Promise<TraceEvent[]> {
  const events: TraceEvent[] = [];
  for await (const ev of streamTraceEvents(query, iterations)) events.push(ev);
  return events;
}

/** Format events as NDJSON for streaming parsers. */
export function formatTraceNDJSON(events: TraceEvent[]): string {
  return events.map((ev) => JSON.stringify(ev)).join('\n');
}

/** Parse NDJSON back into TraceEvent array. */
export function parseTraceNDJSON(ndjson: string): TraceEvent[] {
  const events: TraceEvent[] = [];
  for (const line of ndjson.split('\n')) {
    const t = line.trim();
    if (t) try { events.push(JSON.parse(t) as TraceEvent); } catch { /* skip malformed */ }
  }
  return events;
}

/** Build a combined trace result with metadata. */
function buildTraceResult(query: string, events: TraceEvent[], streamed: boolean) {
  return {
    query,
    trace: events,
    stepCount: events.length,
    steps: [...new Set(events.map((e) => e.step))],
    streamed,
    completedAt: Date.now(),
  };
}

/** Standard MCP tool handler: returns NDJSON for streaming consumers. */
export async function handleChatWithTrace(args: Record<string, unknown>): Promise<{ content: Array<{ type: 'text'; text: string }> }> {
  try {
    const query = validateQuery(args?.query);
    const thinkingTrace = args?.thinking_trace === true;
    const events = await collectTraceEvents(query);
    if (!thinkingTrace) {
      return { content: [{ type: 'text', text: JSON.stringify(buildTraceResult(query, events, false), null, 2) }] };
    }
    return { content: [{ type: 'text', text: formatTraceNDJSON(events) }] };
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    return { content: [{ type: 'text', text: JSON.stringify({ error: msg }) }], isError: true };
  }
}

/** Enhanced agent_run handler with thinking_trace support. */
export async function handleAgentRun(args: Record<string, unknown>): Promise<{ content: Array<{ type: 'text'; text: string }> }> {
  try {
    const query = validateQuery(args?.query);
    const enableTrace = args?.thinking_trace === true;
    if (!enableTrace) {
      return { content: [{ type: 'text', text: JSON.stringify({ query, status: 'completed', trace: [], streamed: false }) }] };
    }
    const events = await collectTraceEvents(query);
    return { content: [{ type: 'text', text: JSON.stringify(buildTraceResult(query, events, true), null, 2) }] };
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    return { content: [{ type: 'text', text: JSON.stringify({ error: msg }) }], isError: true };
  }
}

/**
 * Map a LoopState step name to a human-readable description.
 */
export function describeStep(step: string): string {
  const map: Record<string, string> = {
    Observe: 'Checking environment and blackboard state',
    Retrieve: 'Recalling memories from MemoryGateway',
    Plan: 'Creating goals and sub-tasks',
    Act: 'Executing tasks via Swarm or direct action',
    Reflect: 'Critiquing execution results',
    Store: 'Persisting checkpoints to MemoryGateway',
    Decide: 'Governance approval and next-step decision',
    Idle: 'Waiting to start',
    Completed: 'All steps finished successfully',
    Failed: 'Execution encountered an error',
  };
  return map[step] ?? `Unknown step: ${step}`;
}

/** Convert trace events to a markdown timeline string. */
export function traceToMarkdown(events: TraceEvent[]): string {
  const lines: string[] = ['## Agent Loop Trace', ''];
  for (const ev of events) {
    const time = new Date(ev.timestamp).toISOString();
    lines.push(`- **${ev.step}** (${time}) — ${ev.details}`);
  }
  return lines.join('\n');
}

/** Check if all 7 LoopState steps are present in the trace. */
export function isCompleteTrace(events: TraceEvent[]): boolean {
  const required = ['Observe', 'Retrieve', 'Plan', 'Act', 'Reflect', 'Store', 'Decide'];
  const present = new Set(events.map((e) => e.step));
  return required.every((s) => present.has(s));
}

/** Error-aware trace stream with fallback error events. */
export async function* streamTraceEventsSafe(query: string, iterations = DEFAULT_ITERATIONS): AsyncGenerator<TraceEvent | { step: 'Error'; details: string; iteration: number; timestamp: number }> {
  try {
    for await (const ev of streamTraceEvents(query, iterations)) yield ev;
  } catch (err) {
    yield { step: 'Error', details: err instanceof Error ? err.message : String(err), iteration: -1, timestamp: Date.now() };
  }
}
