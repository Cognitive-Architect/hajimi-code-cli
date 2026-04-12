//! Grep Tool - B-W10/01: Grep增强版

use async_trait::async_trait;
use regex::Regex;
use serde::Deserialize;
use std::io::Read;
use std::path::Path;
use tokio::io::AsyncBufReadExt;
use crate::{Tool, ToolArgs, ToolError, ToolOutput, ToolPermissions, PermissionLevel, Config};
use super::FindTool;

pub struct GrepTool;
impl GrepTool { pub fn new() -> Self { Self } }
impl Default for GrepTool { fn default() -> Self { Self::new() } }

/// Grep配置选项
#[derive(Debug, Clone)]
pub struct GrepOptions { pub context_lines: usize, pub before_context: usize, pub after_context: usize, pub highlight: bool }
impl Default for GrepOptions { fn default() -> Self { Self { context_lines: 0, before_context: 0, after_context: 0, highlight: true } } }
impl GrepOptions { pub fn with_context(n: usize) -> Self { Self { context_lines: n, before_context: n, after_context: n, highlight: true } } }

#[derive(Debug, Deserialize)]
struct GrepArgs { pattern: String, path: String, #[serde(default)] recursive: bool, #[serde(default)] files_from: Option<Vec<String>>, #[serde(default)] context_lines: Option<usize>, #[serde(default)] no_highlight: bool }

/// ANSI高亮匹配文本
pub fn highlight_match(line: &str, pattern: &Regex) -> String { pattern.replace_all(line, "\x1b[1;31m$0\x1b[0m").to_string() }

/// 检测二进制文件（复用Week 9）
fn is_binary(path: &Path) -> Result<bool, ToolError> {
    let mut file = std::fs::File::open(path).map_err(|e| ToolError::new(format!("Open: {}", e)))?;
    let mut buf = [0u8; 512]; let n = file.read(&mut buf).map_err(|e| ToolError::new(format!("Read: {}", e)))?;
    Ok(std::str::from_utf8(&buf[..n]).is_err())
}

/// 流式搜索文件
async fn grep_stream(path: &Path, rx: &Regex, opts: &GrepOptions) -> Result<Vec<String>, ToolError> {
    if is_binary(path)? { return Ok(vec![]); }
    let file = tokio::fs::File::open(path).await.map_err(|e| ToolError::new(format!("Open: {}", e)))?;
    let mut lines = tokio::io::BufReader::new(file).lines();
    let (before, after) = (opts.before_context.max(opts.context_lines), opts.after_context.max(opts.context_lines));
    let mut buffer: Vec<(usize, String)> = Vec::with_capacity(before + 1); let mut results = Vec::new(); let mut n = 0usize;
    while let Ok(Some(line)) = lines.next_line().await {
        n += 1;
        if rx.is_match(&line) {
            let hl = if opts.highlight { highlight_match(&line, rx) } else { line.clone() };
            results.push(format!("{}:{}:{}", path.display(), n, hl));
            for (ln, l) in buffer.iter().rev().take(before) { results.push(format!("{}:{}-{}", path.display(), ln, l)); }
            for _ in 0..after { match lines.next_line().await { Ok(Some(ln)) => { n += 1; results.push(format!("{}:{}+{}", path.display(), n, ln)); }, _ => break } }
        } else if buffer.len() < before { buffer.push((n, line)); } else { buffer.remove(0); buffer.push((n, line)); }
    }
    Ok(results)
}

#[async_trait]
impl Tool for GrepTool {
    fn name(&self) -> &str { "grep" }
    fn description(&self) -> &str { "Search with regex, context, highlight" }
    fn permissions(&self) -> ToolPermissions { ToolPermissions { default_level: PermissionLevel::Allow, requires_confirmation: false, allowed_paths: None } }
    fn is_enabled(&self, _config: &Config) -> bool { true }
    async fn execute(&self, args: ToolArgs) -> Result<ToolOutput, ToolError> {
        let a: GrepArgs = serde_json::from_value(args).map_err(|e| ToolError::new(format!("Args: {}", e)))?;
        let rx = Regex::new(&a.pattern).map_err(|e| ToolError::new(format!("Regex: {}", e)))?;
        let ctx = a.context_lines.unwrap_or(0); let opts = GrepOptions { context_lines: ctx, before_context: ctx, after_context: ctx, highlight: !a.no_highlight };
        let mut files: Vec<std::path::PathBuf> = Vec::new();
        if let Some(list) = a.files_from { for f in list { files.push(f.into()); } }
        else {
            let p = Path::new(&a.path); if !p.exists() { return Err(ToolError::new(format!("Not found: {}", a.path))); }
            if p.is_file() { files.push(p.to_path_buf()); }
            else if a.recursive && p.is_dir() { let mut d = tokio::fs::read_dir(p).await.map_err(|e| ToolError::new(format!("Dir: {}", e)))?; while let Ok(Some(e)) = d.next_entry().await { if e.path().is_file() { files.push(e.path()); } } }
        }
        let mut all = Vec::new(); for f in &files { match grep_stream(f, &rx, &opts).await { Ok(r) => all.extend(r), Err(_) => continue } }
        Ok(ToolOutput { stdout: all.join("\n"), stderr: String::new(), exit_code: Some(if all.is_empty() { 1 } else { 0 }) })
    }
}

/// Find + Grep管道集成
pub struct FindGrepIntegration;
impl FindGrepIntegration {
    pub fn new() -> Self { Self }
    pub async fn pipe(find: &FindTool, pattern: &str, opts: &GrepOptions) -> Result<ToolOutput, ToolError> {
        let rx = Regex::new(pattern).map_err(|e| ToolError::new(format!("Regex: {}", e)))?;
        let find_out = find.execute(serde_json::json!({"path": "."})).await?;
        let mut all = Vec::new();
        for f in find_out.stdout.lines() { match grep_stream(f.as_ref(), &rx, opts).await { Ok(r) => all.extend(r), Err(_) => continue } }
        Ok(ToolOutput { stdout: all.join("\n"), stderr: String::new(), exit_code: Some(if all.is_empty() { 1 } else { 0 }) })
    }
}
impl Default for FindGrepIntegration { fn default() -> Self { Self::new() } }

/// Input types
#[derive(Debug, Clone)]
pub enum GrepInput { Path(String), FileList(Vec<String>) }
impl From<Vec<String>> for GrepInput { fn from(f: Vec<String>) -> Self { Self::FileList(f) } }
impl GrepInput {
    pub async fn expand(self) -> Result<Vec<String>, ToolError> {
        match self {
            Self::Path(p) => {
                let path = Path::new(&p); if !path.exists() { return Err(ToolError::new(format!("Not found: {}", p))); }
                if path.is_file() { return Ok(vec![p]); }
                let mut f = Vec::new(); let mut d = tokio::fs::read_dir(path).await.map_err(|e| ToolError::new(format!("Dir: {}", e)))?;
                while let Ok(Some(e)) = d.next_entry().await { if e.path().is_file() { f.push(e.path().to_string_lossy().to_string()); } }
                Ok(f)
            }
            Self::FileList(f) => Ok(f),
        }
    }
}
