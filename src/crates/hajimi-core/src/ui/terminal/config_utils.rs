//! Config utilities - DEBT-LINES-B16-02
use std::io::Write;
use std::path::Path;

/// Atomically write content to file (write to temp then rename)
pub fn atomic_write_file<P: AsRef<Path>>(path: P, content: &[u8]) -> std::io::Result<()> {
    let p = path.as_ref();
    let dir = p.parent().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "invalid path"))?;
    std::fs::create_dir_all(dir)?;
    let tmp = dir.join(format!(".tmp.{}.{}", p.file_name().unwrap_or_default().to_string_lossy(), std::process::id()));
    { let mut f = std::fs::File::create(&tmp)?; f.write_all(content)?; f.sync_all()?; }
    std::fs::rename(&tmp, p)?;
    Ok(())
}

/// Check if path has json extension
pub fn is_json<P: AsRef<Path>>(path: P) -> bool {
    path.as_ref().extension().map(|e| e == "json").unwrap_or(false)
}
