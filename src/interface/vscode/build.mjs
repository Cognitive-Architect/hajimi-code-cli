import * as esbuild from 'esbuild';
import * as fs from 'fs';
import * as path from 'path';
import { fileURLToPath } from 'url';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const watch = process.argv.includes('--watch');
const outdir = path.resolve(__dirname, 'out', 'webview');

if (!fs.existsSync(outdir)) {
  fs.mkdirSync(outdir, { recursive: true });
}

const ctx = await esbuild.context({
  entryPoints: [path.resolve(__dirname, 'webview', 'src', 'index.tsx')],
  bundle: true,
  outdir,
  format: 'iife',
  target: 'es2020',
  minify: !watch,
  sourcemap: true,
  jsx: 'automatic',
  loader: { '.tsx': 'tsx', '.ts': 'ts', '.css': 'css' },
  alias: {
    '@': path.resolve(__dirname, 'webview', 'src'),
  },
});

if (watch) {
  await ctx.watch();
  console.log('[build:webview] Watching for changes...');
} else {
  await ctx.rebuild();
  await ctx.dispose();
  console.log('[build:webview] Build complete:', outdir);
}
