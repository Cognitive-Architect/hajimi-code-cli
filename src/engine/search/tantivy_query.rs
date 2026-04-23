//! Tantivy Query Interface - Chinese Tokenization + Code Symbol Search
use tantivy::query::{QueryParser, TermQuery};
use tantivy::collector::TopDocs;
use tantivy::schema::*;
use tantivy::Term;
use crate::tantivy_index::{TantivyIndexManager, SearchDoc, ShardIndex};
use std::sync::{Arc, Mutex};
use std::time::Instant;
use lru::LruCache;

/// Query builder for full-text search with parallel shard querying and LRU caching
type QueryCache = Arc<Mutex<LruCache<(String, usize), Vec<SearchResult>>>>;

pub struct TantivyQueryBuilder {
    manager: TantivyIndexManager,
    cache: QueryCache,
}

/// Search results with highlighting
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub doc: SearchDoc,
    pub score: f32,
    pub highlights: Vec<String>,
}

impl TantivyQueryBuilder {
    /// Create new query builder
    pub fn new(manager: TantivyIndexManager) -> Self {
        let non_zero = std::num::NonZeroUsize::new(128).unwrap();
        let cache = Arc::new(Mutex::new(LruCache::new(non_zero)));
        Self { manager, cache }
    }

    fn get_cached(&self, key: &(String, usize)) -> Option<Vec<SearchResult>> {
        match self.cache.lock() {
            Ok(mut guard) => guard.get(key).cloned(),
            Err(_) => None,
        }
    }

    fn put_cache(&self, key: (String, usize), value: Vec<SearchResult>) {
        if let Ok(mut guard) = self.cache.lock() {
            guard.put(key, value);
        }
    }

    /// Search across all shards in parallel (BatchQuery optimization)
    pub async fn search(&self, query: &str, limit: usize) -> anyhow::Result<Vec<SearchResult>> {
        let start = Instant::now();
        let cache_key = (query.to_string(), limit);
        if let Some(cached) = self.get_cached(&cache_key) {
            return Ok(cached);
        }

        let mut handles = Vec::new();
        for shard_id in 0..16 {
            if let Some(shard) = self.manager.get_shard(shard_id) {
                let q = query.to_string();
                handles.push(tokio::spawn(async move {
                    Self::search_one_shard(shard, &q, limit).await
                }));
            }
        }

        let mut results = Vec::new();
        for h in handles {
            results.extend(h.await??);
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(limit);

        let _elapsed = start.elapsed();
        self.put_cache(cache_key, results.clone());
        Ok(results)
    }

    /// Symbol search (exact match) with parallel_search
    pub async fn search_symbol(&self, symbol: &str) -> anyhow::Result<Vec<SearchResult>> {
        let start = Instant::now();
        let limit = 10;
        let cache_key = (format!("sym:{}", symbol), limit);
        if let Some(cached) = self.get_cached(&cache_key) {
            return Ok(cached);
        }

        let mut handles = Vec::new();
        for shard_id in 0..16 {
            if let Some(shard) = self.manager.get_shard(shard_id) {
                let sym = symbol.to_lowercase();
                handles.push(tokio::spawn(async move {
                    Self::symbol_one_shard(shard, &sym, limit).await
                }));
            }
        }

        let mut results = Vec::new();
        for h in handles {
            results.extend(h.await??);
        }
        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));

        let _elapsed = start.elapsed();
        self.put_cache(cache_key, results.clone());
        Ok(results)
    }

    async fn search_one_shard(
        shard: Arc<tokio::sync::RwLock<ShardIndex>>,
        query: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let s = shard.read().await;
        let searcher = s.reader.searcher();
        let schema = s.index.schema();
        let fields: Vec<Field> = vec![
            schema.get_field("title")?,
            schema.get_field("body")?,
            schema.get_field("code")?,
        ];
        let query_parser = QueryParser::for_index(&s.index, fields);
        let parsed_query = query_parser.parse_query(query)?;
        Self::collect_results(&searcher, &schema, &parsed_query, limit).await
    }

    async fn symbol_one_shard(
        shard: Arc<tokio::sync::RwLock<ShardIndex>>,
        symbol: &str,
        limit: usize,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let s = shard.read().await;
        let searcher = s.reader.searcher();
        let schema = s.index.schema();
        let symbol_field = schema.get_field("symbol")?;
        let term = Term::from_field_text(symbol_field, symbol);
        let query = TermQuery::new(term, IndexRecordOption::Basic);
        Self::collect_results(&searcher, &schema, &query, limit).await
    }

    async fn collect_results(
        searcher: &tantivy::Searcher,
        schema: &Schema,
        query: &dyn tantivy::query::Query,
        limit: usize,
    ) -> anyhow::Result<Vec<SearchResult>> {
        let mut results = Vec::new();
        for (score, doc_address) in searcher.search(query, &TopDocs::with_limit(limit))? {
            let doc = searcher.doc(doc_address)?;
            results.push(SearchResult {
                doc: Self::doc_to_search_doc(&doc, schema)?,
                score,
                highlights: vec![],
            });
        }
        Ok(results)
    }

    fn doc_to_search_doc(doc: &tantivy::Document, schema: &Schema) -> anyhow::Result<SearchDoc> {
        let get_text = |doc: &tantivy::Document, field: Field| -> String {
            match doc.get_first(field) {
                Some(v) => match v.as_text() {
                    Some(s) => s.to_string(),
                    None => String::new(),
                },
                None => String::new(),
            }
        };
        let id = get_text(doc, schema.get_field("id")?);
        let title = get_text(doc, schema.get_field("title")?);
        let body = get_text(doc, schema.get_field("body")?);
        let code = match doc.get_first(schema.get_field("code")?) {
            Some(v) => v.as_text().map(|s| s.to_string()),
            None => None,
        };
        let symbol = match doc.get_first(schema.get_field("symbol")?) {
            Some(v) => v.as_text().map(|s| s.to_string()),
            None => None,
        };
        Ok(SearchDoc {
            id,
            title,
            body,
            code: code.filter(|s| !s.is_empty()),
            symbol: symbol.filter(|s| !s.is_empty()),
        })
    }
}

/// Batch indexer for high-throughput indexing
pub struct BatchIndexer {
    manager: TantivyIndexManager,
    buffer: Vec<SearchDoc>,
    buffer_size: usize,
}

impl BatchIndexer {
    pub fn new(manager: TantivyIndexManager, buffer_size: usize) -> Self {
        Self { manager, buffer: Vec::with_capacity(buffer_size), buffer_size }
    }

    pub async fn add(&mut self, doc: SearchDoc) -> anyhow::Result<()> {
        self.buffer.push(doc);
        if self.buffer.len() >= self.buffer_size {
            self.flush().await?;
        }
        Ok(())
    }

    pub async fn flush(&mut self) -> anyhow::Result<()> {
        for doc in self.buffer.drain(..) {
            self.manager.add_document(doc).await?;
        }
        self.manager.commit_all().await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tantivy_index::JiebaTokenizer;
    use std::time::Duration;
    use tokio::time::timeout;

    fn setup_test_index() -> anyhow::Result<(tempfile::TempDir, TantivyIndexManager)> {
        let dir = tempfile::tempdir()?;
        let manager = TantivyIndexManager::new_with_path(
            dir.path().to_str().ok_or_else(|| anyhow::anyhow!("path"))?
        )?;
        Ok((dir, manager))
    }

    #[tokio::test]
    async fn test_query_latency() -> anyhow::Result<()> {
        let (_dir, manager) = setup_test_index()?;
        let builder = TantivyQueryBuilder::new(manager);
        let doc = SearchDoc {
            id: "doc_latency".to_string(),
            title: "latency test".to_string(),
            body: "body content".to_string(),
            code: None,
            symbol: None,
        };
        builder.manager.add_document(doc).await?;

        let start = Instant::now();
        let results = builder.search("latency", 10).await?;
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 200, "query too slow: {}ms", elapsed.as_millis());
        assert!(!results.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_query_empty_index() -> anyhow::Result<()> {
        let (_dir, manager) = setup_test_index()?;
        let builder = TantivyQueryBuilder::new(manager);
        let results = builder.search("nonexistent", 10).await?;
        assert!(results.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_query_timeout() -> anyhow::Result<()> {
        let (_dir, manager) = setup_test_index()?;
        let builder = TantivyQueryBuilder::new(manager);
        let doc = SearchDoc {
            id: "timeout_doc".to_string(),
            title: "timeout".to_string(),
            body: "content".to_string(),
            code: None,
            symbol: None,
        };
        builder.manager.add_document(doc).await?;
        builder.manager.commit_all().await?;

        let res = timeout(Duration::from_millis(500), builder.search("timeout", 10)).await;
        assert!(res.is_ok());
        assert!(!res.unwrap()?.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_chinese_query_performance() -> anyhow::Result<()> {
        let (_dir, manager) = setup_test_index()?;
        let builder = TantivyQueryBuilder::new(manager);
        let tokenizer = JiebaTokenizer::new();
        let doc = SearchDoc {
            id: "cn_doc_0".to_string(),
            title: "中华人民共和国测试".to_string(),
            body: "中文搜索性能测试".to_string(),
            code: None,
            symbol: None,
        };
        builder.manager.add_document(doc).await?;

        let tokens = tokenizer.tokenize("中华人民共和国");
        assert!(!tokens.is_empty());

        let start = Instant::now();
        let results = builder.search("中华人民共和国", 10).await?;
        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 200, "chinese query too slow: {}ms", elapsed.as_millis());
        assert!(!results.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_query_malformed_chinese() -> anyhow::Result<()> {
        let (_dir, manager) = setup_test_index()?;
        let builder = TantivyQueryBuilder::new(manager);
        let doc = SearchDoc {
            id: "malformed_doc".to_string(),
            title: "test".to_string(),
            body: "body".to_string(),
            code: None,
            symbol: None,
        };
        builder.manager.add_document(doc).await?;
        builder.manager.commit_all().await?;

        let query = "\u{0000}\u{0001} malformed";
        let _ = builder.search(query, 10).await;
        Ok(())
    }
}
