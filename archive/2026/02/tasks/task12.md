🚀 饱和攻击波次：HAJIMI-PHASE1-DEBT-CLEARANCE-001

火力配置：5 Agent 并行（或1 Agent串行，视额度而定）

轰炸目标：12号审计发现的3项新债务+2项P1问题 → 产出《PHASE1-DEBT-CLEARED-白皮书-v1.0.md》+《PHASE1-DEBT-CLEARED-自测表-v1.0.md》

输入基线：ID-179（12号审计报告）+ GitHub坐标 `https://github.com/Cognitive-Architect/hajimi-code-cli` + commit `e5a7e5d`

质量门禁（未满足禁止开工）：  
- 《P4自测轻量检查表》10/10项已预读（见下方Template）  
- 已读取ID-179审计报告，明确5项待清偿债务位置  
- 债务清偿方案不涉及破坏性重构（仅增强/补充）  

---

【P4自测轻量检查表·Phase 1特化版】
（Engineer填写，每项必须手写[x]并附证据）

CHECK_ID	检查项	覆盖情况（Engineer填）	证据指针	
P4-P1-001	.gitignore已添加并提交	[ ]	`git status` 显示新增`.gitignore`	
P4-P1-002	CORS配置可自定义origin	[ ]	`grep corsOrigin src/api/server.js` 命中	
P4-P1-003	2处TODO已处理或转债务卡片	[ ]	代码中无TODO标记或债务文档已更新	
P4-P1-004	配置校验增强已实现	[ ]	非法port启动时抛出错误	
P4-P1-005	请求ID追踪中间件已注入	[ ]	日志输出含UUID格式requestId	
P4-P1-006	全部5项变更通过单元测试	[ ]	`npm test` 全绿（新增测试≥5项）	
P4-P1-007	安全扫描无新增漏洞	[ ]	`grep -r "api_key\|password" src/` 0命中	
P4-P1-008	文档已更新（CHANGELOG+债务清偿证明）	[ ]	`docs/debt/DEBT-CLEARANCE-v2.0.md`存在	
P4-P1-009	Git提交符合规范（conventional commits）	[ ]	`git log --oneline` 显示`fix:`/`feat:`前缀	
P4-P1-010	无破坏性变更（向后兼容）	[ ]	旧配置文件仍能启动（默认回退）	

---

【工单矩阵·五债并行】

工单 B-01/05 DEBT-AUDIT-001 → 根目录.gitignore缺失  
目标：添加标准Node.js .gitignore，防止敏感文件意外提交

输入：  
- 审计证据：`docs/audit report/12/12-AUDIT-EXPLORATION-建设性审计报告.md` FUNC-001节  
- 参考模板：GitHub Node.js.gitignore标准模板

输出：  
- `.gitignore`（新增，位于项目根目录）  
- 必须包含：`.env*`, `node_modules/`, `*.log`, `dist/`, `.DS_Store`  
- 必须排除：`src/`, `docs/`, `crates/`, `README.md`（whitelist模式）

自测点：  
- GIT-001: `git check-ignore -v .env.local` 返回匹配规则  
- GIT-002: `git check-ignore -v src/index.js` 返回未匹配  
- GIT-003: 已执行`git add .gitignore && git commit -m "chore: add .gitignore"`  

工单 B-02/05 DEBT-AUDIT-002 → HTTP CORS配置过宽  
目标：`src/api/server.js:153`处`Access-Control-Allow-Origin: *`改为可配置，默认localhost

输入：  
- 代码基线：`src/api/server.js` 第153行（或搜索cors关键字）  
- 配置范式：现有`config.port`配置方式参考

输出：  
- `src/api/server.js`（修改，第153行附近）  
- `src/config/cors.js`（新增，或并入现有config）  
- 必须实现：  
  
```javascript
  // 默认只允许localhost，生产环境可配置
  corsOrigin: config.corsOrigin || 'http://localhost:3000'
  // 如果设为'*'则保持原行为（显式允许）
  ```  

自测点：  
- CORS-001: 默认配置下，非localhost请求被拒绝（403）  
- CORS-002: 设置`corsOrigin: '*'`后，任意来源允许（与原行为一致）  
- CORS-003: 设置`corsOrigin: 'http://example.com'`后，仅该来源允许  

工单 B-03/05 DEBT-AUDIT-003 → 2处TODO标记处理  
目标：处理`src/vector/hnsw-core.js`和`src/vector/hybrid-retriever.js`中的TODO

输入：  
- 代码扫描：`grep -n "TODO\|FIXME" src/vector/*.js`  
- 债务文档：`docs/debt/`目录现有格式参考

输出：  
- 若实现简单（<30min）：直接修复代码，删除TODO注释  
- 若实现复杂：创建债务卡片`docs/debt/DEBT-TODO-001.md`和`DEBT-TODO-002.md`  
- 必须包含：TODO位置、问题描述、预计清偿版本、技术路径

自测点：  
- TODO-001: `grep -r "TODO\|FIXME" src/vector/` 返回0结果（若已修复）或债务文档存在（若延后）  
- TODO-002: 债务卡片符合ID-133审计归档规范（含ID、优先级、排期）  

工单 B-04/05 FUNC-001 → 启动配置校验增强  
目标：`src/api/server.js`的`start()`方法增加port/host合法性校验

输入：  
- 代码基线：`src/api/server.js` start()方法  
- 校验规则：port必须为1-65535整数，host必须为合法IP或域名

输出：  
- `src/api/server.js`（修改，start()方法入口）  
- 必须实现：  
  
```javascript
  if (!Number.isInteger(port) || port < 1 || port > 65535) {
    throw new HajimiError('INVALID_CONFIG', `Invalid port: ${port}`);
  }
  // host可选校验（非空即可）
  ```  

自测点：  
- CFG-001: 启动时port=0抛出错误（Exit非0）  
- CFG-002: 启动时port=70000抛出错误  
- CFG-003: 启动时port=3000正常监听  

工单 B-05/05 FUNC-002 → 请求ID追踪中间件  
目标：在请求处理链中注入UUID生成，记录于日志上下文

输入：  
- 中间件位置：`src/api/middleware/`目录（或新建）  
- UUID生成：使用原生`crypto.randomUUID()`（Node.js≥14.17）或轻量级uuid库  
- 日志系统：现有日志实现方式（console或自定义）

输出：  
- `src/api/middleware/request-id.js`（新增）  
- `src/api/server.js`（修改，中间件注册）  
- 必须实现：  
  
```javascript
  // 中间件，为每个请求生成唯一ID
  req.requestId = crypto.randomUUID();
  res.setHeader('X-Request-Id', req.requestId);
  // 后续日志自动携带此ID
  ```  

自测点：  
- REQ-001: 请求头返回`X-Request-Id`（UUID v4格式）  
- REQ-002: 日志输出包含相同requestId  
- REQ-003: 并发请求下ID唯一（100次请求无重复）  

---

【刀刃风险自测表·16项（Phase 1特化）】

用例ID	类别	场景	验证命令（可复制）	通过标准	状态（Engineer填）	
DEBT-001	FUNC	.gitignore生效	`git check-ignore -v .env.local`	返回匹配行	[ ]	
DEBT-002	FUNC	CORS默认拒绝	`curl -H "Origin: http://evil.com" http://localhost:3000/health`	返回403或无CORS头	[ ]	
DEBT-003	FUNC	CORS允许配置	修改config.corsOrigin=''后重试	返回200含CORS头	[ ]	
DEBT-004	FUNC	TODO已清理	`grep -r "TODO" src/vector/`	返回空或债务文档存在	[ ]	
DEBT-005	FUNC	配置校验拦截非法port	`node -e "require('./src/api/server').start({port:0})"`	抛出错误Exit非0	[ ]	
DEBT-006	FUNC	请求ID生成	`curl -I http://localhost:3000/health \| grep -i x-request-id`	返回UUID格式	[ ]	
DEBT-007	CONST	向后兼容	使用旧版config（无corsOrigin）启动	正常启动（默认localhost）	[ ]	
DEBT-008	CONST	无硬编码密钥	`grep -ri "api_key\|secret\|password" src/api/`	0命中	[ ]	
DEBT-009	NEG	.gitignore不屏蔽源码	`git check-ignore -v src/index.js`	返回未匹配	[ ]	
DEBT-010	NEG	无效host处理	`node -e "require('./src/api/server').start({host:'invalid host!'})"`	优雅拒绝或回退	[ ]	
DEBT-011	UX	错误提示可读	触发CFG-001错误	错误信息含"Invalid port"	[ ]	
DEBT-012	E2E	全链路启动	`npm start`（整合全部5项变更）	正常启动，日志含requestId	[ ]	
DEBT-013	E2E	审计验证	`ls docs/debt/DEBT-CLEARANCE-v2.0.md`	文件存在	[ ]	
DEBT-014	HIGH	安全基线保持	`grep -r "Access-Control-Allow-Origin: \\*" src/`	无硬编码通配符（仅配置中允许）	[ ]	
DEBT-015	HIGH	债务诚实	检查B-03/05产出（TODO处理或债务卡片）	不隐瞒TODO，不虚假删除	[ ]	
DEBT-016	SELF	自测全绿	`npm test`（含新增5项测试）	全部通过	[ ]

【D级红线（地狱难度，触发即永久失败）】
1. 
未添加.gitignore或添加后仍提交敏感文件 → 永久禁用
2. 
CORS改为硬编码localhost但破坏原有 * 灵活性（未保留配置项） → 永久禁用
3. 
隐瞒TODO未处理且未创建债务卡片 → 永久禁用
4. 
配置校验导致合法配置无法启动（过度校验） → 永久禁用
5. 
请求ID生成依赖外部库（增加依赖债务） → 永久禁用（必须用原生crypto）
6. 
任何⬜标记为[x]但实际未验证 → 永久禁用
7. 
未产出DEBT-CLEARANCE-v2.0.md文档 → 永久禁用
8. 
Git提交不符合conventional commits规范 → 打回重写

【验收标准（数值化）】

验收项	验收命令	通过标准	失败标准	
5项债务清偿	`ls docs/debt/DEBT-CLEARANCE-v2.0.md`	文件存在且列出5项清偿证明	缺失	
.gitignore	`git check-ignore -v .env.local`	返回规则	未匹配	
CORS配置	`grep corsOrigin src/api/server.js`	命中配置项	未命中	
配置校验	`node -e "require('./src/api/server').start({port:0})"` 2>&1	抛出错误	静默启动	
请求ID	`curl -s -I http://localhost:3000/health \| grep -i x-request-id`	返回UUID	缺失	
单元测试	`npm test`	100%通过（含新增≥5项）	任何失败	
安全扫描	`grep -ri "api_key\|secret" src/`	0命中	0命中	
债务诚实	`grep -c "TODO\|FIXME" src/vector/*.js`	0（或债务文档存在）	0且无文档


【收卷强制交付物（5件套）】
1. 
Git提交： git log --oneline -5  显示5条conventional commits
2. 
债务清偿证明： docs/debt/DEBT-CLEARANCE-v2.0.md （5项清偿记录+验证命令）
3. 
自测表：《PHASE1-DEBT-CLEARED-自测表-v1.0.md》（16项刀刃+10项P4，全部手动[x]）
4. 
白皮书：《PHASE1-DEBT-CLEARED-白皮书-v1.0.md》（4章：背景/清偿过程/验证结果/剩余债务）
5. 
回归验证： npm test  截图/日志（全绿）