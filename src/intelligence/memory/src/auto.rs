use crate::session::{SessionEntry, SessionMemory};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use tempfile::NamedTempFile;

/// Auto层错误类型
#[derive(Debug, thiserror::Error)]
pub enum AutoError {
    #[error("IO错误: {0}")]
    Io(#[from] io::Error),
    #[error("JSON错误: {0}")]
    Json(#[from] serde_json::Error),
    #[error("无法获取用户主目录")]
    NoHomeDir,
    #[error("无效的项目ID")]
    InvalidProjectId,
}

/// Auto层条目 - 包含384维向量embedding预留字段（ONNX集成预备）
#[derive(Debug, Clone)]
pub struct AutoEntry {
    pub session_entry: SessionEntry,
    pub file_path: PathBuf,
    pub last_persisted: DateTime<Utc>,
    /// 384维向量embedding占位，Week 28预留用于ONNX集成
    pub embedding: Option<Vec<f32>>,
}

/// JSONL格式
#[derive(Serialize, Deserialize)]
struct PersistedEntry {
    id: String,
    content: String,
    tokens: usize,
    timestamp: String,
}

/// Auto层内存 - JSONL持久化，原子写入，延迟写入
pub struct AutoMemory {
    pub storage_dir: PathBuf,
    pub entries: HashMap<String, AutoEntry>,
    pub dirty: bool,
}

impl AutoMemory {
    /// 创建新实例，路径: ~/.hajimi/memory/{project_id}/
    pub fn new(project_id: &str) -> Result<Self, AutoError> {
        if project_id.is_empty() || project_id.contains('/') || project_id.contains('\\') {
            return Err(AutoError::InvalidProjectId);
        }
        let home = dirs::home_dir().ok_or(AutoError::NoHomeDir)?;
        let storage_dir = home.join(".hajimi").join("memory").join(project_id);
        fs::create_dir_all(&storage_dir)?;
        Ok(Self { storage_dir, entries: HashMap::new(), dirty: false })
    }

    /// 获取存储目录
    pub fn storage_dir(&self) -> &PathBuf { &self.storage_dir }

    /// 检查是否脏
    pub fn is_dirty(&self) -> bool { self.dirty }

    /// 获取条目数
    pub fn len(&self) -> usize { self.entries.len() }

    /// 是否为空
    pub fn is_empty(&self) -> bool { self.entries.is_empty() }

    /// 原子持久化 - NamedTempFile + fs::rename
    pub fn persist(&mut self) -> Result<(), AutoError> {
        if !self.dirty { return Ok(()); }
        let persist_file = self.storage_dir.join("memory.jsonl");
        let mut temp = NamedTempFile::new_in(&self.storage_dir)?;
        for (k, v) in &self.entries {
            let p = PersistedEntry {
                id: k.clone(),
                content: v.session_entry.content.clone(),
                tokens: v.session_entry.tokens,
                timestamp: v.last_persisted.to_rfc3339(),
            };
            writeln!(temp, "{}", serde_json::to_string(&p)?)?;
        }
        temp.flush()?;
        fs::rename(temp.path(), &persist_file)?;
        let now = Utc::now();
        for e in self.entries.values_mut() { e.last_persisted = now; }
        self.dirty = false;
        Ok(())
    }

    /// 从JSONL加载
    pub fn load(&mut self) -> Result<(), AutoError> {
        let file = self.storage_dir.join("memory.jsonl");
        if !file.exists() { return Ok(()); }
        let content = fs::read_to_string(&file)?;
        self.entries.clear();
        for line in content.lines() {
            if line.trim().is_empty() { continue; }
            let p: PersistedEntry = serde_json::from_str(line)?;
            let entry = SessionEntry {
                content: p.content,
                tokens: p.tokens,
                timestamp: std::time::Instant::now(),
                access_count: 0,
            };
            let ts = DateTime::parse_from_rfc3339(&p.timestamp)
                .map_err(|e| io::Error::new(io::ErrorKind::InvalidData, e))?
                .with_timezone(&Utc);
            self.entries.insert(p.id, AutoEntry {
                session_entry: entry,
                file_path: file.clone(),
                last_persisted: ts,
                embedding: None, // 384维embedding预留，默认None
            });
        }
        self.dirty = false;
        Ok(())
    }

    /// 插入条目（延迟写入）
    pub fn insert(&mut self, key: String, entry: SessionEntry) -> Result<(), AutoError> {
        self.entries.insert(key, AutoEntry {
            session_entry: entry,
            file_path: self.storage_dir.join("memory.jsonl"),
            last_persisted: Utc::now(),
            embedding: None, // 384维embedding预留，默认None
        });
        self.dirty = true;
        Ok(())
    }

    /// 获取条目
    pub fn get(&self, key: &str) -> Option<&AutoEntry> { self.entries.get(key) }

    /// 获取可变条目
    pub fn get_mut(&mut self, key: &str) -> Option<&mut AutoEntry> {
        self.entries.get_mut(key).inspect(|_| { self.dirty = true; })
    }

    /// 删除条目（延迟写入）
    pub fn remove(&mut self, key: &str) -> Option<AutoEntry> {
        self.entries.remove(key).inspect(|_| { self.dirty = true; })
    }

    /// 清空（延迟写入）
    pub fn clear(&mut self) {
        if !self.entries.is_empty() { self.dirty = true; }
        self.entries.clear();
    }

    /// 从SessionMemory同步
    pub fn sync_from_session(&mut self, session: &SessionMemory) -> Result<(), AutoError> {
        for key in session.keys() {
            if let Some(se) = session.get(key) {
                let need = match self.entries.get(key) {
                    None => true,
                    Some(ae) => ae.session_entry.content != se.content || ae.session_entry.tokens != se.tokens,
                };
                if need { self.insert(key.clone(), se.clone())?; }
            }
        }
        Ok(())
    }

    /// 获取所有键
    pub fn keys(&self) -> impl Iterator<Item = &String> { self.entries.keys() }

    /// 是否包含键
    pub fn contains_key(&self, key: &str) -> bool { self.entries.contains_key(key) }
}

/// W27-AUDIT-001: Drop自动persist（数据安全）
/// Drop实现中使用let _ = 优雅忽略错误，禁止unwrap/expect
impl Drop for AutoMemory {
    fn drop(&mut self) {
        if self.dirty {
            let _ = self.persist(); // 必须：let _ = 忽略错误，禁止unwrap
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Instant;

    fn entry(content: &str, tokens: usize) -> SessionEntry {
        SessionEntry { content: content.to_string(), tokens, timestamp: Instant::now(), access_count: 0 }
    }

    #[test]
    fn test_new_creates_dir() -> Result<(), AutoError> {
        let auto = AutoMemory::new("test_new")?;
        assert!(auto.storage_dir().exists());
        assert!(!auto.is_dirty());
        Ok(())
    }

    #[test]
    fn test_invalid_project() {
        assert!(AutoMemory::new("").is_err());
        assert!(AutoMemory::new("a/b").is_err());
    }

    #[test]
    fn test_insert_get() -> Result<(), AutoError> {
        let mut auto = AutoMemory::new("test_insert")?;
        auto.insert("k1".to_string(), entry("test", 10))?;
        assert!(auto.is_dirty());
        assert_eq!(auto.get("k1").ok_or(AutoError::InvalidProjectId)?.session_entry.content, "test");
        Ok(())
    }

    #[test]
    fn test_get_none() -> Result<(), AutoError> {
        let auto = AutoMemory::new("test_get_none")?;
        assert!(auto.get("none").is_none());
        Ok(())
    }

    #[test]
    fn test_persist_load() -> Result<(), AutoError> {
        let pid = "test_persist";
        let mut a = AutoMemory::new(pid)?;
        a.insert("k1".to_string(), entry("data", 20))?;
        a.persist()?;
        assert!(!a.is_dirty());
        let mut b = AutoMemory::new(pid)?;
        b.load()?;
        assert_eq!(b.get("k1").ok_or(AutoError::InvalidProjectId)?.session_entry.content, "data");
        Ok(())
    }

    #[test]
    fn test_remove() -> Result<(), AutoError> {
        let mut auto = AutoMemory::new("test_remove")?;
        auto.insert("k1".to_string(), entry("a", 5))?;
        assert!(auto.remove("k1").is_some());
        assert!(auto.is_dirty());
        assert!(auto.is_empty());
        Ok(())
    }

    #[test]
    fn test_clear() -> Result<(), AutoError> {
        let mut auto = AutoMemory::new("test_clear")?;
        auto.insert("k1".to_string(), entry("a", 1))?;
        auto.clear();
        assert!(auto.is_empty() && auto.is_dirty());
        Ok(())
    }

    #[test]
    fn test_no_persist_clean() -> Result<(), AutoError> {
        let mut auto = AutoMemory::new("test_clean")?;
        auto.persist()?;
        assert!(!auto.is_dirty());
        Ok(())
    }

    #[test]
    fn test_update_key() -> Result<(), AutoError> {
        let mut auto = AutoMemory::new("test_update")?;
        auto.insert("k1".to_string(), entry("old", 5))?;
        auto.persist()?;
        auto.insert("k1".to_string(), entry("new", 10))?;
        assert_eq!(auto.get("k1").ok_or(AutoError::InvalidProjectId)?.session_entry.tokens, 10);
        Ok(())
    }
}
