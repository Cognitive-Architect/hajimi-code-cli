use std::collections::{HashMap, VecDeque};
use std::time::Instant;

const MAX_SESSION_TOKENS: usize = 4_000;
const ESTIMATED_TOKENS_PER_CHAR: usize = 4;

#[derive(Debug, Clone)]
pub struct SessionEntry {
    pub content: String,
    pub tokens: usize,
    pub timestamp: Instant,
    pub access_count: u64,
}

#[derive(Debug)]
pub struct SessionMemory {
    entries: HashMap<String, SessionEntry>,
    lru: VecDeque<String>,
    token_counter: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SessionError {
    TokenLimitExceeded,
    KeyNotFound,
    EmptyContent,
}

impl std::fmt::Display for SessionError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SessionError::TokenLimitExceeded => write!(f, "Token limit exceeded"),
            SessionError::KeyNotFound => write!(f, "Key not found"),
            SessionError::EmptyContent => write!(f, "Empty content"),
        }
    }
}

impl std::error::Error for SessionError {}

fn estimate_tokens(content: &str) -> usize {
    content.len().div_ceil(ESTIMATED_TOKENS_PER_CHAR).max(1)
}

impl SessionMemory {
    pub fn new() -> Self {
        Self { entries: HashMap::new(), lru: VecDeque::new(), token_counter: 0 }
    }

    pub fn get(&self, key: &str) -> Option<&SessionEntry> {
        self.entries.get(key)
    }

    pub fn get_mut(&mut self, key: &str) -> Option<&mut SessionEntry> {
        if let Some(e) = self.entries.get_mut(key) {
            e.access_count += 1;
            Some(e)
        } else {
            None
        }
    }

    fn evict_lru(&mut self, required: usize) -> Result<(), SessionError> {
        while self.token_counter + required > MAX_SESSION_TOKENS {
            let key = self.lru.pop_front().ok_or(SessionError::TokenLimitExceeded)?;
            if let Some(e) = self.entries.remove(&key) { self.token_counter -= e.tokens; }
        }
        Ok(())
    }

    pub fn insert(&mut self, key: String, content: String) -> Result<(), SessionError> {
        if content.is_empty() { return Err(SessionError::EmptyContent); }
        let tokens = estimate_tokens(&content);
        if tokens > MAX_SESSION_TOKENS { return Err(SessionError::TokenLimitExceeded); }
        if self.entries.contains_key(&key) {
            if let Some(e) = self.entries.remove(&key) { self.token_counter -= e.tokens; }
            if let Some(p) = self.lru.iter().position(|k| k == &key) { self.lru.remove(p); }
        }
        self.evict_lru(tokens)?;
        let entry = SessionEntry { content, tokens, timestamp: Instant::now(), access_count: 0 };
        self.entries.insert(key.clone(), entry);
        self.lru.push_back(key);
        self.token_counter += tokens;
        Ok(())
    }

    pub fn delete(&mut self, key: &str) -> Result<(), SessionError> {
        match self.entries.remove(key) {
            Some(e) => {
                self.token_counter -= e.tokens;
                if let Some(p) = self.lru.iter().position(|k| k == key) { self.lru.remove(p); }
                Ok(())
            }
            None => Err(SessionError::KeyNotFound),
        }
    }

    pub fn clear(&mut self) {
        self.entries.clear();
        self.lru.clear();
        self.token_counter = 0;
    }

    pub fn total_tokens(&self) -> usize { self.token_counter }
    pub fn len(&self) -> usize { self.entries.len() }
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }
    pub fn keys(&self) -> impl Iterator<Item = &String> { self.lru.iter() }
    pub fn contains_key(&self, key: &str) -> bool { self.entries.contains_key(key) }
}

impl Default for SessionMemory { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new() {
        let s = SessionMemory::new();
        assert_eq!(s.total_tokens(), 0);
        assert!(s.is_empty());
    }

    #[test]
    fn test_insert_get() {
        let mut s = SessionMemory::new();
        assert!(s.insert("k1".into(), "Hello".into()).is_ok());
        assert_eq!(s.get("k1").map(|e| &e.content), Some(&"Hello".to_string()));
        assert!(s.contains_key("k1"));
    }

    #[test]
    fn test_update_existing_key() {
        let mut s = SessionMemory::new();
        s.insert("k1".into(), "Hello".into()).expect("insert");
        s.insert("k1".into(), "World".into()).expect("update");
        assert_eq!(s.get("k1").map(|e| &e.content), Some(&"World".to_string()));
        assert_eq!(s.len(), 1);
    }

    #[test]
    fn test_lru_eviction() {
        let mut s = SessionMemory::new();
        let c = "a".repeat(1000);
        for i in 0..20 { let _ = s.insert(format!("k{}", i), c.clone()); }
        assert!(s.total_tokens() <= MAX_SESSION_TOKENS);
        assert!(s.len() < 20);
    }

    #[test]
    fn test_delete() {
        let mut s = SessionMemory::new();
        assert!(s.insert("k1".into(), "Hello".into()).is_ok());
        let before = s.total_tokens();
        assert!(s.delete("k1").is_ok());
        assert!(s.get("k1").is_none());
        assert!(s.total_tokens() < before);
    }

    #[test]
    fn test_delete_not_found() {
        let mut s = SessionMemory::new();
        assert_eq!(s.delete("missing"), Err(SessionError::KeyNotFound));
    }

    #[test]
    fn test_clear() {
        let mut s = SessionMemory::new();
        assert!(s.insert("k1".into(), "Hello".into()).is_ok());
        assert!(s.insert("k2".into(), "World".into()).is_ok());
        s.clear();
        assert!(s.is_empty());
        assert_eq!(s.total_tokens(), 0);
        assert!(!s.contains_key("k1"));
    }

    #[test]
    fn test_empty_content_error() {
        let mut s = SessionMemory::new();
        assert_eq!(s.insert("k1".into(), "".into()), Err(SessionError::EmptyContent));
    }

    #[test]
    fn test_get_mut_increments_access() {
        let mut s = SessionMemory::new();
        s.insert("k1".into(), "Hello".into()).expect("insert");
        let _ = s.get_mut("k1");
        let _ = s.get_mut("k1");
        assert_eq!(s.get("k1").map(|e| e.access_count), Some(2));
    }

    #[test]
    fn test_keys_order() {
        let mut s = SessionMemory::new();
        s.insert("k1".into(), "a".into()).expect("insert");
        s.insert("k2".into(), "b".into()).expect("insert");
        let keys: Vec<_> = s.keys().collect();
        assert_eq!(keys, vec!["k1", "k2"]);
    }
}
