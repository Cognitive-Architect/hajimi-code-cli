import React, { useState, useEffect, useCallback, useMemo } from 'react';
import { Card, CardHeader, CardTitle, CardContent } from '@/components/ui/Card';
import type { TraceStep } from '../types/webview';

interface ThinkingTraceProps {
  trace?: TraceStep[];
}

const STEP_META: Record<string, { icon: string; color: string; bg: string; desc: string }> = {
  Observe: { icon: '👁', color: 'text-blue-400', bg: 'bg-blue-400/10', desc: 'Check environment & blackboard' },
  Retrieve: { icon: '📚', color: 'text-cyan-400', bg: 'bg-cyan-400/10', desc: 'Recall memories & context' },
  Plan: { icon: '📝', color: 'text-amber-400', bg: 'bg-amber-400/10', desc: 'Create goals & sub-tasks' },
  Act: { icon: '⚡', color: 'text-orange-400', bg: 'bg-orange-400/10', desc: 'Execute via swarm or direct' },
  Reflect: { icon: '🔍', color: 'text-purple-400', bg: 'bg-purple-400/10', desc: 'Critique execution result' },
  Store: { icon: '💾', color: 'text-emerald-400', bg: 'bg-emerald-400/10', desc: 'Checkpoint & persist plan' },
  Decide: { icon: '🎯', color: 'text-rose-400', bg: 'bg-rose-400/10', desc: 'Governance approval & next step' },
  Idle: { icon: '⏳', color: 'text-gray-400', bg: 'bg-gray-400/10', desc: 'Waiting to start' },
  Completed: { icon: '✅', color: 'text-green-400', bg: 'bg-green-400/10', desc: 'All steps finished' },
  Failed: { icon: '❌', color: 'text-red-400', bg: 'bg-red-400/10', desc: 'Execution failed' },
};

const ALL_STEPS = ['Observe', 'Retrieve', 'Plan', 'Act', 'Reflect', 'Store', 'Decide'];

function getStepStatus(stepName: string, trace: TraceStep[]): 'active' | 'completed' | 'error' | 'pending' {
  const found = trace.find((t) => t.step === stepName);
  if (found) return found.status;
  return 'pending';
}

function getStepDetails(stepName: string, trace: TraceStep[]): string {
  const found = trace.find((t) => t.step === stepName);
  return found?.details ?? 'Waiting for agent to reach this step...';
}

function getStepMeta(stepName: string) {
  return STEP_META[stepName] ?? STEP_META.Idle;
}

/**
 * ThinkingTrace — Accordion-style Collapsible 7-step agent loop trace cards.
 * Features: real-time updates, auto-expand active step, localStorage persistence,
 * iteration counter, status indicators, and timeline sync support.
 */
export const ThinkingTrace: React.FC<ThinkingTraceProps> = ({ trace = [] }) => {
  // Throttle rapid trace updates to prevent UI jank during high-frequency streaming
  const [throttledTrace, setThrottledTrace] = useState<TraceStep[]>(trace);
  useEffect(() => {
    const timer = setTimeout(() => setThrottledTrace(trace), 30);
    return () => clearTimeout(timer);
  }, [trace]);

  const [expanded, setExpanded] = useState<Set<string>>(() => {
    try {
      const raw = localStorage.getItem('hajimi-trace-expanded');
      return raw ? new Set(JSON.parse(raw)) : new Set<string>();
    } catch {
      return new Set<string>();
    }
  });

  // Auto-expand the currently active step when trace updates
  useEffect(() => {
    const active = trace.find((t) => t.status === 'active');
    if (active) {
      setExpanded((prev) => {
        const next = new Set(prev);
        next.add(active.step);
        return next;
      });
    }
  }, [trace]);

  // Persist expanded state to localStorage
  useEffect(() => {
    try {
      localStorage.setItem('hajimi-trace-expanded', JSON.stringify(Array.from(expanded)));
    } catch {
      // Silent fail if localStorage is unavailable
    }
  }, [expanded]);

  const toggleStep = useCallback((step: string) => {
    setExpanded((prev) => {
      const next = new Set(prev);
      if (next.has(step)) next.delete(step);
      else next.add(step);
      return next;
    });
  }, []);

  const activeStep = useMemo(() => throttledTrace.find((t) => t.status === 'active')?.step ?? null, [throttledTrace]);
  const completedCount = useMemo(() => throttledTrace.filter((t) => t.status === 'completed').length, [throttledTrace]);
  const hasError = useMemo(() => throttledTrace.some((t) => t.status === 'error'), [throttledTrace]);

  return (
    <Card className="h-full flex flex-col">
      <CardHeader className="py-2 px-3">
        <CardTitle className="text-xs uppercase tracking-wider text-[var(--vscode-descriptionForeground)] flex items-center gap-2">
          <span>Thinking Trace</span>
          {activeStep && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-[var(--vscode-badge-background)] text-[var(--vscode-badge-foreground)] animate-pulse">
              {activeStep}
            </span>
          )}
          {hasError && (
            <span className="text-[10px] px-1.5 py-0.5 rounded bg-red-600 text-white">
              Error
            </span>
          )}
        </CardTitle>
        <div className="flex items-center gap-1 mt-1">
          {ALL_STEPS.map((s, i) => (
            <div
              key={s}
              className={`h-1 flex-1 rounded-full transition-colors ${
                i < completedCount ? 'bg-green-400' : i === completedCount && activeStep ? 'bg-blue-400' : 'bg-[var(--vscode-panel-border)]'
              }`}
            />
          ))}
        </div>
      </CardHeader>
      <CardContent className="flex-1 overflow-y-auto px-2 py-1 space-y-1">
        {ALL_STEPS.map((stepName) => {
          const status = getStepStatus(stepName, throttledTrace);
          const details = getStepDetails(stepName, throttledTrace);
          const meta = getStepMeta(stepName);
          const isExpanded = expanded.has(stepName);
          const isActive = status === 'active';
          const stepData = throttledTrace.find((t) => t.step === stepName);

          return (
            <div
              key={stepName}
              className={`rounded-md border border-[var(--vscode-panel-border)] overflow-hidden transition-all ${
                isActive ? 'ring-1 ring-blue-400/30' : ''
              }`}
            >
              <button
                onClick={() => toggleStep(stepName)}
                className={`w-full flex items-center gap-2 px-2 py-1.5 text-left text-xs hover:bg-[var(--vscode-list-hoverBackground)] transition-colors ${meta.bg}`}
                aria-expanded={isExpanded}
              >
                <span className="text-sm" aria-hidden="true">{meta.icon}</span>
                <span className={`font-medium ${meta.color}`}>{stepName}</span>
                <span className="text-[10px] text-[var(--vscode-descriptionForeground)] opacity-60 hidden sm:inline">
                  {meta.desc}
                </span>
                <span className="ml-auto flex items-center gap-1.5">
                  {isActive && (
                    <span className="h-1.5 w-1.5 rounded-full bg-blue-400 animate-pulse" title="Active" />
                  )}
                  {status === 'completed' && (
                    <span className="h-1.5 w-1.5 rounded-full bg-green-400" title="Completed" />
                  )}
                  {status === 'error' && (
                    <span className="h-1.5 w-1.5 rounded-full bg-red-400" title="Error" />
                  )}
                  {status === 'pending' && (
                    <span className="h-1.5 w-1.5 rounded-full bg-[var(--vscode-panel-border)]" title="Pending" />
                  )}
                  <span className="text-[10px] text-[var(--vscode-descriptionForeground)] opacity-70 w-3 text-center">
                    {isExpanded ? '▾' : '▸'}
                  </span>
                </span>
              </button>
              {isExpanded && (
                <div className="px-2 py-1.5 text-[10px] text-[var(--vscode-descriptionForeground)] border-t border-[var(--vscode-panel-border)] bg-[var(--vscode-editor-background)]">
                  <p className="leading-relaxed">{details}</p>
                  {stepData && (
                    <div className="mt-1 flex items-center gap-2 opacity-50">
                      <span>iteration {stepData.iteration}</span>
                      <span>•</span>
                      <span>{new Date(stepData.timestamp).toLocaleTimeString()}</span>
                    </div>
                  )}
                </div>
              )}
            </div>
          );
        })}
      </CardContent>
    </Card>
  );
};
