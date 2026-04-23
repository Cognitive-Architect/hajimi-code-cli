import { describe, it } from 'node:test';
import { strict as assert } from 'node:assert';
import type { WebviewToExtensionMessage, ExtensionToWebviewMessage } from '../../src/interface/vscode/webview/src/types/webview';

function expect(actual: unknown) {
  return {
    toBe(expected: unknown) { assert.strictEqual(actual, expected); },
    toBeTruthy() { assert.ok(actual); },
    toContain(sub: string) { assert.ok(String(actual).includes(sub)); },
  };
}

describe('E2E-W4: Apply + Feedback + Undo loop', () => {
  const extToWebview: ExtensionToWebviewMessage[] = [];
  const webviewToExt: WebviewToExtensionMessage[] = [];
  function postToWebview(msg: ExtensionToWebviewMessage) { extToWebview.push(msg); }
  function postToExtension(msg: WebviewToExtensionMessage) { webviewToExt.push(msg); }

  it('Accept → submitFeedback sent', () => {
    postToExtension({ type: 'submitFeedback', payload: { messageId: 'm1', choice: 'accept', context: { query: 'hello' } } });
    expect(webviewToExt.length).toBe(1);
    expect(webviewToExt[0].type).toBe('submitFeedback');
    expect((webviewToExt[0].payload as { choice: string }).choice).toBe('accept');
  });

  it('Reject → submitFeedback sent', () => {
    postToExtension({ type: 'submitFeedback', payload: { messageId: 'm2', choice: 'reject', context: { query: 'fix bug' } } });
    expect(webviewToExt[webviewToExt.length - 1].type).toBe('submitFeedback');
  });

  it('Explain → submitFeedback sent', () => {
    postToExtension({ type: 'submitFeedback', payload: { messageId: 'm3', choice: 'explain', reason: 'why?', context: { query: 'explain' } } });
    const last = webviewToExt[webviewToExt.length - 1];
    expect(last.type).toBe('submitFeedback');
    expect((last.payload as { choice: string }).choice).toBe('explain');
  });

  it('Undo → requestUndo sent', () => {
    postToExtension({ type: 'requestUndo', payload: {} });
    expect(webviewToExt[webviewToExt.length - 1].type).toBe('requestUndo');
  });

  it('Feedback result → webview receives feedbackResult', () => {
    postToWebview({ type: 'feedbackResult', payload: { success: true, storedCount: 3 } });
    expect(extToWebview[extToWebview.length - 1].type).toBe('feedbackResult');
  });
});
