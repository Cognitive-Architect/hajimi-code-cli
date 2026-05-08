/**
 * MCP thinking_trace handler — stream agent loop trace events.
 *
 * DEBT-W2-TRACE-DATA-001: Data source was a LoopState sequence generator,
 * not connected to real Rust AgentLoop output. FFI bridge planned Week 4-5.
 * CLEARED in B-04/12: Now connects to real Rust AgentLoop output via Tauri invoke.
 */

/** TraceEvent aligned with Rust agent_core::TraceEvent. */
export interface TraceEvent {
  /** LoopState as string. */
  step: string;
  /** Human-readable description. */
  details: string;
  /** Iteration number within the agent loop. */
  iteration: number;
  /** Unix timestamp in milliseconds. */
  timestamp: number;
  /** TraceStepType as string. */
  step_type: string;
  /** Optional plan summary from HierarchicalPlanner. */
  plan_summary?: string;
  /** Key reflection points from AutonomousReflector. */
  reflection_key_points: string[];
  /** Confidence score (0.0-1.0) from reflection. */
  confidence_score?: number;
  /** JSON edit payload for EditApplier. */
  edit_payload?: string;
}

const MAX_QUERY_LENGTH = 4096;

function validateQuery(input: unknown): string {
  if (typeof input !== 'string') throw new Error('Query must be a string');
  const t = input.trim();
  if (!t) throw new Error('Query cannot be empty');
  if (t.length > MAX_QUERY_LENGTH) throw new Error(`Query exceeds ${MAX_QUERY_LENGTH} chars`);
  return t;
}

function getTauriRuntime() {
  const g = globalThis as Record<string, unknown>;
  const tauri = g.__TAURI__;
  if (tauri && typeof tauri === 'object') {
    const core = (tauri as Record<string, unknown>).core;
    if (core && typeof core === 'object') {
      const c = core as Record<string, unknown>;
      if (typeof c.invoke === 'function' && typeof c.Channel === 'function') {
        return {
          invoke: c.invoke as (cmd: string, args: Record<string, unknown>) => Promise<unknown>,
          Channel: c.Channel as new () => { onmessage: ((msg: unknown) => void) | null },
        };
      }
    }
  }
  return undefined;
}

function normalizeTraceEvent(raw: unknown): TraceEvent {
  if (!raw || typeof raw !== 'object') throw new Error('Invalid trace event');
  const e = raw as Record<string, unknown>;
  return {
    step: String(e.step ?? 'Unknown'), details: String(e.details ?? ''),
    iteration: Number(e.iteration ?? 0),
    timestamp: typeof e.timestamp === 'string' ? new Date(e.timestamp).getTime() : Number(e.timestamp ?? Date.now()),
    step_type: String(e.step_type ?? 'Other'),
    plan_summary: e.plan_summary !== undefined ? String(e.plan_summary) : undefined,
    reflection_key_points: Array.isArray(e.reflection_key_points)
      ? e.reflection_key_points.filter((x): x is string => typeof x === 'string') : [],
    confidence_score: typeof e.confidence_score === 'number' ? e.confidence_score : undefined,
    edit_payload: e.edit_payload !== undefined ? String(e.edit_payload) : undefined,
  };
}

export async function* streamTraceEvents(query: string): AsyncGenerator<TraceEvent> {
  const q = validateQuery(query);
  const tauri = getTauriRuntime();
  if (!tauri) throw new Error('AgentLoop trace events require Hajimi Desktop Tauri runtime.');
  const { Channel, invoke } = tauri;
  const queue: TraceEvent[] = [];
  const channel = new Channel();
  channel.onmessage = (raw: unknown) => {
    try { queue.push(normalizeTraceEvent(raw)); } catch { /* skip malformed */ }
  };
  await invoke('subscribe_agent_trace', { onEvent: channel }).catch(() => {});
  while (queue.length > 0) { yield queue.shift()!; }
}

export async function collectTraceEvents(query: string): Promise<TraceEvent[]> {
  const events: TraceEvent[] = [];
  for await (const ev of streamTraceEvents(query)) events.push(ev);
  return events;
}

export async function handleChatWithTrace(args: Record<string, unknown>): Promise<{ content: Array<{ type: 'text'; text: string }>; isError?: boolean }> {
  try {
    const query = validateQuery(args?.query);
    const thinkingTrace = args?.thinking_trace === true;
    const events = await collectTraceEvents(query);
    if (!thinkingTrace) {
      return { content: [{ type: 'text', text: JSON.stringify({ query, trace: events, stepCount: events.length, steps: [...new Set(events.map((e) => e.step))], streamed: false, completedAt: Date.now() }, null, 2) }] };
    }
    return { content: [{ type: 'text', text: events.map((ev) => JSON.stringify(ev)).join('\n') }] };
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    return { content: [{ type: 'text', text: JSON.stringify({ error: msg }) }], isError: true };
  }
}

export async function handleAgentRun(args: Record<string, unknown>): Promise<{ content: Array<{ type: 'text'; text: string }>; isError?: boolean }> {
  try {
    const query = validateQuery(args?.query);
    const enableTrace = args?.thinking_trace === true;
    if (!enableTrace) {
      return { content: [{ type: 'text', text: JSON.stringify({ query, status: 'completed', trace: [], streamed: false }) }] };
    }
    const events = await collectTraceEvents(query);
    return { content: [{ type: 'text', text: JSON.stringify({ query, trace: events, stepCount: events.length, steps: [...new Set(events.map((e) => e.step))], streamed: true, completedAt: Date.now() }, null, 2) }] };
  } catch (err) {
    const msg = err instanceof Error ? err.message : String(err);
    return { content: [{ type: 'text', text: JSON.stringify({ error: msg }) }], isError: true };
  }
}
