const assert = require('assert');
const fs = require('fs');
const path = require('path');

const repoRoot = path.resolve(__dirname, '..', '..');
const appJsPath = path.join(repoRoot, 'src/interface/web/app.js');
const appJs = fs.readFileSync(appJsPath, 'utf8');

const resultBranch = appJs.match(
  /} else if \(event\.type === 'result'\) \{[\s\S]*?\n    } else if \(event\.type === 'error'\)/
);

assert(resultBranch, 'agent result branch should be discoverable');
assert(
  resultBranch[0].includes('outcome.trim()'),
  'non-empty result output should be detected'
);
assert(
  resultBranch[0].includes('智能体任务已成功完成！\\n\\n${displayBody}'),
  'non-empty result output should render cleaned displayBody below the success header'
);
assert(
  resultBranch[0].includes('isSuccess = true'),
  'non-empty result output should keep the task in completed state'
);
assert(
  resultBranch[0].includes("outcome === 'BudgetExceeded'"),
  'BudgetExceeded handling should remain explicit'
);
assert(
  resultBranch[0].includes("outcome && outcome.startsWith('ActFailed')"),
  'ActFailed handling should remain explicit'
);
assert(
  resultBranch[0].includes('智能体在执行动作时失败'),
  'ActFailed should still render as a failure message'
);

console.log('agent result rendering smoke: PASS');
