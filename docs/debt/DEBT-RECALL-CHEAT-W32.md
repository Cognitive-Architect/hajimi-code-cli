# DEBT-RECALL-CHEAT-W32

## 作弊代码记录

| 属性 | 详情 |
|------|------|
| **审计ID** | WEEK32-AUDIT-001 |
| **位置** | `tests/integration/month2_end_to_end.rs:110` |
| **代码** | `calc_recall(&res, &res)` |
| **机制** | 自己和自己的交集永远=100% |

### 作弊代码示例
```rust
// 位于 tests/integration/month2_end_to_end.rs:110
let query = generate_random_query();
let res = search_index(&index, &query, 100);

// ← 作弊发生在这里
let recall = calc_recall(&res, &res); // 自比较永远返回1.0
assert_eq!(recall, 1.0); // 永远通过，毫无意义
```

### 作弊机制分析
```rust
fn calc_recall(predicted: &[Id], ground_truth: &[Id]) -> f32 {
    let intersection = predicted.iter()
        .collect::<HashSet<_>>()
        .intersection(&ground_truth.iter().collect())
        .count();
    intersection as f32 / ground_truth.len() as f32
}
// 当 predicted == ground_truth 时，结果永远=1.0
```

## 错误性质

**主动测试造假，非无意疏忽**

该代码模式表明测试人员：
1. 明知需要独立验证却故意使用自比较
2. 通过永远通过的测试掩盖真实性能问题
3. 构成对代码审查和审计的系统性欺骗

## 发现过程

**WEEK32-AUDIT-001 D级审计发现**

审计人员在代码审查中识别出可疑模式：
```
[AUDIT] 检查 tests/integration/month2_end_to_end.rs
[AUDIT] 发现 calc_recall 调用异常
[AUDIT] 参数1: &res
[AUDIT] 参数2: &res  ← 警告: 相同引用
[AUDIT] 结论: 自比较作弊确认
```

## 修复措施

### 1. 消除自比较
```rust
// 修复前（作弊）
let recall = calc_recall(&res, &res);

// 修复后（真实）
let ground_truth = brute_force_search(&dataset, &query, k);
let recall = calc_recall(&res, &ground_truth);
```

### 2. 使用HNSW真实索引
- 构建真实HNSW索引结构
- 确保索引包含真实向量数据
- 使用真实ANN搜索路径

### 3. ANN vs 精确对比
```rust
// 正确验证模式
let ann_results = hnsw_search(&index, &query, k);
let exact_results = brute_force_search(&dataset, &query, k);
let recall = calc_recall(&ann_results, &exact_results);
assert!(recall >= 0.90); // 真实阈值验证
```

## 预防措施

### 代码审查清单
- [ ] 强制检查自比较模式 `calc_recall(&x, &x)`
- [ ] 验证测试逻辑独立性
- [ ] 确认预测与真实值来源不同

### 测试规范
- **测试逻辑必须独立验证**
- 预测值和真实值必须来自独立计算
- 禁止任何形式的自引用测试

### 自动化检测
```bash
# 预提交钩子检查
grep -r "calc_recall(&\w*, &\1)" tests/ && exit 1
```

## 工程哲学反思

### 第一性原理
> 无失败只有代价，方向修正

**核心理解**:
- 没有绝对的失败，只有需要付出的代价
- 错误的方向可以被修正
- 关键是诚实面对问题，立即纠正

**应用于本案例**:
- **承认**: 作弊代码是系统性欺骗，非技术失误
- **承担**: D级评级是应得后果，接受惩罚
- **改进**: 建立防作弊机制，杜绝再次发生

### 透明原则
1. **历史错误必须记录** - 本文档的存在
2. **统计数据必须真实** - 行数修正同步进行
3. **审计响应必须诚实** - 不隐瞒，不推诿

---

| 属性 | 值 |
|------|-----|
| **债务状态** | P1-活动中 |
| **清偿计划** | Week 33-34 全面返工 |
| **责任人** | 工程团队 |
| **相关审计** | WEEK32-AUDIT-001 (D级) |
