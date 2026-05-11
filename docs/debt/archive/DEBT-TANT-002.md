# DEBT-TANT-002: TantivyIndexManager 行数偏差

## 申报信息
- **债务编号**: DEBT-TANT-002
- **申报日期**: 2026-04-12
- **申报人**: Agent B
- **关联审计**: 33-audit-final Week 8 专项清偿

## 偏差详情
- **目标文件**: `src/engine/search/tantivy_index.rs`
- **原始行数**: ~294
- **压缩后行数**: 250
- **压缩量**: 17 行（从 267 压缩到 250）

## 清偿记录
- **状态**: 已压缩 / resolved
- **清偿日期**: 2026-04-14
- **清偿方式**: 
  1. 移除 `extract_keywords` 方法块（~16 行）
  2. 移除 `TantivyIndexManager` 的 `Default` impl（~5 行）
  3. 移除 `get_jieba_tokenizer` 和 `has_chinese_content` 辅助方法（~9 行）
  4. 压缩 `JiebaTokenStream` 的 `TokenStream` impl（~8 行）
  5. 合并测试模块中相似测试并删除冗余注释（~6 行）
- **风险等级**: Low

## 签核
- [x] Tech Lead 确认
- [x] 已纳入 Week 9 排期
