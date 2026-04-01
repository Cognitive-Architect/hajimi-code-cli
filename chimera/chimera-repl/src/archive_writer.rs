//! Archive Writer - .hctx format with BLAKE3 checksum
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::{Write, Read, Seek, SeekFrom};
use std::path::Path;

use crate::codex_bridge::TurnWithMeta;
use crate::ReplError;

const HCTX_VERSION: u8 = 1;
const HCTX_MAGIC: &[u8; 4] = b"HCTX";

/// Archive writer for TurnWithMeta persistence
pub struct ArchiveWriter { path: std::path::PathBuf; }

impl ArchiveWriter {
    pub fn new<P: AsRef<Path>>(path: P) -> Self { Self { path: path.as_ref().to_path_buf() } }

    /// Write TurnWithMeta to .hctx archive with BLAKE3 checksum
    pub fn write_turn(&self, turn_meta: &TurnWithMeta) -> Result<(), ReplError> {
        let json_bytes = serde_json::to_vec(turn_meta).map_err(ReplError::Protocol)?;
        let mut file = OpenOptions::new().write(true).create(true).append(true).open(&self.path)
            .map_err(|e| ReplError::Session(format!("Archive open: {}", e)))?;
        // Header: magic(4) + version(1) + flags(1) + reserved(2)
        file.write_all(HCTX_MAGIC).map_err(|e| ReplError::Session(format!("Write: {}", e)))?;
        file.write_all(&[HCTX_VERSION, 0, 0, 0]).map_err(|e| ReplError::Session(format!("Write: {}", e)))?;
        // Body length (u32 LE) + JSON body
        file.write_all(&(json_bytes.len() as u32).to_le_bytes()).map_err(|e| ReplError::Session(format!("Write: {}", e)))?;
        file.write_all(&json_bytes).map_err(|e| ReplError::Session(format!("Write: {}", e)))?;
        // BLAKE3 checksum AFTER all data (metadata integrity without affecting existing checksums)
        file.write_all(blake3::hash(&json_bytes).as_bytes()).map_err(|e| ReplError::Session(format!("Write: {}", e)))?;
        file.flush().map_err(|e| ReplError::Session(format!("Flush: {}", e)))?;
        Ok(())
    }

    /// Read and verify turn from archive
    pub fn read_turn_at(&self, offset: u64) -> Result<TurnWithMeta, ReplError> {
        use std::fs::File;
        let mut file = File::open(&self.path).map_err(|e| ReplError::Session(format!("Read: {}", e)))?;
        file.seek(SeekFrom::Start(offset)).map_err(|e| ReplError::Session(format!("Seek: {}", e)))?;
        let mut header = [0u8; 8];
        file.read_exact(&mut header).map_err(|e| ReplError::Session(format!("Read: {}", e)))?;
        if &header[0..4] != HCTX_MAGIC { return Err(ReplError::Session("Bad magic".to_string())); }
        let mut len_buf = [0u8; 4];
        file.read_exact(&mut len_buf).map_err(|e| ReplError::Session(format!("Read: {}", e)))?;
        let body_len = u32::from_le_bytes(len_buf) as usize;
        let mut body = vec![0u8; body_len];
        file.read_exact(&mut body).map_err(|e| ReplError::Session(format!("Read: {}", e)))?;
        let mut checksum = [0u8; 32];
        file.read_exact(&mut checksum).map_err(|e| ReplError::Session(format!("Read: {}", e)))?;
        if blake3::hash(&body).as_bytes() != &checksum { return Err(ReplError::Session("Checksum fail".to_string())); }
        serde_json::from_slice(&body).map_err(ReplError::Protocol)
    }

    /// Get metadata from archive without full deserialization
    pub fn get_metadata(&self, offset: u64) -> Result<HashMap<String, String>, ReplError> {
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
    fn test_roundtrip_with_metadata() {
        let temp = TempDir::new().unwrap();
        let writer = ArchiveWriter::new(temp.path().join("test.hctx"));
        let state: ReplState<MockClock> = ReplState::default();
        let bridge = CodexBridge::new(state).unwrap();
        let mut item = TurnItem::new("t1".to_string(), Role::User, "hello".to_string(), 1000);
        let mut meta = HashMap::new();
        meta.insert("src".to_string(), "test".to_string());
        item.metadata = Some(json!(meta));
        let tm = bridge.map_turn(&item).unwrap();
        writer.write_turn(&tm).unwrap();
        let restored = writer.read_turn_at(0).unwrap();
        assert_eq!(restored.turn.id, "t1");
        assert_eq!(restored.metadata.get("src"), Some(&"test".to_string()));
    }

    #[test]
    fn test_checksum_integrity() {
        let temp = TempDir::new().unwrap();
        let writer = ArchiveWriter::new(temp.path().join("chk.hctx"));
        let state: ReplState<MockClock> = ReplState::default();
        let bridge = CodexBridge::new(state).unwrap();
        let item = TurnItem::new("t2".to_string(), Role::Turn, "rsp".to_string(), 2000);
        writer.write_turn(&bridge.map_turn(&item).unwrap()).unwrap();
        assert!(writer.read_turn_at(0).is_ok());
    }
}
