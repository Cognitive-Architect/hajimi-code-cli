//! Image preview tool - B-02/06: view_image implementation
//! Supports PNG/JPG/WebP with ASCII preview and Base64 output

use super::{PermissionLevel, Tool, ToolArgs, ToolError, ToolErrorKind, ToolOutput, ToolPermissions};
use image::{DynamicImage, GenericImageView};
use serde_json::Value;
use std::path::PathBuf;
use base64::{Engine as _, engine::general_purpose::STANDARD as BASE64};

const MAX_IMAGE_SIZE: u64 = 50 * 1024 * 1024;
const MAX_ASCII_HEIGHT: u32 = 40;
const DEFAULT_MAX_WIDTH: u32 = 80;
const ASCII_CHARS: &[u8] = b" .:-=+*#%@";

pub struct ViewImageTool;

impl ViewImageTool {
    pub fn new() -> Self { Self }
}

fn human_size(size: u64) -> String {
    if size < 1024 { format!("{}B", size) }
    else if size < 1024 * 1024 { format!("{:.1}KB", size as f64 / 1024.0) }
    else { format!("{:.1}MB", size as f64 / (1024.0 * 1024.0)) }
}

fn is_image_file(path: &PathBuf) -> bool {
    match path.extension().and_then(|e| e.to_str()) {
        Some(ext) => matches!(ext.to_lowercase().as_str(), "png" | "jpg" | "jpeg" | "webp"),
        None => false,
    }
}

/// Validate image path and return appropriate error for invalid inputs
#[allow(dead_code)]
fn validate_image_path(path: &PathBuf) -> Result<(), ToolError> {
    if !path.exists() {
        return Err(ToolError { message: format!("Path not found: {}", path.display()), kind: ToolErrorKind::NotFound });
    }
    if !is_image_file(path) {
        return Err(ToolError { message: "Invalid format: only PNG/JPG/WebP supported".into(), kind: ToolErrorKind::InvalidFormat });
    }
    Ok(())
}

fn ascii_art(img: &DynamicImage, max_width: u32) -> String {
    let (orig_w, orig_h) = img.dimensions();
    let aspect = orig_h as f32 / orig_w.max(1) as f32;
    let width = max_width.min(orig_w).min(DEFAULT_MAX_WIDTH);
    let height = ((width as f32 * aspect) as u32).min(MAX_ASCII_HEIGHT);
    let resized = img.thumbnail(width, height);
    let (w, h) = resized.dimensions();
    let gray = resized.to_luma8();
    let mut out = String::with_capacity((w * h + h) as usize);
    for y in 0..h {
        for x in 0..w {
            let pixel = gray.get_pixel(x, y)[0] as usize;
            let idx = (pixel * (ASCII_CHARS.len() - 1)) / 255;
            out.push(ASCII_CHARS[idx] as char);
        }
        out.push('\n');
    }
    out
}

#[async_trait::async_trait]
impl Tool for ViewImageTool {
    fn name(&self) -> &str { "view_image" }
    fn description(&self) -> &str { "Preview image with ASCII art and metadata" }
    fn permissions(&self) -> ToolPermissions {
        ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None }
    }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let path = args.get("path").and_then(Value::as_str).ok_or_else(|| ToolError { message: "Missing path".into(), kind: ToolErrorKind::InvalidArgs })?;
        let path_buf: PathBuf = path.into();
        let meta = tokio::fs::metadata(&path_buf).await.map_err(|e| if e.kind() == std::io::ErrorKind::NotFound { ToolError { message: format!("Not found: {}", path), kind: ToolErrorKind::NotFound } } else { ToolError::new(format!("Access: {}", e)) })?;
        if !meta.is_file() { return Err(ToolError { message: "Not a file".into(), kind: ToolErrorKind::InvalidArgs }); }
        if meta.len() > MAX_IMAGE_SIZE { return Err(ToolError::new(format!("File too large: {}", human_size(meta.len())))); }
        if !is_image_file(&path_buf) { return Err(ToolError { message: "Invalid format: only PNG/JPG/WebP".into(), kind: ToolErrorKind::InvalidFormat }); }
        let img = image::open(&path_buf).map_err(|e| ToolError { message: format!("Parse error: {}", e), kind: ToolErrorKind::ParseError })?;
        let (width, height) = img.dimensions();
        let base64 = args.get("base64").and_then(Value::as_bool).unwrap_or(false);
        let ascii = args.get("ascii").and_then(Value::as_bool).unwrap_or(!base64);
        let max_width = args.get("max_width").and_then(Value::as_u64).map(|v| v as u32).unwrap_or(DEFAULT_MAX_WIDTH);
        let mut result = format!("Image: {}\nSize: {}\nDimensions: {}x{}\n", path, human_size(meta.len()), width, height);
        if ascii { result.push_str(&format!("\nASCII Preview ({} cols):\n{}", max_width, ascii_art(&img, max_width))); }
        if base64 { let data = std::fs::read(&path_buf).map_err(|e| ToolError::new(format!("Read: {}", e)))?; result.push_str(&format!("\nBase64:\n{}", BASE64.encode(&data))); }
        Ok(ToolOutput::success(result))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_human_size_bytes() { assert_eq!(human_size(512), "512B"); }
    #[test] fn test_human_size_kb() { assert_eq!(human_size(1536), "1.5KB"); }
    #[test] fn test_human_size_mb() { assert_eq!(human_size(2 * 1024 * 1024), "2.0MB"); }
    #[test] fn test_is_image_file_png() { assert!(is_image_file(&PathBuf::from("test.png"))); }
    #[test] fn test_is_image_file_jpg() { assert!(is_image_file(&PathBuf::from("test.jpg"))); }
    #[test] fn test_is_image_file_jpeg() { assert!(is_image_file(&PathBuf::from("test.jpeg"))); }
    #[test] fn test_is_image_file_webp() { assert!(is_image_file(&PathBuf::from("test.webp"))); }
    #[test] fn test_is_image_file_invalid() { assert!(!is_image_file(&PathBuf::from("test.txt"))); }
    #[test] fn test_view_image_tool_name() {
        let tool = ViewImageTool::new();
        assert_eq!(tool.name(), "view_image");
        assert_eq!(tool.description(), "Preview image with ASCII art and metadata");
    }
}
