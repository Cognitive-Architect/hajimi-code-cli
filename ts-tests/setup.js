// FFI Test Setup - 真实FFI测试环境初始化
// 加载编译好的.node文件

const path = require('path');
const fs = require('fs');

// 设置NODE_PATH包含编译输出
const addonPath = path.join(__dirname, '../crates/codex-twist/index.node');
process.env.CODEX_TWIST_ADDON_PATH = addonPath;

console.log('FFI Test Setup:');
console.log('  Addon path:', addonPath);

// 检查.node文件是否存在
if (fs.existsSync(addonPath)) {
  console.log('  Status: .node file found ✓');
} else {
  console.log('  Status: .node file NOT found ✗');
  console.log('  Run: npm run build');
}
