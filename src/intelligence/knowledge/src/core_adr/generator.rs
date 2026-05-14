//! ADR自动生成器（编号生成与文件创建）
use crate::core_adr::parser::generate_frontmatter;
use crate::core_adr::{AdrError, AdrStatus, Result};
use regex::Regex;
use std::sync::Mutex;

pub struct AdrGenerator {
    next_id: Mutex<usize>,
    dir: std::path::PathBuf,
}

impl AdrGenerator {
    pub fn new(dir: std::path::PathBuf) -> Result<Self> {
        let mut max_id = 0usize;
        let id_re = Regex::new(r"ADR-(\d{4})").map_err(|e| AdrError::Parse(e.to_string()))?;
        if dir.exists() {
            for entry in std::fs::read_dir(&dir).map_err(AdrError::Io)? {
                let entry = entry.map_err(AdrError::Io)?;
                let name = entry.file_name().to_string_lossy().to_string();
                if let Some(cap) = id_re.captures(&name) {
                    if let Some(id_str) = cap.get(1) {
                        if let Ok(id) = id_str.as_str().parse::<usize>() {
                            max_id = max_id.max(id);
                        }
                    }
                }
            }
        }
        Ok(Self {
            next_id: Mutex::new(max_id + 1),
            dir,
        })
    }
    pub fn next_id(&self) -> Result<String> {
        let mut guard = self
            .next_id
            .lock()
            .map_err(|e| AdrError::Lock(e.to_string()))?;
        let id = *guard;
        *guard += 1;
        Ok(format!("ADR-{:04}", id))
    }
    pub fn create_adr(&self, title: &str) -> Result<std::path::PathBuf> {
        let id = self.next_id()?;
        let filename = format!("{}-{}.md", id, title.to_lowercase().replace(' ', "-"));
        let path = self.dir.join(&filename);
        let content = generate_frontmatter(title, AdrStatus::Proposed);
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent).map_err(AdrError::Io)?;
            }
        }
        let temp_path = path.with_extension("tmp");
        std::fs::write(&temp_path, content).map_err(AdrError::Io)?;
        std::fs::rename(&temp_path, &path).map_err(AdrError::Io)?;
        Ok(path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_adr_generator() -> Result<()> {
        let tmp = TempDir::new()?;
        let gen = AdrGenerator::new(tmp.path().to_path_buf())?;
        assert_eq!(gen.next_id()?, "ADR-0001");
        assert_eq!(gen.next_id()?, "ADR-0002");
        Ok(())
    }

    #[test]
    fn test_adr_create() -> Result<()> {
        let tmp = TempDir::new()?;
        let gen = AdrGenerator::new(tmp.path().to_path_buf())?;
        let path = gen.create_adr("Test Decision")?;
        assert!(path.exists());
        let content = fs::read_to_string(&path)?;
        assert!(content.contains("Test Decision"));
        assert!(content.contains("status: proposed"));
        Ok(())
    }
}
