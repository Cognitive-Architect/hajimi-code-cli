import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';
import { SidebarProvider } from './providers/SidebarProvider';
import { getThemeManager } from './theme/ThemeManager';

// Acquire VSCode API once at module level (only valid inside Webview)
declare function acquireVsCodeApi(): {
  postMessage(msg: unknown): void;
  getState(): unknown;
  setState(state: unknown): void;
};

export const vscodeApi = acquireVsCodeApi();

// Week 6: Initialize unified theme (Terminal Solarized ↔ VSCode) before first render
const themeManager = getThemeManager();
themeManager.listenVSCodeTheme();

const container = document.getElementById('root');
if (!container) {
  throw new Error('Root element #root not found in Webview HTML');
}

const root = createRoot(container);
root.render(
  <StrictMode>
    <SidebarProvider />
  </StrictMode>
);
