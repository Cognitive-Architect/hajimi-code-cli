/** Message types sent FROM the React frontend TO the extension host */
export interface WebviewToExtensionMessage {
  type: 'sendMessage' | 'executeTool' | 'syncEditor' | 'applyEdits' | 'rejectEdits' | 'cancelEdit' | 'setEditMode' | 'submitFeedback' | 'requestUndo' | 'requestRestore' | 'requestFileList' | 'requestFolderList' | 'dismissOnboarding';
  payload?: unknown;
}

/** Message types sent FROM the extension host TO the React frontend */
export interface ExtensionToWebviewMessage {
  type: 'streamChunk' | 'streamComplete' | 'streamError' | 'toolResult' | 'traceStep' | 'traceComplete' | 'traceError' | 'editorState' | 'editChunk' | 'editComplete' | 'editError' | 'editResult' | 'feedbackResult' | 'undoResult' | 'onboardingState' | 'contextPreview' | 'fileList' | 'folderList';
  payload?: unknown;
}

/** Onboarding state sent from extension host on first open. */
export interface OnboardingState {
  show: boolean;
  welcome: { title: string; body: string; emoji: string };
  examples: { id: string; label: string; icon: string; text: string }[];
  steps: { id: string; target: string; message: string }[];
}

/** Context preview sent from extension host for auto-injected context. */
export interface ContextPreview {
  fileName: string;
  language: string;
  hasSelection: boolean;
  lines: number;
}

/** Chat message shape */
export interface ChatMessage {
  id: string;
  role: 'user' | 'assistant' | 'system';
  content: string;
  timestamp: number;
  status?: 'sending' | 'streaming' | 'complete' | 'error';
}

/** Single trace step emitted by AgentLoop */
export interface TraceStep {
  step: 'Observe' | 'Retrieve' | 'Plan' | 'Act' | 'Reflect' | 'Store' | 'Decide' | 'Idle' | 'Completed' | 'Failed';
  details: string;
  iteration: number;
  timestamp: number;
  status: 'active' | 'completed' | 'error' | 'pending';
}

/** Editor state for timeline sync */
export interface EditorState {
  uri?: string;
  version?: number;
  selection?: { start: number; end: number };
  language?: string;
}

/** Tool definition aligned with CommandRegistry */
export interface ToolDef {
  id: string;
  name: string;
  icon: string;
  category: string;
}

/** Single incremental edit chunk streamed from the agent */
export interface EditChunk {
  uri: string;
  range: [number, number, number, number];
  text: string;
  stepIndex: number;
}

/** Diff statistics for UI display */
export interface DiffStats {
  insertions: number;
  deletions: number;
}

/** Edit state tracked in the webview */
export interface EditState {
  diff: string;
  isStreaming: boolean;
  error: string | null;
  mode: 'preview' | 'live';
}
