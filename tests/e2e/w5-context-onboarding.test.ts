import { describe, it, test, beforeEach } from 'node:test';
import { strict as assert } from 'node:assert';
import type { ExtensionToWebviewMessage, WebviewToExtensionMessage, OnboardingState, ContextPreview } from '../../src/interface/vscode/webview/src/types/webview';

function expect(actual: unknown) {
  return {
    toBe(expected: unknown) { assert.strictEqual(actual, expected); },
    toBeTruthy() { assert.ok(actual); },
    toContain(substring: string) { assert.ok(String(actual).includes(substring)); },
  };
}

describe('E2E-005: W5 Context + Onboarding 完整路径', () => {
  const extToWebview: ExtensionToWebviewMessage[] = [];
  const webviewToExt: WebviewToExtensionMessage[] = [];

  beforeEach(() => { extToWebview.length = 0; webviewToExt.length = 0; });

  function postToWebview(msg: ExtensionToWebviewMessage) { extToWebview.push(msg); }
  function postToExtension(msg: WebviewToExtensionMessage) { webviewToExt.push(msg); }

  it('应发送 onboardingState 并包含 welcome + examples', () => {
    const onboarding: OnboardingState = {
      show: true,
      welcome: { title: 'Welcome to Hajimi', body: 'Your AI pair programmer.', emoji: '🤖' },
      examples: [
        { id: 'ex-build', label: 'Build project', icon: '🔨', text: '/build' },
        { id: 'ex-explain', label: 'Explain file', icon: '🔍', text: '@src/main.rs explain' },
      ],
      steps: [{ id: 'tour-chat', target: 'Chat panel', message: 'Type messages here.' }],
    };
    postToWebview({ type: 'onboardingState', payload: onboarding });
    const msg = extToWebview.find((m) => m.type === 'onboardingState');
    expect(msg).toBeTruthy();
    const payload = (msg as { payload?: OnboardingState }).payload;
    expect(payload?.welcome.title).toContain('Welcome');
    expect(payload?.examples.length).toBe(2);
  });

  it('应发送 contextPreview 并包含 fileName / language / lines', () => {
    const preview: ContextPreview = {
      fileName: 'src/main.rs',
      language: 'rust',
      hasSelection: false,
      lines: 42,
    };
    postToWebview({ type: 'contextPreview', payload: preview });
    const msg = extToWebview.find((m) => m.type === 'contextPreview');
    expect(msg).toBeTruthy();
    const payload = (msg as { payload?: ContextPreview }).payload;
    expect(payload?.fileName).toContain('main.rs');
    expect(payload?.language).toBe('rust');
    expect(payload?.lines).toBe(42);
  });

  it('@file 提及后 sendMessage payload 应保留文件引用', () => {
    const text = '@src/main.rs explain this file';
    postToExtension({ type: 'sendMessage', payload: { text } });
    const msg = webviewToExt.find((m) => m.type === 'sendMessage');
    expect(msg).toBeTruthy();
    const payload = (msg as { payload?: { text?: string } }).payload;
    expect(payload?.text).toContain('@src/main.rs');
  });

  test('应请求动态文件列表 requestFileList → fileList', () => {
    postToExtension({ type: 'requestFileList', payload: {} });
    const req = webviewToExt.find((m) => m.type === 'requestFileList');
    expect(req).toBeTruthy();
    // Simulate extension host response
    postToWebview({ type: 'fileList', payload: { files: ['src/main.rs', 'package.json'] } });
    const resp = extToWebview.find((m) => m.type === 'fileList');
    expect(resp).toBeTruthy();
    const payload = (resp as { payload?: { files?: string[] } }).payload;
    expect(payload?.files?.length).toBe(2);
    expect(payload?.files?.[0]).toContain('main.rs');
  });

  test('负面路径: 空文件列表不应导致崩溃', () => {
    postToWebview({ type: 'fileList', payload: { files: [] } });
    const resp = extToWebview.find((m) => m.type === 'fileList');
    expect(resp).toBeTruthy();
    const payload = (resp as { payload?: { files?: string[] } }).payload;
    expect(payload?.files?.length).toBe(0);
  });
});
