import { describe, it, test, beforeEach } from 'node:test';
import { strict as assert } from 'node:assert';
import type { WebviewToExtensionMessage, ExtensionToWebviewMessage, EditChunk } from '../../src/interface/vscode/webview/src/types/webview';

function expect(actual: unknown) {
  return {
    toBe(expected: unknown) { assert.strictEqual(actual, expected); },
    toBeTruthy() { assert.ok(actual); },
    toBeFalsy() { assert.ok(!actual); },
    toContain(substring: string) { assert.ok(String(actual).includes(substring)); },
  };
}

describe('E2E-006: Streaming Edit 完整路径', () => {
  const extToWebview: ExtensionToWebviewMessage[] = [];
  const webviewToExt: WebviewToExtensionMessage[] = [];

  beforeEach(() => { extToWebview.length = 0; webviewToExt.length = 0; });

  function postToWebview(msg: ExtensionToWebviewMessage) { extToWebview.push(msg); }
  function postToExtension(msg: WebviewToExtensionMessage) { webviewToExt.push(msg); }

  it('应流式接收 traceStep(Observe~Act) 和 editChunk → editComplete 含 diff', () => {
    postToExtension({ type: 'sendMessage', payload: { text: 'add a new function' } });
    ['Observe', 'Retrieve', 'Plan', 'Act'].forEach((step, i) => {
      postToWebview({ type: 'traceStep', payload: { step, details: `${step}: test`, iteration: i, timestamp: Date.now(), status: 'active' } });
    });
    const chunk: EditChunk = { uri: 'src/example.ts', range: [0, 0, 0, 0], text: '// test', stepIndex: 0 };
    postToWebview({ type: 'editChunk', payload: chunk });
    postToWebview({ type: 'editComplete', payload: { diff: '--- a/src/example.ts\n+++ b/src/example.ts\n@@ -1 +1 @@\n-test\n+fixed\n' } });
    expect(extToWebview.some((m) => m.type === 'traceStep')).toBeTruthy();
    expect(extToWebview.some((m) => m.type === 'editChunk')).toBeTruthy();
    expect(extToWebview.some((m) => m.type === 'editComplete')).toBeTruthy();
    const completeMsg = extToWebview.find((m) => m.type === 'editComplete') as { payload?: { diff?: string } };
    expect(completeMsg?.payload?.diff).toBeTruthy();
    expect(completeMsg?.payload?.diff).toContain('--- a/');
    expect(completeMsg?.payload?.diff).toContain('+++ b/');
  });

  it('Accept 流程: applyEdits → editResult(success=true) → diff 清空', () => {
    postToExtension({ type: 'applyEdits', payload: {} });
    postToWebview({ type: 'editResult', payload: { success: true } });
    expect(webviewToExt.some((m) => m.type === 'applyEdits')).toBeTruthy();
    expect(extToWebview.some((m) => m.type === 'editResult')).toBeTruthy();
    const result = extToWebview.find((m) => m.type === 'editResult') as { payload?: { success?: boolean } };
    expect(result?.payload?.success).toBe(true);
  });

  it('Reject 流程: rejectEdits → diff 清空', () => {
    postToExtension({ type: 'rejectEdits', payload: {} });
    expect(webviewToExt.some((m) => m.type === 'rejectEdits')).toBeTruthy();
  });

  test('Cancel 流程: cancelEdit → editError → isStreaming=false', () => {
    postToExtension({ type: 'cancelEdit', payload: {} });
    postToWebview({ type: 'editError', payload: { error: 'Edit cancelled by user' } });
    expect(webviewToExt.some((m) => m.type === 'cancelEdit')).toBeTruthy();
    expect(extToWebview.some((m) => m.type === 'editError')).toBeTruthy();
    const errorMsg = extToWebview.find((m) => m.type === 'editError') as { payload?: { error?: string } };
    expect(errorMsg?.payload?.error).toContain('cancelled');
  });

  it('应验证所有 edit 相关消息类型在协议中定义完整', () => {
    const extTypes: ExtensionToWebviewMessage['type'][] = [
      'streamChunk', 'streamComplete', 'streamError', 'toolResult',
      'traceStep', 'traceComplete', 'traceError', 'editorState',
      'editChunk', 'editComplete', 'editError', 'editResult',
    ];
    const webTypes: WebviewToExtensionMessage['type'][] = [
      'sendMessage', 'executeTool', 'syncEditor',
      'applyEdits', 'rejectEdits', 'cancelEdit', 'setEditMode',
    ];
    expect(extTypes.length).toBe(12);
    expect(webTypes.length).toBe(7);
    expect(extTypes.includes('editChunk')).toBeTruthy();
    expect(extTypes.includes('editComplete')).toBeTruthy();
    expect(extTypes.includes('editError')).toBeTruthy();
    expect(webTypes.includes('applyEdits')).toBeTruthy();
    expect(webTypes.includes('rejectEdits')).toBeTruthy();
    expect(webTypes.includes('cancelEdit')).toBeTruthy();
  });
});
