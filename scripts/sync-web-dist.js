const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..');
const webRoot = path.join(repoRoot, 'src', 'interface', 'web');
const distRoot = path.join(webRoot, 'dist', 'dist');

const files = [
  'index.html',
  'style.css',
  'app.js',
  'logo.jpg',
];

const dirs = [
  'modules',
];

function copyFile(relativePath) {
  const source = path.join(webRoot, relativePath);
  const target = path.join(distRoot, relativePath);

  if (!fs.existsSync(source)) {
    throw new Error(`Missing web asset: ${source}`);
  }

  fs.mkdirSync(path.dirname(target), { recursive: true });
  fs.copyFileSync(source, target);
}

function copyDir(relativePath) {
  const source = path.join(webRoot, relativePath);
  const target = path.join(distRoot, relativePath);

  if (!fs.existsSync(source)) {
    throw new Error(`Missing web asset directory: ${source}`);
  }

  fs.rmSync(target, { recursive: true, force: true });
  fs.cpSync(source, target, {
    recursive: true,
    filter: (entry) => !entry.includes(`${path.sep}node_modules${path.sep}`),
  });
}

fs.mkdirSync(distRoot, { recursive: true });

for (const file of files) {
  copyFile(file);
}

for (const dir of dirs) {
  copyDir(dir);
}

console.log(`Synced web release assets to ${distRoot}`);
