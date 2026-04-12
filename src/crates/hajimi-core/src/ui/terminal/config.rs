//! Theme config with hot reload - B-14/01
use crate::ui::terminal::config_utils::{atomic_write_file, is_json};
use crate::ui::terminal::theme::Theme;
use notify::{Config, Event, RecommendedWatcher, RecursiveMode, Watcher};
use ratatui::style::Color;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Config file not found")] NotFound,
    #[error("Invalid format: {0}")] InvalidFormat(String),
    #[error("Permission denied")] PermissionDenied,
    #[error("Invalid color: {0}")] InvalidColor(String),
    #[error("IO error: {0}")] Io(#[from] std::io::Error),
}

#[derive(Serialize, Deserialize)]
struct ThemeConfig { primary: String, success: String, error: String, background: String, foreground: String, muted: String }

fn parse_color(s: &str) -> Result<Color, ConfigError> {
    if s.starts_with('#') && s.len() == 7 {
        let r = u8::from_str_radix(&s[1..3], 16).map_err(|_| ConfigError::InvalidColor(s.to_string()))?;
        let g = u8::from_str_radix(&s[3..5], 16).map_err(|_| ConfigError::InvalidColor(s.to_string()))?;
        let b = u8::from_str_radix(&s[5..7], 16).map_err(|_| ConfigError::InvalidColor(s.to_string()))?;
        return Ok(Color::Rgb(r, g, b));
    }
    match s.to_lowercase().as_str() {
        "black" => Ok(Color::Black), "red" => Ok(Color::Red), "green" => Ok(Color::Green), "yellow" => Ok(Color::Yellow),
        "blue" => Ok(Color::Blue), "magenta" => Ok(Color::Magenta), "cyan" => Ok(Color::Cyan), "gray" => Ok(Color::Gray), "white" => Ok(Color::White),
        _ => Err(ConfigError::InvalidColor(s.to_string())),
    }
}

fn color_to_hex(c: Color) -> String {
    match c {
        Color::Rgb(r, g, b) => format!("#{:02x}{:02x}{:02x}", r, g, b), Color::Black => "#000000".into(), Color::Red => "#ff0000".into(),
        Color::Green => "#00ff00".into(), Color::Yellow => "#ffff00".into(), Color::Blue => "#0000ff".into(), Color::Magenta => "#ff00ff".into(),
        Color::Cyan => "#00ffff".into(), Color::Gray => "#808080".into(), Color::White => "#ffffff".into(), _ => "#000000".into(),
    }
}

fn map_io_err(e: std::io::Error) -> ConfigError {
    match e.kind() { std::io::ErrorKind::NotFound => ConfigError::NotFound, std::io::ErrorKind::PermissionDenied => ConfigError::PermissionDenied, _ => ConfigError::Io(e) }
}

pub fn load_theme_from_file<P: AsRef<Path>>(path: P) -> Result<Theme, ConfigError> {
    let p = path.as_ref(); let content = std::fs::read_to_string(p).map_err(map_io_err)?;
    let cfg: ThemeConfig = if is_json(p) { serde_json::from_str(&content).map_err(|e| ConfigError::InvalidFormat(e.to_string()))? } else { toml::from_str(&content).map_err(|e| ConfigError::InvalidFormat(e.to_string()))? };
    Ok(Theme { primary: parse_color(&cfg.primary)?, success: parse_color(&cfg.success)?, error: parse_color(&cfg.error)?, background: parse_color(&cfg.background)?, foreground: parse_color(&cfg.foreground)?, muted: parse_color(&cfg.muted)? })
}

pub fn save_theme_to_file<P: AsRef<Path>>(theme: &Theme, path: P) -> Result<(), ConfigError> {
    let cfg = ThemeConfig { primary: color_to_hex(theme.primary), success: color_to_hex(theme.success), error: color_to_hex(theme.error), background: color_to_hex(theme.background), foreground: color_to_hex(theme.foreground), muted: color_to_hex(theme.muted) };
    let content = if is_json(&path) { serde_json::to_string_pretty(&cfg).map_err(|e| ConfigError::InvalidFormat(e.to_string()))? } else { toml::to_string_pretty(&cfg).map_err(|e| ConfigError::InvalidFormat(e.to_string()))? };
    atomic_write_file(&path, content.as_bytes()).map_err(ConfigError::Io)
}

pub fn config_dir() -> Option<PathBuf> { dirs::config_dir().map(|d| d.join("hajimi")) }

pub async fn watch_theme_file<P: AsRef<Path>>(path: P, theme: Arc<RwLock<Theme>>) -> Result<(), ConfigError> {
    let (tx, mut rx) = mpsc::channel(4); let p = path.as_ref().to_path_buf();
    let mut watcher: RecommendedWatcher = Watcher::new(
        move |res: Result<Event, notify::Error>| { if let Ok(evt) = res { let _ = tx.try_send(evt); } },
        Config::default().with_poll_interval(std::time::Duration::from_millis(100))
    ).map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;
    watcher.watch(&p, RecursiveMode::NonRecursive).map_err(|e| ConfigError::InvalidFormat(e.to_string()))?;
    while let Some(evt) = rx.recv().await { if evt.kind.is_modify() { if let Ok(t) = load_theme_from_file(&p) { let mut g = theme.write().await; *g = t; } } }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_parse_color_hex() -> Result<(), ConfigError> { assert!(matches!(parse_color("#ff0000")?, Color::Rgb(255, 0, 0))); Ok(()) }
    #[test] fn test_parse_color_named() -> Result<(), ConfigError> { assert_eq!(parse_color("red")?, Color::Red); Ok(()) }
    #[test] fn test_color_to_hex() { assert_eq!(color_to_hex(Color::Rgb(255, 0, 0)), "#ff0000"); }
}
