# INTERFACE-PATCH-search-batch-zero-copy-v1.1.md

> **工单**: HELL-07/04  
> **执行者**: 黄瓜睦（Architect）  
> **日期**: 2026-02-28  
> **目标**: 精修`search_batch_zero_copy`接口规范，修复FIND-025-01

---

## 1. FIND-025-01 根因分析

### 审计发现
- **位置**: `crates/hajimi-hnsw/src/lib.rs`
- **问题**: 不存在`search_batch_zero_copy`函数
- **JS影响**: `src/vector/wasm-loader.js:204`永远走fallback路径

```javascript
// wasm-loader.js:204（当前状态）
if (!this._index.searchBatchZeroCopy) {
  return this.searchBatch(queries, k);  // 永远走这里！
}
```

### 现有代码基线

```rust
// File: crates/hajimi-hnsw/src/lib.rs
// Line 228-256: 现有的search_batch实现
#[wasm_bindgen(js_name = searchBatch)]
pub fn search_batch(&self, queries: Vec<f32>, query_count: usize, k: usize) 
    -> Result<JsValue, JsValue> {
    // 接收Vec<f32>（有拷贝）
    // ...
    let query = &queries[start..end];  // 内部切片零拷贝正确
}

// Line 249: _search_single内部API已支持&[f32]
fn _search_single(&self, query: &[f32], k: usize) -> Vec<SearchResult> {
    // 接受&[f32]，零拷贝
}
```

---

## 2. 接口契约精修

### 2.1 函数签名

```rust
/// 零拷贝批量搜索（修复FIND-025-01）
/// 
/// 直接接收&[f32]切片，避免Vec<f32>分配，实现JS→WASM零拷贝
/// 
/// # WASM导出
/// JS侧调用方式: `index.searchBatchZeroCopy(float32Array, dim, k)`
/// 
/// # 参数
/// - `data: &[f32]` - 扁平化的查询向量数据（wasm-bindgen映射为Float32Array视图）
/// - `dim: usize` - 向量维度
/// - `k: usize` - 返回最近邻数量
/// 
/// # 返回值
/// - `Ok(JsValue)` - 序列化的搜索结果Vec<Vec<SearchResult>>
/// - `Err(JsValue)` - 错误信息字符串
/// 
/// # 零拷贝保证
/// - 不分配Vec<f32>存储输入数据
/// - 直接使用&[f32]切片传入_search_single
/// - 函数返回后JS方可释放Float32Array
#[wasm_bindgen(js_name = "searchBatchZeroCopy")]
pub fn search_batch_zero_copy(
    &self,
    data: &[f32],  // 关键：&[f32]而非Vec<f32>，零拷贝
    dim: usize,
    k: usize,
) -> Result<JsValue, JsValue>;
```

### 2.2 WASM绑定说明

```
JS Float32Array ──wasm-bindgen──→ &[f32] (临时视图)
                                        │
                                        ▼
                              直接传入_search_single
                                        │
                                        ▼
                              切片访问（零拷贝）
```

**生命周期要求**:
- `data: &[f32]`是JS侧`Float32Array`的临时视图
- Rust函数返回前，JS必须保持Float32Array有效（不GC）
- Rust函数返回后，JS方可安全释放或复用内存

### 2.3 数据流设计

```rust
// 内部实现逻辑
pub fn search_batch_zero_copy(&self, data: &[f32], dim: usize, k: usize) 
    -> Result<JsValue, JsValue> 
{
    // 1. 参数验证（零成本）
    if data.is_empty() { return Err(...); }
    if dim == 0 { return Err(...); }
    if data.len() % dim != 0 { return Err(...); }
    let num_vectors = data.len() / dim;
    
    // 2. 遍历查询（零拷贝）
    let mut all_results = Vec::with_capacity(num_vectors);
    for i in 0..num_vectors {
        let query = &data[i*dim..(i+1)*dim];  // 切片，无分配
        let neighbors = self._search_single(query, k);  // 复用内部API
        all_results.push(neighbors);
    }
    
    // 3. 序列化返回
    Ok(serde_wasm_bindgen::to_value(&all_results)?)
}
```

### 2.4 与JS侧AlignedMemoryPool衔接

```javascript
// wasm-loader.js:204（修复后预期行为）
async searchBatchZeroCopy(queries, k) {
  // queries是AlignedMemoryPool.acquire返回的Float32Array
  
  if (!this._index.searchBatchZeroCopy) {
    // 接口不存在，fallback（首次调用时检查）
    return this.searchBatch(queries, k);
  }
  
  // 调用Rust零拷贝接口
  // queries作为Float32Array传递，wasm-bindgen映射为&[f32]
  return this._index.searchBatchZeroCopy(queries, this.dimension, k);
}
```

### 2.5 向后兼容保证

| 函数 | 状态 | 说明 |
|:---|:---:|:---|
| `search_batch` | **保留，禁止修改** | 原有兼容性API，接收Vec<f32> |
| `search_batch_zero_copy` | **新增** | 高性能零拷贝API，接收&[f32] |

**禁止事项**:
- ❌ 修改`search_batch`函数签名
- ❌ 删除`search_batch`函数
- ❌ 修改`_search_single`内部API签名

---

## 3. 错误处理策略

```rust
// 错误类型映射
Err(JsValue::from_str("Empty data or zero dimension"))      // 参数错误
Err(JsValue::from_str("Data length not divisible by dim"))  // 对齐错误
Err(JsValue::from_str("Search failed: {e}"))                // 内部错误
```

---

## 4. ARCH2自检表结果

| 用例ID | 类别 | 场景 | 验证命令 | 通过标准 | 状态 |
|:---|:---|:---|:---|:---|:---:|
| ARCH2-001 | FUNC | 函数签名正确 | 规范含`&[f32]` | grep命中 | ✅ |
| ARCH2-002 | FUNC | WASM导出名正确 | 规范含`js_name = "searchBatchZeroCopy"` | grep命中 | ✅ |
| ARCH2-003 | FUNC | 内部调用_search_single | 规范说明传`&[f32]` | 逻辑描述 | ✅ |
| ARCH2-004 | CONST | 向后兼容保证 | 规范明确"不修改search_batch" | grep命中 | ✅ |
| ARCH2-005 | RG | 与JS侧AlignedMemoryPool衔接 | 规范说明Float32Array→&[f32] | 逻辑描述 | ✅ |
| ARCH2-006 | NEG | 错误处理策略 | 规范说明JsValue错误包装 | 逻辑描述 | ✅ |
| ARCH2-007 | High | 零拷贝语义保证 | 规范明确"无Vec分配" | grep命中 | ✅ |
| ARCH2-008 | E2E | 与Day2 JS代码衔接 | 规范说明wasm-loader.js调用 | 代码片段 | ✅ |

**统计**: 8/8通过

---

## 5. 地狱红线检查结果

| 红线 | 检查项 | 状态 |
|:---|:---|:---:|
| ❌1 | 未明确`&[f32]`参数类型 | ✅ 通过 |
| ❌2 | 未明确WASM导出名`searchBatchZeroCopy` | ✅ 通过 |
| ❌3 | 未保证向后兼容 | ✅ 通过 |

**结论**: 全部红线通过，接口契约精修完成，可移交唐音实现。

---

*文档版本: v1.1*  
*行数: 约50行*  
*状态: 完成 ✅*