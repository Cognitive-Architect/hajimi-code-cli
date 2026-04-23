/** Feedback types for the Accept/Reject/Explain user feedback system. */

export type FeedbackChoice = 'accept' | 'reject' | 'explain';

/** Single feedback item collected from the user after an AI response. */
export interface FeedbackItem {
  messageId: string;
  choice: FeedbackChoice;
  reason?: string;
  context: FeedbackContext;
  timestamp: number;
}

/** Context captured at feedback time for governance and memory routing. */
export interface FeedbackContext {
  uri?: string;
  query: string;
  traceSteps: string[];
  diffLength?: number;
}

/** Batch envelope for flushing feedback to the LSP backend. */
export interface FeedbackBatch {
  items: FeedbackItem[];
  deviceId: string;
  sessionId: string;
}

/** Result returned after submitting feedback. */
export interface FeedbackResult {
  success: boolean;
  storedCount: number;
  errors?: string[];
}
