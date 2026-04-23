/** ------------------------------------------------------------------
 *  ThemeManager — Bridges Terminal Solarized palette with VSCode CSS variables.
 *  Week 6 Polishing: unifiedTheme for dark/light coordination.
 *  No tailwind.config needed; variables are injected at runtime.
 * ------------------------------------------------------------------ */

/** Terminal Solarized palette (from ratatui theme.rs).
 *  These map to the Rust terminal's exact colors.
 */
const SOLARIZED = {
  dark: {
    base03: '#002b36',
    base02: '#073642',
    base01: '#586e75',
    base00: '#657b83',
    base0:  '#839496',
    base1:  '#93a1a1',
    base2:  '#eee8d5',
    base3:  '#fdf6e3',
    yellow: '#b58900',
    orange: '#cb4b16',
    red:    '#dc322f',
    magenta:'#d33682',
    violet: '#6c71c4',
    blue:   '#268bd2',
    cyan:   '#2aa198',
    green:  '#859900',
  },
  light: {
    base03: '#fdf6e3',
    base02: '#eee8d5',
    base01: '#93a1a1',
    base00: '#839496',
    base0:  '#657b83',
    base1:  '#586e75',
    base2:  '#073642',
    base3:  '#002b36',
    yellow: '#b58900',
    orange: '#cb4b16',
    red:    '#dc322f',
    magenta:'#d33682',
    violet: '#6c71c4',
    blue:   '#268bd2',
    cyan:   '#2aa198',
    green:  '#859900',
  },
} as const;

/** vscodeColor mapping: which VSCode CSS variable receives which Solarized color.
 *  This is the unifiedTheme bridge table.
 */
const VSCodeColorMap = {
  '--hajimi-primary':   'cyan',
  '--hajimi-success':   'green',
  '--hajimi-error':     'red',
  '--hajimi-warning':   'yellow',
  '--hajimi-muted':     'base01',
  '--hajimi-fg':        'base0',
  '--hajimi-bg':        'base03',
  '--hajimi-bg-panel':  'base02',
} as const;

type ThemeMode = 'dark' | 'light';

/** Unified palette object exposed to components. */
export interface UnifiedPalette {
  primary: string;
  success: string;
  error: string;
  warning: string;
  muted: string;
  fg: string;
  bg: string;
  bgPanel: string;
}

/** ThemeManager — Detects VSCode theme mode and injects unified CSS variables.
 *  Coordinates Terminal Solarized with VSCode's native dark/light tokens.
 */
export class ThemeManager {
  private mode: ThemeMode = 'dark';
  private observer?: MutationObserver;

  /** Apply a theme mode immediately (dark or light). */
  public applyTheme(mode: ThemeMode): void {
    this.mode = mode;
    const palette = SOLARIZED[mode];
    const root = document.documentElement;

    // Inject mapped Solarized colors as custom properties
    (Object.entries(VSCodeColorMap) as [string, keyof typeof SOLARIZED.dark][]).forEach(([cssVar, colorKey]) => {
      root.style.setProperty(cssVar, palette[colorKey]);
    });

    // Also set a data attribute for tailwind darkMode selectors
    root.setAttribute('data-hajimi-theme', mode);

    // Persist preference
    try {
      localStorage.setItem('hajimi.theme', mode);
    } catch { /* silent */ }
  }

  /** Detect VSCode's current theme by reading body classes.
   *  VSCode sets 'vscode-dark' or 'vscode-light' on the webview body.
   */
  public detectVSCodeTheme(): ThemeMode {
    const bodyClass = document.body?.className ?? '';
    if (bodyClass.includes('vscode-light')) return 'light';
    if (bodyClass.includes('vscode-high-contrast')) return 'dark';
    return 'dark';
  }

  /** Listen for VSCode theme changes via MutationObserver.
   *  VSCode toggles the body class when the user switches themes.
   */
  public listenVSCodeTheme(): void {
    // Initial apply
    const initial = this.detectVSCodeTheme();
    this.applyTheme(initial);

    // Watch for class changes on <body>
    this.observer = new MutationObserver((mutations) => {
      for (const m of mutations) {
        if (m.type === 'attributes' && m.attributeName === 'class') {
          const detected = this.detectVSCodeTheme();
          if (detected !== this.mode) {
            this.applyTheme(detected);
          }
        }
      }
    });

    this.observer.observe(document.body, { attributes: true, attributeFilter: ['class'] });
  }

  /** Get the current unified palette for programmatic use. */
  public getUnifiedPalette(): UnifiedPalette {
    const p = SOLARIZED[this.mode];
    return {
      primary: p.cyan,
      success: p.green,
      error: p.red,
      warning: p.yellow,
      muted: p.base01,
      fg: p.base0,
      bg: p.base03,
      bgPanel: p.base02,
    };
  }

  /** Toggle between dark and light manually (for testing or user override). */
  public toggle(): ThemeMode {
    const next = this.mode === 'dark' ? 'light' : 'dark';
    this.applyTheme(next);
    return next;
  }

  /** Clean up the MutationObserver. */
  public dispose(): void {
    this.observer?.disconnect();
  }
}

/** Singleton instance for the webview. */
let globalManager: ThemeManager | null = null;

export function getThemeManager(): ThemeManager {
  if (!globalManager) {
    globalManager = new ThemeManager();
  }
  return globalManager;
}
