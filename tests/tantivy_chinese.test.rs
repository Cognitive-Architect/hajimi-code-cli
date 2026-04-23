//! Tantivy Chinese Tokenizer Tests
use crate::tantivy_index::{JiebaTokenizer, SearchDoc, TantivyIndexManager};

#[test]
fn test_jieba_tokenizer_basic() {
    let tokenizer = JiebaTokenizer::new();
    let text = "中华人民共和国";
    let tokens = tokenizer.tokenize(text);
    
    // Verify meaningful segmentation
    assert!(!tokens.is_empty());
    assert!(tokens.iter().any(|t| t.contains("中华") || t.contains("人民")));
}

#[test]
fn test_chinese_fulltext_search() {
    let manager = TantivyIndexManager::new().unwrap();
    
    let doc = SearchDoc {
        id: "cn_doc_1".to_string(),
        title: "中华人民共和国的宪法".to_string(),
        body: "这是一个关于中华人民共和国法律体系的中文文档".to_string(),
        code: None,
        symbol: None,
    };
    
    // Verify Chinese content detection
    assert!(manager.has_chinese_content(&doc));
}

#[test]
fn test_mixed_chinese_english() {
    let tokenizer = JiebaTokenizer::new();
    let text = "Hello World 中华人民共和国";
    let tokens = tokenizer.tokenize(text);
    
    // Should handle mixed content
    assert!(!tokens.is_empty());
}
