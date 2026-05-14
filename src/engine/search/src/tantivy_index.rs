//! Tantivy Full-Text Index Integration with SimHash-64 Sharding
//! Reuses existing 16-shard SQLite architecture
//! Week 8: Added JiebaTokenizer Chinese tokenizer support
use foundation_hash::{get_shard_id, NUM_SHARDS};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tantivy::schema::*;
use tantivy::tokenizer::{TextAnalyzer, Tokenizer, TokenizerManager};
use tantivy::{Index, IndexReader, IndexWriter, TantivyError};
use tokio::sync::RwLock;

pub struct TantivyIndexManager {
    shards: Vec<Arc<RwLock<ShardIndex>>>,
    schema: Schema,
    #[allow(dead_code)]
    tokenizer_manager: TokenizerManager,
}

pub struct ShardIndex {
    pub index: Index,
    pub writer: IndexWriter,
    pub reader: IndexReader,
    pub shard_id: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SearchDoc {
    pub id: String,
    pub title: String,
    pub body: String,
    pub code: Option<String>,
    pub symbol: Option<String>,
}

pub struct JiebaTokenizer {
    jieba: Arc<jieba_rs::Jieba>,
}

impl Clone for JiebaTokenizer {
    fn clone(&self) -> Self {
        Self {
            jieba: Arc::clone(&self.jieba),
        }
    }
}

impl JiebaTokenizer {
    pub fn new() -> Self {
        Self {
            jieba: Arc::new(jieba_rs::Jieba::new()),
        }
    }
    pub fn tokenize(&self, text: &str) -> Vec<String> {
        self.jieba
            .cut(text, true)
            .into_iter()
            .map(|s| s.to_string())
            .collect()
    }
    pub fn has_chinese(&self, text: &str) -> bool {
        text.chars().any(|c| ('\u{4e00}'..='\u{9fff}').contains(&c))
    }
}

impl Default for JiebaTokenizer {
    fn default() -> Self {
        Self::new()
    }
}

impl Tokenizer for JiebaTokenizer {
    type TokenStream<'a> = JiebaTokenStream<'a>;
    fn token_stream<'a>(&'a mut self, text: &'a str) -> Self::TokenStream<'a> {
        let tokens = self.tokenize(text);
        JiebaTokenStream {
            tokens,
            current: 0,
            offset_from: 0,
            token: tantivy::tokenizer::Token {
                offset_from: 0,
                offset_to: 0,
                position: 0,
                text: String::new(),
                position_length: 1,
            },
            _marker: std::marker::PhantomData,
        }
    }
}

pub struct JiebaTokenStream<'a> {
    tokens: Vec<String>,
    current: usize,
    offset_from: usize,
    token: tantivy::tokenizer::Token,
    _marker: std::marker::PhantomData<&'a ()>,
}

impl<'a> tantivy::tokenizer::TokenStream for JiebaTokenStream<'a> {
    fn advance(&mut self) -> bool {
        if self.current < self.tokens.len() {
            let text = &self.tokens[self.current];
            self.token.text = text.clone();
            self.token.offset_from = self.offset_from;
            self.token.offset_to = self.offset_from + text.len();
            self.token.position = self.current;
            self.offset_from += text.len();
            self.current += 1;
            true
        } else {
            false
        }
    }
    fn token(&self) -> &tantivy::tokenizer::Token {
        &self.token
    }
    fn token_mut(&mut self) -> &mut tantivy::tokenizer::Token {
        &mut self.token
    }
}

impl TantivyIndexManager {
    pub fn new() -> Result<Self, TantivyError> {
        Self::new_with_path("./data/search")
    }

    pub fn new_with_path(base_path: &str) -> Result<Self, TantivyError> {
        let mut schema_builder = Schema::builder();
        let text_options = TextOptions::default()
            .set_indexing_options(TextFieldIndexing::default().set_tokenizer("jieba"))
            .set_stored();
        schema_builder.add_text_field("id", STRING | STORED);
        schema_builder.add_text_field("title", text_options.clone());
        schema_builder.add_text_field("body", text_options.clone());
        schema_builder.add_text_field("code", text_options);
        schema_builder.add_text_field("symbol", STRING | STORED);
        schema_builder.add_u64_field("shard_id", INDEXED | STORED);
        let schema = schema_builder.build();

        let tokenizer_manager = TokenizerManager::new();
        let jieba = JiebaTokenizer::new();
        let jieba_analyzer = TextAnalyzer::builder(jieba).build();
        tokenizer_manager.register("jieba", jieba_analyzer.clone());

        let mut shards = Vec::with_capacity(NUM_SHARDS);
        for shard_id in 0..NUM_SHARDS {
            let index_path = format!("{}/shard_{}", base_path, shard_id);
            std::fs::create_dir_all(&index_path).ok();
            let index = Index::create_in_dir(&index_path, schema.clone())?;
            index.tokenizers().register("jieba", jieba_analyzer.clone());
            let writer = index.writer(50_000_000)?;
            let reader = index.reader()?;
            shards.push(Arc::new(RwLock::new(ShardIndex {
                index,
                writer,
                reader,
                shard_id,
            })));
        }
        Ok(Self {
            shards,
            schema,
            tokenizer_manager,
        })
    }

    pub async fn add_document(&self, doc: SearchDoc) -> Result<(), TantivyError> {
        let shard_id = get_shard_id(&doc.id);
        let mut shard = self.shards[shard_id].write().await;
        let mut tantivy_doc = Document::default();
        tantivy_doc.add_text(self.schema.get_field("id")?, &doc.id);
        tantivy_doc.add_text(self.schema.get_field("title")?, &doc.title);
        tantivy_doc.add_text(self.schema.get_field("body")?, &doc.body);
        if let Some(code) = doc.code {
            tantivy_doc.add_text(self.schema.get_field("code")?, &code);
        }
        if let Some(symbol) = doc.symbol {
            tantivy_doc.add_text(self.schema.get_field("symbol")?, &symbol);
        }
        tantivy_doc.add_u64(self.schema.get_field("shard_id")?, shard_id as u64);
        shard.writer.add_document(tantivy_doc)?;
        shard.writer.commit()?;
        shard.reader.reload()?;
        Ok(())
    }

    pub async fn commit_all(&self) -> Result<(), TantivyError> {
        for shard in &self.shards {
            let mut s = shard.write().await;
            s.writer.commit()?;
            s.reader.reload()?;
        }
        Ok(())
    }

    pub fn get_shard(&self, shard_id: usize) -> Option<Arc<RwLock<ShardIndex>>> {
        self.shards.get(shard_id).cloned()
    }

    pub fn shard_count(&self) -> usize {
        self.shards.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simhash_routing() {
        assert!(get_shard_id("test_doc_1") < NUM_SHARDS);
        assert_eq!(
            get_shard_id("consistent_doc"),
            get_shard_id("consistent_doc")
        );
    }

    #[test]
    fn test_simhash64_values() {
        let h1 = simhash64("a");
        let h2 = simhash64("b");
        let h3 = simhash64("a");
        assert_ne!(h1, h2);
        assert_eq!(h1, h3);
    }

    #[test]
    fn test_simhash64_shard_distribution() {
        let mut counts = vec![0; NUM_SHARDS];
        for i in 0..1000 {
            counts[get_shard_id(&format!("doc_{}", i))] += 1;
        }
        for (i, c) in counts.iter().enumerate() {
            assert!(*c > 0, "Shard {} empty", i);
        }
    }

    #[test]
    fn test_jieba_tokenizer_creation() {
        let t1 = JiebaTokenizer::new();
        let t2 = JiebaTokenizer::default();
        assert!(t1.has_chinese("中文"));
        assert!(!t2.has_chinese("abc"));
    }

    #[test]
    fn test_jieba_tokenizer() {
        let tokenizer = JiebaTokenizer::new();
        assert!(tokenizer.has_chinese("中华人民共和国"));
        assert!(!tokenizer.has_chinese("Hello World"));
        let tokens = tokenizer.tokenize("中华人民共和国");
        assert!(!tokens.is_empty());
        assert!(tokens
            .iter()
            .any(|t| t.contains("中华") || t.contains("人民") || t.contains("共和")));
        assert!(tokens.len() < "中华人民共和国".chars().count());
    }

    #[tokio::test]
    async fn test_manager_creation_and_document() {
        let dir = tempfile::tempdir().expect("tempdir");
        let manager =
            TantivyIndexManager::new_with_path(dir.path().to_str().unwrap()).expect("create");
        assert_eq!(manager.shard_count(), NUM_SHARDS);
        let doc = SearchDoc {
            id: "doc_1".to_string(),
            title: "Test".to_string(),
            body: "Body".to_string(),
            code: None,
            symbol: None,
        };
        manager.add_document(doc).await.expect("add");
        manager.commit_all().await.expect("commit");
    }

    #[test]
    fn test_shard_access() {
        let dir = tempfile::tempdir().expect("tempdir");
        let manager =
            TantivyIndexManager::new_with_path(dir.path().to_str().unwrap()).expect("create");
        assert!(manager.get_shard(0).is_some());
        assert!(manager.get_shard(NUM_SHARDS).is_none());
        assert_eq!(manager.shard_count(), NUM_SHARDS);
    }
}
