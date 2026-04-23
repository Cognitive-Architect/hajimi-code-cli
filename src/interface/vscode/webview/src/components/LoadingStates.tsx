import React from 'react';

/** ------------------------------------------------------------------
 *  LoadingStates — Skeleton, LoadingSpinner, EmptyState
 *  Week 6 Polishing: Pure CSS/Tailwind animations (no framer-motion).
 *  Colors bridge Terminal Solarized → VSCode CSS variables.
 *  tailwind-animate classes: animate-pulse, animate-spin.
 * ------------------------------------------------------------------ */

/** Skeleton — Pulsing placeholder for streaming assistant messages.
 *  Uses animate-pulse + vscode panel-border for shimmer effect.
 */
export const Skeleton: React.FC<{ lines?: number }> = ({ lines = 3 }) => {
  return (
    <div className="space-y-2 w-full">
      {Array.from({ length: lines }).map((_, i) => (
        <div
          key={i}
          className="h-3 rounded animate-pulse"
          style={{
            backgroundColor: 'var(--vscode-panel-border)',
            width: `${60 + Math.random() * 35}%`,
            opacity: 0.4,
          }}
        />
      ))}
    </div>
  );
};

/** loadingAnimation keyframes injected via inline style for the spinner.
 *  Matches Terminal Solarized S_CYAN (primary accent).
 */
const spinKeyframes = `
  @keyframes loadingAnimation-spin {
    from { transform: rotate(0deg); }
    to   { transform: rotate(360deg); }
  }
`;

/** LoadingSpinner — Rotating ring used in ThinkingTrace and global loading.
 *  Color derived from --vscode-textLink-foreground (maps to S_CYAN).
 */
export const LoadingSpinner: React.FC<{ size?: number }> = ({ size = 16 }) => {
  return (
    <>
      <style>{spinKeyframes}</style>
      <div
        className="inline-block rounded-full border-2 border-transparent"
        style={{
          width: size,
          height: size,
          borderTopColor: 'var(--vscode-textLink-foreground, #2aa198)',
          borderRightColor: 'var(--vscode-textLink-foreground, #2aa198)',
          animation: 'loadingAnimation-spin 0.8s linear infinite',
        }}
      />
    </>
  );
};

/** EmptyState — Friendly illustration when no messages exist.
 *  Guides the user with quick-start hints.
 */
export const EmptyState: React.FC = () => {
  return (
    <div className="flex flex-col items-center justify-center h-full select-none">
      {/* Friendly robot SVG illustration */}
      <svg
        width="64"
        height="64"
        viewBox="0 0 24 24"
        fill="none"
        stroke="currentColor"
        strokeWidth="1.2"
        className="mb-4 opacity-40"
        style={{ color: 'var(--vscode-textLink-foreground, #2aa198)' }}
      >
        <rect x="4" y="8" width="16" height="10" rx="2" />
        <path d="M9 12h.01M15 12h.01" strokeLinecap="round" />
        <path d="M12 16v2M8 20h8" strokeLinecap="round" />
        <path d="M8 8V6a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2" />
      </svg>

      <h3
        className="text-sm font-semibold mb-1"
        style={{ color: 'var(--vscode-foreground)' }}
      >
        Welcome to Hajimi
      </h3>
      <p
        className="text-xs text-center max-w-[220px] leading-relaxed mb-3"
        style={{ color: 'var(--vscode-descriptionForeground)' }}
      >
        Your AI pair programmer. Type a message, use <span className="font-mono text-[10px] px-1 rounded" style={{ background: 'var(--vscode-textCodeBlock-background)' }}>/build</span>, or mention a file with <span className="font-mono text-[10px] px-1 rounded" style={{ background: 'var(--vscode-textCodeBlock-background)' }}>@file</span>.
      </p>

      <div className="flex gap-2">
        <span className="text-[10px] px-2 py-1 rounded-full" style={{ background: 'var(--vscode-badge-background)', color: 'var(--vscode-badge-foreground)' }}>
          🤖 AI Assistant
        </span>
        <span className="text-[10px] px-2 py-1 rounded-full" style={{ background: 'var(--vscode-badge-background)', color: 'var(--vscode-badge-foreground)' }}>
          ⚡ Real-time
        </span>
      </div>
    </div>
  );
};

/** TraceSkeleton — Compact skeleton for the ThinkingTrace panel.
 *  Shows 7 animated bars representing the AgentLoop steps.
 */
export const TraceSkeleton: React.FC = () => {
  const steps = ['Observe', 'Retrieve', 'Plan', 'Act', 'Reflect', 'Store', 'Decide'];
  return (
    <div className="space-y-1.5 p-2">
      {steps.map((s, i) => (
        <div key={s} className="flex items-center gap-2">
          <div
            className="h-2 w-2 rounded-full animate-pulse"
            style={{
              backgroundColor: 'var(--vscode-textLink-foreground)',
              opacity: 0.3 + (i / steps.length) * 0.5,
              animationDelay: `${i * 120}ms`,
            }}
          />
          <div
            className="h-2 rounded animate-pulse"
            style={{
              backgroundColor: 'var(--vscode-panel-border)',
              width: `${50 + Math.random() * 40}%`,
              opacity: 0.35,
              animationDelay: `${i * 120}ms`,
            }}
          />
        </div>
      ))}
    </div>
  );
};

/** GlobalLoadingOverlay — Full-panel overlay for heavy operations.
 *  Used when applying edits or loading workspace files.
 */
export const GlobalLoadingOverlay: React.FC<{ text?: string }> = ({ text = 'Working...' }) => {
  return (
    <div
      className="absolute inset-0 z-50 flex flex-col items-center justify-center gap-3"
      style={{ backgroundColor: 'var(--vscode-editor-background)', opacity: 0.92 }}
    >
      <LoadingSpinner size={24} />
      <span className="text-xs" style={{ color: 'var(--vscode-descriptionForeground)' }}>
        {text}
      </span>
    </div>
  );
};
