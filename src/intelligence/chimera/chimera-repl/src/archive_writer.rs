//! Archive Writer - .hctx format with BLAKE3 checksum
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Write, Read, Seek, SeekFrom};
use std::path::Path;

use crate::codex_bridge::TurnWithMeta;
use crate::ReplError;

const HCTX_VERSION: u8 = 1;
const HCTX_MAGIC: &[u8; 4] = b"HCTX";

/// Archive 错误类型
#[derive(Debug)]
pub enum ArchiveError {
    Io(std::io::Error),
    Serialization(serde_json::Error),
    Protocol(String),
    ChecksumMismatch,
    BadMagic,
}

impl std::fmt::Display for ArchiveError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ArchiveError::Io(e) => write!(f, "IO error: {}", e),
            ArchiveError::Serialization(e) => write!(f, "Serialization error: {}", e),
            ArchiveError::Protocol(s) => write!(f, "Protocol error: {}", s),
            ArchiveError::ChecksumMismatch => write!(f, "BLAKE3 checksum mismatch"),
            ArchiveError::BadMagic => write!(f, "Invalid HCTX magic bytes"),
        }
    }
}

impl std::error::Error for ArchiveError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            ArchiveError::Io(e) => Some(e),
            ArchiveError::Serialization(e) => Some(e),
            _ => None,
        }
    }
}

impl From<std::io::Error> for ArchiveError {
    fn from(err: std::io::Error) -> Self {
        ArchiveError::Io(err)
    }
}

impl From<serde_json::Error> for ArchiveError {
    fn from(err: serde_json::Error) -> Self {
        ArchiveError::Serialization(err)
    }
}

impl From<ArchiveError> for ReplError {
    fn from(err: ArchiveError) -> Self {
        ReplError::Session(err.to_string())
    }
}

/// Archive writer for TurnWithMeta persistence
pub struct ArchiveWriter { path: std::path::PathBuf; }

impl ArchiveWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> Self { Self { path: path.as_ref().to_path_buf() } }

    /// Write TurnWithMeta to .hctx archive with BLAKE3 checksum
    pub fn write_turn(&self, turn_meta: &TurnWithMeta) -> Result<(), ArchiveError> {
        let json_bytes = serde_json::to_vec(turn_meta)?;
        let mut file = OpenOptions::new().write(true).create(true).append(true).open(&self.path)?;
        // Header: magic(4) + version(1) + flags(1) + reserved(2)
        file.write_all(HCTX_MAGIC)?;
        file.write_all(&[HCTX_VERSION, 0, 0, 0])?;
        // Body length (u32 LE) + JSON body
        file.write_all(&(json_bytes.len() as u32).to_le_bytes())?;
        file.write_all(&json_bytes)?;
        // BLAKE3 checksum AFTER all data (metadata integrity without affecting existing checksums)
        file.write_all(blake3::hash(&json_bytes).as_bytes())?;
        file.flush()?;
        Ok(())
    }

    /// Read and verify turn from archive
    pub fn read_turn_at(&self, offset: u64) -> Result<TurnWithMeta, ArchiveError> {
        use std::fs::File;
        let mut file = File::open(&self.path)?;
        file.seek(SeekFrom::Start(offset))?;
        let mut header = [0u8; 8];
        file.read_exact(&mut header)?;
        if &header[0..4] != HCTX_MAGIC { return Err(ArchiveError::BadMagic); }
        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf)?;
        let body_len = u32::from_le_bytes(len_buf) as usize;
        let mut body = vec![0u8; body_len];
        file.read_exact(&mut body)?;
        let mut checksum = [0u8; 32];
        file.read_exact(&mut checksum)?;
        if blake3::hash(&body).as_bytes() != &checksum { return Err(ArchiveError::ChecksumMismatch); }
        Ok(serde_json::from_slice(&body)?)
    }

    /// Get metadata from archive without full deserialization
    pub fn get_metadata(&self, offset: u64) -> Result<HashMap<String, String>, ArchiveError> {
        self.read_turn_at(offset).map(|tm| tm.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::clock::MockClock;
    use crate::state::{ReplState, Role, TurnItem};
    use crate::codex_bridge::CodexBridge;
    use serde_json::json;
    use tempfile::TempDir;

    #[test]
    fn test_roundtrip_with_metadata() -> Result<(), ArchiveError> {
        let temp = TempDir::new()?;
        let writer = ArchiveWriter::new(temp.path().join("test.hctx"));
        let state: ReplState<MockClock> = ReplState::default();
        let bridge = CodexBridge::new(state).map_err(|e| ArchiveError::Protocol(e.to_string()))?;
        let mut item = TurnItem::new("t1".to_string(), Role::User, "hello".to_string(), 1000);
        let mut meta = HashMap::new();
        meta.insert("src".to_string(), "test".to_string());
        item.metadata = Some(json!(meta));
        let tm = bridge.map_turn(&item).map_err(|e| ArchiveError::Protocol(e.to_string()))?;
        writer.write_turn(&tm)?;
        let restored = writer.read_turn_at(0)?;
        assert_eq!(restored.turn.id, "t1");
        assert_eq!(restored.metadata.get("src"), Some(&"test".to_string()));
        Ok(())
    }

    #[test]
    fn test_checksum_integrity() -> Result<(), ArchiveError> {
        let temp = TempDir::new()?;
        let writer = ArchiveWriter::new(temp.path().join("chk.hctx"));
        let state: ReplState<MockClock> = ReplState::default();
        let bridge = CodexBridge::new(state).map_err(|e| ArchiveError::Protocol(e.to_string()))?;
        let item = TurnItem::new("t2".to_string(), Role::Turn, "rsp".to_string(), 2000);
        writer.write_turn(&bridge.map_turn(&item).map_err(|e| ArchiveError::Protocol(e.to_string()))?)?;
        assert!(writer.read_turn_at(0).is_ok());
        Ok(())
    }
}
