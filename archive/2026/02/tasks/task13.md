🚀 饱和攻击波次：HAJIMI-PHASE2-SECURITY-HARDENING-001

火力配置：4 Agent 串行（B-01→B-02→B-03→B-04，前完成后启动后）

轰炸目标：P2瑕疵修复 + Phase 2安全加固全量（限流/超时/安全头/日志）→ 产出《PHASE2-SECURITY-HARDENED-白皮书-v1.0.md》+《PHASE2-SECURITY-HARDENED-自测表-v1.0.md》

输入基线：ID-180（Phase 1完结态，Git坐标46e2877）+ ID-179（Phase 2路线图）

质量门禁（未满足禁止开工）：  
- 已读取ID-180，确认P2瑕疵位置（validateConfig未调用）  
- 已读取ID-179 Phase 2路线图（2周工期，4大模块）  
- 《P4自测轻量检查表》10/10项已预读（见下方Template）  
- 债务预声明已接受：DEBT-SEC-001（Rate Limiting内存存储，重启清零）→ P2  

---

【P4自测轻量检查表·Phase 2特化版】
（Engineer逐项手写[x]，附可验证证据）

CHECK_ID	检查项	覆盖情况（Engineer填）	证据指针	
P4-P2-001	P2瑕疵修复（validateConfig调用）	[ ]	`grep "_validateConfig" src/api/server.js` 命中start()内	
P4-P2-002	Token Bucket算法实现	[ ]	`src/security/rate-limiter.js` 存在且含consume/refill逻辑	
P4-P2-003	限流中间件集成	[ ]	`grep rate-limit src/api/server.js` 命中app.use	
P4-P2-004	API超时控制实现	[ ]	`grep timeout src/api/middleware/timeout.js` 命中	
P4-P2-005	安全响应头完整	[ ]	`curl -I localhost:3000` 显示X-Frame-Options等头	
P4-P2-006	结构化JSON日志	[ ]	日志输出为`{"level":"info","message":"..."}`格式	
P4-P2-007	限流触发测试（100req/min）	[ ]	101次请求时返回429状态码	
P4-P2-008	超时触发测试（30s）	[ ]	慢查询31秒时自动终止并返回504	
P4-P2-009	向后兼容（无破坏性变更）	[ ]	旧配置启动正常（限流/超时默认可用但不过度限制）	
P4-P2-010	债务诚实声明（DEBT-SEC-001）	[ ]	`docs/debt/DEBT-SEC-001.md` 存在，说明内存存储限制	

---

【工单矩阵·串行4阶段】

前置工单 FIX-P2/01 P2瑕疵修复 → validateConfig调用统一  
目标：消除src/api/server.js中内联重复校验逻辑，统一调用_validateConfig()

输入：  
- 代码基线：`46e2877` src/api/server.js  
- 当前问题：validateConfig()已实现（215-239行）但start()中内联重复逻辑（94-131行）

输出：  
- `src/api/server.js`（修改，start()改为调用_validateConfig()）  
- 删除内联重复代码，保持校验逻辑单一

自测点：  
- P2-FIX-001: `grep -A5 "start(" src/api/server.js` 显示调用_validateConfig()  
- P2-FIX-002: `grep -c "port.*65535" src/api/server.js` 返回1（仅_validateConfig内）  
- P2-FIX-003: `npm test` 全绿（无回归）  
- P2-FIX-004: `node -e "new HajimiServer({port:0}).start()"` 仍抛出错误（功能正常）

收卷标准：4项自测全绿 → 触发B-01/04启动

---

工单 B-01/04 Token Bucket限流器核心 → 算法实现（内存存储）  
目标：实现基于内存的Token Bucket限流算法（100 req/min/IP，burst 20）

输入：  
- 算法参考：经典Token Bucket（capacity=20，refillRate=100/60 per second）  
- 存储限制：内存Map（债务DEBT-SEC-001：重启清零，后续换Redis）

输出：  
- `src/security/rate-limiter.js`（新增，纯算法实现）  
- `src/security/rate-limiter.test.js`（单元测试）

强制代码结构：  

```javascript
class TokenBucketRateLimiter {
  constructor(options = {}) {
    this.capacity = options.capacity || 20;      // 桶容量
    this.refillRate = options.refillRate || 100/60; // 每秒补充token数
    this.buckets = new Map();                     // IP -> {tokens, lastRefill}
  }
  
  consume(ip, tokens = 1) {
    // 返回 { allowed: boolean, remaining: number, resetTime: Date }
  }
  
  _refill(bucket) { /* 补充逻辑 */ }
}
```  

自测点：  
- RATE-001: 单IP 100次请求内全部通过（allowed=true）  
- RATE-002: 单IP 101次请求返回429（allowed=false）  
- RATE-003: 等待60秒后token补充，请求再次通过  
- RATE-004: 不同IP独立计数（IP1超限时IP2正常）  
- RATE-005: 突发20请求立即通过（burst capacity）  

---

工单 B-02/04 限流中间件集成 → HTTP层限流保护  
目标：将Token Bucket集成到Express中间件链，应用到所有路由

输入：  
- B-01/04产出：`src/security/rate-limiter.js`  
- 集成点：`src/api/server.js` 中间件注册（在request-id之后，路由之前）

输出：  
- `src/api/middleware/rate-limit.js`（新增，包装RateLimiter为中间件）  
- `src/api/server.js`（修改，app.use注册限流中间件）

强制代码结构：  

```javascript
// rate-limit.js
const rateLimiter = new (require('../security/rate-limiter'))();

module.exports = function rateLimitMiddleware(req, res, next) {
  const ip = req.ip || req.connection.remoteAddress;
  const result = rateLimiter.consume(ip);
  
  res.setHeader('X-RateLimit-Limit', 100);
  res.setHeader('X-RateLimit-Remaining', result.remaining);
  res.setHeader('X-RateLimit-Reset', result.resetTime);
  
  if (!result.allowed) {
    res.status(429).json({ error: 'Too Many Requests', retryAfter: 60 });
    return;
  }
  next();
};
```  

自测点：  
- MID-001: `curl -I localhost:3000/health` 显示X-RateLimit-头  
- MID-002: 快速请求101次，`curl` 返回HTTP 429  
- MID-003: 429响应体包含retryAfter字段  
- MID-004: 限流触发时记录warn日志（含requestId）  

---

工单 B-03/04 API超时控制 → 请求超时保护  
目标：为所有API请求添加超时控制（默认30秒，可配置），防止慢查询阻塞

输入：  
- 超时算法：Promise.race + setTimeout  
- 配置集成：HajimiServer options.timeout

输出：  
- `src/api/middleware/timeout.js`（新增）  
- `src/api/server.js`（修改，集成timeout中间件）

强制代码结构：  

```javascript
// timeout.js
module.exports = function timeoutMiddleware(options = {}) {
  const timeoutMs = options.timeout || 30000; // 默认30秒
  
  return (req, res, next) => {
    const timer = setTimeout(() => {
      res.status(504).json({ error: 'Gateway Timeout', requestId: req.requestId });
    }, timeoutMs);
    
    res.on('finish', () => clearTimeout(timer));
    next();
  };
};
```  

自测点：  
- TIME-001: 正常请求（<30s）正常返回，无504  
- TIME-002: 慢查询（>30s）返回504，含requestId  
- TIME-003: 配置timeout: 5000时5秒触发超时  
- TIME-004: 超时后资源正确释放（无内存泄漏）  

---

工单 B-04/04 安全响应头与日志增强 → 安全基线提升  
目标：实现Helmet.js轻量级替代（安全响应头）+ 结构化JSON日志

输入：  
- 安全头清单：X-Frame-Options/DNS/X-Content-Type-Options/X-XSS-Protection  
- 日志结构：{timestamp, level, requestId, message, metadata}

输出：  
- `src/security/headers.js`（新增，安全头中间件）  
- `src/utils/logger.js`（新增，JSON格式日志）  
- `src/api/server.js`（修改，集成安全头中间件，替换console.log为logger）

强制代码结构： 

// headers.js - Helmet轻量级替代
module.exports = function securityHeaders(req, res, next) {
  res.setHeader('X-Frame-Options', 'DENY');
  res.setHeader('X-Content-Type-Options', 'nosniff');
  res.setHeader('X-XSS-Protection', '1; mode=block');
  res.setHeader('Referrer-Policy', 'strict-origin-when-cross-origin');
  next();
};

// logger.js
class JSONLogger {
  log(level, message, metadata = {}) {
    console.log(JSON.stringify({
      timestamp: new Date().toISOString(),
      level,
      requestId: metadata.requestId,
      message,
      ...metadata
    }));
  }
}

自测点：
 
SEC-001:  curl -I localhost:3000  显示全部4个安全头
 
SEC-002: 日志输出为合法JSON（可 JSON.parse ）
 
SEC-003: 日志含requestId字段（与请求头一致）
 
SEC-004: 错误日志含堆栈信息（error对象序列化安全）
 
SEC-005: 无敏感信息泄露（password字段被掩码）


【刀刃风险自测表·16项（Phase 2特化）】

用例ID	类别	场景	验证命令（可复制）	通过标准	状态（Engineer填）	
PH2-001	FUNC	P2瑕疵修复	`grep "_validateConfig" src/api/server.js`	命中且调用在start内	[ ]	
PH2-002	FUNC	限流100req/min	`for i in {1..101}; do curl -s -o /dev/null -w "%{http_code}" localhost:3000/health; done`	前100=200，最后1=429	[ ]	
PH2-003	FUNC	限流burst20	`for i in {1..20}; do curl -s -o /dev/null -w "%{http_code}," localhost:3000/health; done`	全部200	[ ]	
PH2-004	FUNC	限流头信息	`curl -I localhost:3000/health`	含X-RateLimit-	[ ]	
PH2-005	FUNC	超时30秒	`curl -m 35 localhost:3000/slow`（模拟慢查询）	返回504	[ ]	
PH2-006	FUNC	安全头完整	`curl -I localhost:3000`	含X-Frame-Options等4个头	[ ]	
PH2-007	FUNC	JSON日志	`node -e "require('./src/utils/logger').info('test')" 2>&1`	输出合法JSON	[ ]	
PH2-008	CONST	内存存储限制	检查DEBT-SEC-001.md	明确记录重启清零限制	[ ]	
PH2-009	CONST	向后兼容	使用Phase 1配置启动	正常启动，限流默认可用	[ ]	
PH2-010	NEG	不限流白名单	配置limiter: {enabled: false}	101次请求仍200	[ ]	
PH2-011	NEG	无日志泄露	`grep -r "password\|secret" src/utils/logger.js`	0命中	[ ]	
PH2-012	UX	429错误可读	`curl localhost:3000`（触发限流后）	返回JSON含retryAfter	[ ]	
PH2-013	E2E	全链路启动	`npm start`	正常启动，中间件无报错	[ ]	
PH2-014	HIGH	限流不误伤	多IP并发测试	IP1超限不影响IP2	[ ]	
PH2-015	HIGH	超时资源释放	触发超时后检查内存	无持续增长（RSS稳定）	[ ]	
PH2-016	SELF	26/16项自测全绿	全部手动勾选	无⬜	[ ]

【D级红线（地狱难度，触发即永久失败）】
1. 
P2瑕疵未修复（_validateConfig仍内联重复） → 永久禁用
2. 
限流算法错误（101次请求仍返回200） → 永久禁用
3. 
超时中间件导致内存泄漏（RSS持续增长） → 永久禁用
4. 
安全头缺失导致Security Headers评分< A → 永久禁用
5. 
日志泄露敏感信息（password字段未掩码） → 永久禁用
6. 
未声明DEBT-SEC-001（Rate Limiting内存限制） → 永久禁用
7. 
任何⬜标记为[x]但实际未验证 → 永久禁用
8. 
破坏性变更（旧配置无法启动） → 永久禁用


【验收标准（数值化，零容忍）】

验收项	验收命令	通过标准	失败标准（D级）	
P2修复	`grep -c "_validateConfig" src/api/server.js`	≥2（定义+调用）	0或1（未调用）	
限流100	`for i in 1..101; do curl...; done`	最后1次=429	全部200或提前429	
限流头	`curl -I localhost:3000`	含X-RateLimit-Limit	缺失	
超时30s	模拟31秒查询	返回504	超时无响应或崩溃	
安全头	`curl -I localhost:3000`	4个安全头全在	缺失>1个	
JSON日志	`logger.info('test')` 输出	可JSON.parse	非法JSON或undefined	
债务声明	`ls docs/debt/DEBT-SEC-001.md`	文件存在	缺失	
自测全绿	16+10项检查	全部[x]	任何⬜


【串行开发流程（强制顺序）

FIX-P2/01 (P2瑕疵修复)
    ↓ [4项自测全绿]
B-01/04 (Token Bucket核心)
    ↓ [5项自测全绿]
B-02/04 (限流中间件集成)
    ↓ [4项自测全绿]
B-03/04 (API超时控制)
    ↓ [4项自测全绿]
B-04/04 (安全头与日志)
    ↓ [5项自测全绿]
最终审计 → A/B/C/D评级

串行规则：前一工单未收卷（未全绿）禁止启动后一工单。

【收卷强制交付物（5件套）】
1. 
Git提交： git log --oneline -6  显示6条commits（1 P2修复 + 4 Phase 2 + 1 文档）
2. 
债务声明： docs/debt/DEBT-SEC-001.md （Rate Limiting内存限制说明）
3. 
自测表：《PHASE2-SECURITY-HARDENED-自测表-v1.0.md》（16项刀刃+10项P4，全手写[x]）
4. 
白皮书：《PHASE2-SECURITY-HARDENED-白皮书-v1.0.md》（5章：背景/实现/验证/债务/路线图）
5. 
安全扫描： grep -r "password\|secret\|api_key" src/  结果截图（0命中）