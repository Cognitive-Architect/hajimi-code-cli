//! Ink Theme - B-W11/04
use ratatui::style::{Color, Modifier, Style};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;
use tokio::fs;
use tokio::sync::mpsc::Receiver;

// Solarized palette
const S_BASE03: Color = Color::Rgb(0, 43, 54);
const S_BASE01: Color = Color::Rgb(88, 110, 117);
const S_BASE00: Color = Color::Rgb(101, 123, 131);
const S_BASE0: Color = Color::Rgb(131, 148, 150);
const S_BASE1: Color = Color::Rgb(147, 161, 161);
const S_BASE3: Color = Color::Rgb(253, 246, 227);
const S_YELLOW: Color = Color::Rgb(181, 137, 0);
const S_RED: Color = Color::Rgb(220, 50, 47);
const S_BLUE: Color = Color::Rgb(38, 139, 210);
const S_CYAN: Color = Color::Rgb(42, 161, 152);
const S_GREEN: Color = Color::Rgb(133, 153, 0);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Theme {
    pub primary: Color, pub success: Color, pub error: Color,
    pub background: Color, pub foreground: Color, pub muted: Color,
}

impl Default for Theme {
    fn default() -> Self {
        Self { primary: S_CYAN, success: S_GREEN, error: S_RED,
            background: S_BASE03, foreground: S_BASE0, muted: S_BASE01 }
    }
}

impl Theme {
    pub fn style_primary(&self) -> Style { Style::default().fg(self.primary) }
    pub fn style_success(&self) -> Style { Style::default().fg(self.success) }
    pub fn style_error(&self) -> Style { Style::default().fg(self.error) }
    pub fn style_selected(&self) -> Style { Style::default().bg(self.primary).fg(self.background).add_modifier(Modifier::BOLD) }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode { Normal, Insert, Command }

impl InputMode {
    pub fn style(&self, theme: &Theme) -> Style {
        match self {
            InputMode::Normal => Style::default().fg(theme.primary).add_modifier(Modifier::BOLD),
            InputMode::Insert => Style::default().fg(theme.success).add_modifier(Modifier::BOLD),
            InputMode::Command => Style::default().fg(S_YELLOW).add_modifier(Modifier::BOLD),
        }
    }
}

/// Theme errors
#[derive(Debug, Clone, Error)]
pub enum ThemeError {
    #[error("Theme not found: {0}")]
    NotFound(String),
    #[error("IO error: {0}")]
    Io(String),
}

impl From<std::io::Error> for ThemeError {
    fn from(e: std::io::Error) -> Self { ThemeError::Io(e.to_string()) }
}

/// User theme preference for serialization
#[derive(Debug, Clone, Serialize, Deserialize)]
struct UserThemePreference { name: String }

/// Theme manager supporting runtime switching and persistence
pub struct ThemeManager {
    current: Theme,
    available: Vec<String>,
    watch_receiver: Option<Receiver<Theme>>,
}

impl ThemeManager {
    pub fn new() -> Self {
        let available = vec!["default".to_string(), "dark".to_string(), "light".to_string()];
        Self { current: Theme::default(), available, watch_receiver: None }
    }

    pub fn current_theme(&self) -> &Theme { &self.current }

    pub async fn switch_theme(&mut self, name: &str) -> Result<(), ThemeError> {
        let common = (S_BLUE, S_GREEN, S_RED);
        self.current = match name {
            "default" => Theme::default(),
            "dark" => Theme { primary: common.0, success: common.1, error: common.2,
                background: S_BASE03, foreground: S_BASE0, muted: S_BASE01 },
            "light" => Theme { primary: common.0, success: common.1, error: common.2,
                background: S_BASE3, foreground: S_BASE00, muted: S_BASE1 },
            _ => return Err(ThemeError::NotFound(name.to_string())),
        };
        Ok(())
    }

    pub fn list_themes(&self) -> &[String] { &self.available }

    pub async fn save_user_preference(&self, name: &str) -> Result<(), ThemeError> {
        let config_dir = home_config_dir().await?;
        fs::create_dir_all(&config_dir).await?;
        let content = serde_json::to_string_pretty(&UserThemePreference { name: name.to_string() })
            .map_err(|e| ThemeError::Io(e.to_string()))?;
        fs::write(config_dir.join("user_theme.json"), content).await?;
        Ok(())
    }
}

impl Default for ThemeManager {
    fn default() -> Self { Self::new() }
}

/// Get user config directory (~/.config/hajimi)
async fn home_config_dir() -> Result<PathBuf, ThemeError> {
    let home = std::env::var("HOME").or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| ThemeError::Io("HOME not set".to_string()))?;
    Ok(PathBuf::from(home).join(".config").join("hajimi"))
}
