# RISK-02-FIX-白皮书-v1.0.md

> **风险ID**: RISK-02  
> **等级**: C  
> **靶点**: `src/vector/wasm-loader.js:199-211`  
> **问题**: `getWASMLoader` 单例无并发保护  
> **执行者**: 黄瓜睦  
> **日期**: 2026-02-27

---

## 第一章：问题分析

### 1.1 原代码缺陷

```javascript
let loaderInstance = null;

async function getWASMLoader() {
  if (!loaderInstance) {
    loaderInstance = new WASMLoader();
    await loaderInstance.init();  // ← 异步间隙
  }
  return loaderInstance;
}
```

**问题**: 
- `loaderInstance` 在 `init()` 完成前仍为 `null`
- 多个并发请求同时调用时，会创建多个 `WASMLoader` 实例
- 每个实例独立持有WASM模块引用（2-5MB），造成内存重复占用

### 1.2 竞态条件

```
时间线:
T1: 请求A检查 loaderInstance === null (true)
T2: 请求B检查 loaderInstance === null (true)  ← 竞态！
T3: 请求A创建实例A，开始init()
T4: 请求B创建实例B，开始init()  ← 多实例！
```

### 1.3 潜在影响

- 内存重复占用（N个并发 = N倍占用）
- 多实例状态不一致
- 服务启动阶段（高并发初始化）最易触发

---

## 第二章：修复方案

### 2.1 修复策略

使用 **Promise缓存** 替代裸变量：
- 立即创建Promise，后续并发调用等待同一个Promise
- 确保 `init()` 只执行一次
- 天然支持异步并发控制

### 2.2 修复后代码

```javascript
let loaderPromise = null;

async function getWASMLoader() {
  if (!loaderPromise) {
    // 立即创建Promise，后续并发调用会等待同一个Promise
    loaderPromise = (async () => {
      const loader = new WASMLoader();
      await loader.init();
      return loader;
    })();
  }
  return loaderPromise;
}
```

### 2.3 修复原理

```
时间线:
T1: 请求A检查 loaderPromise === null (true)
T2: 请求A创建 Promise，开始执行
T3: 请求B检查 loaderPromise === null (false)  ← 拿到同一个Promise！
T4: 请求B await loaderPromise  ← 等待请求A完成
T5: 请求A完成 init()，返回 loader
T6: 请求B从 await 返回，得到同一个 loader
```

---

## 第三章：实现细节

### 3.1 关键变更

| 项目 | 修改前 | 修改后 |
|:---|:---|:---|
| 变量名 | `loaderInstance` | `loaderPromise` |
| 类型 | `WASMLoader|null` | `Promise<WASMLoader>|null` |
| 创建时机 | `init()` 后 | 立即（Promise构造函数） |
| 并发控制 | 无 | Promise原生排队 |

### 3.2 配套更新

`resetWASMLoader()` 函数同步更新：

```javascript
function resetWASMLoader() {
  loaderPromise = null;  // 原为 loaderInstance = null
}
```

---

## 第四章：验证结果

### 4.1 测试验证

```bash
node tests/wasm-loader-concurrent.test.js
```

**结果**:
- ✅ 10并发调用返回同实例
- ✅ init()只执行1次
- ✅ 内存增长<1.5x
- ✅ 无竞态创建多实例

### 4.2 性能对比

| 指标 | 修复前 | 修复后 |
|:---|:---|:---|
| 10并发实例数 | N个 | 1个 |
| 内存占用 | N × 2-5MB | 1 × 2-5MB |
| init()执行次数 | N次 | 1次 |

### 4.3 测试详情

```
✅ CONC-001: 10 concurrent calls return same instance
✅ CONC-002: init() executes only once
✅ CONC-003: No race condition creates multiple instances
✅ CONC-004: Memory does not double with concurrent init (0.33% increase)
✅ CONC-005: Sequential calls return same instance
✅ CONC-006: resetWASMLoader creates new instance on next call
```

---

## 附录：逐行对比

### 修改前（第199-207行）

```javascript
// 单例导出
let loaderInstance = null;

async function getWASMLoader() {
  if (!loaderInstance) {
    loaderInstance = new WASMLoader();
    await loaderInstance.init();
  }
  return loaderInstance;
}
```

### 修改后（第199-210行）

```javascript
// 单例导出 - 使用Promise缓存防止并发创建多实例
let loaderPromise = null;

async function getWASMLoader() {
  if (!loaderPromise) {
    // 立即创建Promise，后续并发调用会等待同一个Promise
    loaderPromise = (async () => {
      const loader = new WASMLoader();
      await loader.init();
      return loader;
    })();
  }
  return loaderPromise;
}
```

**变更统计**:
- 变量名: `loaderInstance` → `loaderPromise`
- 代码行数: 8行 → 10行
- 核心逻辑: 裸变量 → Promise缓存

### resetWASMLoader 修改

```javascript
// 修改前
function resetWASMLoader() {
  loaderInstance = null;
}

// 修改后
function resetWASMLoader() {
  loaderPromise = null;
}
```

---

*修复状态: 完成*  
*测试状态: 6/6通过*  
*风险等级: C → 已修复*
