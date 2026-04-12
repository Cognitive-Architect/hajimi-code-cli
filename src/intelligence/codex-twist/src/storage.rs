//! 存储持久化层 - .hctx文件格式
use serde::{Serialize, Deserialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ContextChunk {
    System { content: String, metadata: Option<Metadata> },
    User { content: String, timestamp: u64 },
    Assistant { content: String, tokens: Option<usize> },
}

impl ContextChunk {
    pub fn system(content: String) -> Self { Self::System { content, metadata: None } }
    pub fn user(content: String) -> Self { Self::User { content, timestamp: now() } }
    pub fn assistant(content: String) -> Self { Self::Assistant { content, tokens: None } }
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
pub struct Metadata { pub id: String, pub created_at: u64, pub version: String }

#[derive(Clone, Debug, PartialEq)]
pub enum StorageError {
    IoError(String),
    ParseError(String),
    NotFound(String),
}

impl std::fmt::Display for StorageError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self { StorageError::IoError(s) => write!(f, "IO: {}", s), StorageError::ParseError(s) => write!(f, "Parse: {}", s), StorageError::NotFound(s) => write!(f, "NotFound: {}", s) }
    }
}
impl std::error::Error for StorageError {}

pub struct HctxStorage { path: PathBuf }

impl HctxStorage {
    pub fn new(path: PathBuf) -> Result<Self, StorageError> {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| StorageError::IoError(e.to_string()))?;
        }
        Ok(Self { path })
    }

    pub fn save_context(&self, chunks: &[ContextChunk]) -> Result<(), StorageError> {
        let json = serde_json::to_string_pretty(chunks).map_err(|e| StorageError::IoError(format!("序列化: {}", e)))?;
        fs::write(&self.path, json).map_err(|e| StorageError::IoError(format!("写入 {}: {}", self.path.display(), e)))?;
        Ok(())
    }

    pub fn load_context(&self) -> Result<Vec<ContextChunk>, StorageError> {
        if !self.path.exists() { return Err(StorageError::NotFound(self.path.display().to_string())); }
        let json = fs::read_to_string(&self.path).map_err(|e| StorageError::IoError(format!("读取 {}: {}", self.path.display(), e)))?;
        let chunks: Vec<ContextChunk> = serde_json::from_str(&json).map_err(|e| StorageError::ParseError(format!("解析: {}", e)))?;
        Ok(chunks)
    }

    pub fn path(&self) -> &Path { &self.path }
    pub fn exists(&self) -> bool { self.path.exists() }
}

fn now() -> u64 { std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_secs() }
