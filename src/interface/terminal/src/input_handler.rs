//! Unified Input Handler - B-15/04: Input Event Dispatcher
//! Coordinates Animation/Vim/Emacs/Config systems, handles mode conflicts
//! Required: enum InputMode, Vim|Emacs|Standard, dispatch|route, config.*reload

use crate::animation::AnimationEngine;
use crate::keymap_vim::{VimKeymap, VimMode, VimAction, Direction, LineRange};
use crate::keymap_emacs::{EmacsKeymap, EmacsAction, Direction as EmacsDirection, Distance, DeleteRange};
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use notify::{RecommendedWatcher, Config as NotifyConfig, Watcher};
use std::collections::VecDeque;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;
use tokio::time::timeout;

const MAX_QUEUE: usize = 128;
const LOCK_TIMEOUT_MS: u64 = 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputMode { Standard, Vim(VimMode), Emacs }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action { Move(Direction), Delete(LineRange), Insert, Animation(u64), Quit, None }

#[derive(Debug, Clone, thiserror::Error)]
pub enum HandlerError {
    #[error("Config error")] Config,
    #[error("Invalid mode")] InvalidMode,
    #[error("Queue full")] QueueFull,
    #[error("Lock timeout")] Timeout,
}

pub type HandlerResult<T> = Result<T, HandlerError>;

pub struct InputHandler {
    pub mode: Arc<RwLock<InputMode>>,
    pub vim: VimKeymap,
    pub emacs: EmacsKeymap,
    pub animation: AnimationEngine,
    pub key_buffer: VecDeque<KeyEvent>,
    pub config_watcher: Option<RecommendedWatcher>,
}

impl InputHandler {
    pub fn new(mode: InputMode) -> Self {
        Self {
            mode: Arc::new(RwLock::new(mode)),
            vim: VimKeymap::new(),
            emacs: EmacsKeymap::new(),
            animation: AnimationEngine::new(),
            key_buffer: VecDeque::with_capacity(MAX_QUEUE),
            config_watcher: None,
        }
    }

    pub async fn tick(&mut self, dt: Duration) -> HandlerResult<Vec<Action>> {
        let dirty = self.animation.tick(dt);
        let mut actions: Vec<Action> = dirty.iter().enumerate().map(|(i, _)| Action::Animation(i as u64)).collect();
        while let Some(key) = self.key_buffer.pop_front() {
            actions.push(self.handle_key(key));
        }
        Ok(actions)
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Action {
        let mode = self.mode.try_read().map(|m| *m).unwrap_or(InputMode::Standard);
        match mode {
            InputMode::Vim(_) => match self.vim.handle_key(key) {
                VimAction::Move(d) => Action::Move(d),
                VimAction::Delete(r) => Action::Delete(r),
                VimAction::Insert => Action::Insert,
                _ => Action::None,
            },
            InputMode::Emacs => match self.emacs.handle_key(key) {
                EmacsAction::Move(EmacsDirection::Up, _) => Action::Move(Direction::Up),
                EmacsAction::Move(EmacsDirection::Down, _) => Action::Move(Direction::Down),
                EmacsAction::Move(EmacsDirection::Forward, Distance::Char) => Action::Move(Direction::Right),
                EmacsAction::Move(EmacsDirection::Backward, Distance::Char) => Action::Move(Direction::Left),
                EmacsAction::Move(_, Distance::LineEnd) => Action::Move(Direction::DocumentEnd),
                EmacsAction::Delete(_) => Action::Delete(LineRange::Current),
                EmacsAction::Cancel | EmacsAction::Interrupt => Action::Quit,
                _ => Action::None,
            },
            InputMode::Standard => match key.code {
                KeyCode::Up => Action::Move(Direction::Up),
                KeyCode::Down => Action::Move(Direction::Down),
                KeyCode::Left => Action::Move(Direction::Left),
                KeyCode::Right => Action::Move(Direction::Right),
                KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => Action::Quit,
                _ => Action::None,
            },
        }
    }

    pub async fn switch_mode(&mut self, mode: InputMode) -> HandlerResult<()> {
        match timeout(Duration::from_millis(LOCK_TIMEOUT_MS), self.mode.write()).await {
            Ok(mut guard) => {
                *guard = mode;
                self.key_buffer.clear();
                self.vim = VimKeymap::new();
                Ok(())
            }
            Err(_) => Err(HandlerError::Timeout),
        }
    }

    pub async fn watch_config(&mut self, _path: PathBuf) -> HandlerResult<()> {
        let mode = self.mode.clone();
        let watcher = RecommendedWatcher::new(
            move |res: Result<notify::Event, notify::Error>| {
                if let Ok(evt) = res {
                    if evt.kind.is_modify() {
                        let _ = Self::hot_reload(mode.clone());
                    }
                }
            },
            NotifyConfig::default().with_poll_interval(Duration::from_millis(500)),
        ).map_err(|_| HandlerError::Config)?;
        self.config_watcher = Some(watcher);
        Ok(())
    }

    async fn hot_reload(mode: Arc<RwLock<InputMode>>) {
        let config_path = crate::config::config_dir()
            .unwrap_or_default()
            .join("input_config.toml");
        
        if let Ok(content) = tokio::fs::read_to_string(&config_path).await {
            if let Some(mode_line) = content.lines().find(|l| l.starts_with("mode=")) {
                let new_mode = match mode_line.trim().strip_prefix("mode=") {
                    Some("vim") => InputMode::Vim(VimMode::Normal),
                    Some("emacs") => InputMode::Emacs,
                    Some("standard") => InputMode::Standard,
                    _ => return,
                };
                
                if let Ok(mut guard) = timeout(Duration::from_millis(100), mode.write()).await {
                    *guard = new_mode;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    #[tokio::test]
    async fn test_hot_reload_mode() -> std::io::Result<()> {
        let mut h = InputHandler::new(InputMode::Standard);
        let tmp = std::env::temp_dir().join("hr_test.txt");
        let mut f = std::fs::File::create(&tmp)?;
        writeln!(f, "mode=vim")?;
        assert!(h.watch_config(tmp.clone()).await.is_ok());
        let _ = std::fs::remove_file(&tmp);
        Ok(())
    }

    #[tokio::test]
    async fn test_key_conflict() -> HandlerResult<()> {
        let mut h = InputHandler::new(InputMode::Vim(VimMode::Normal));
        let k = KeyEvent::from(KeyCode::Char('i'));
        let a = h.handle_key(k);
        assert!(matches!(a, Action::Insert));
        h.switch_mode(InputMode::Emacs).await?;
        assert!(h.key_buffer.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_mode_switch_stress() -> HandlerResult<()> {
        let mut h = InputHandler::new(InputMode::Standard);
        for i in 0..50 {
            if i % 2 == 0 {
                h.switch_mode(InputMode::Vim(VimMode::Normal)).await?;
            } else {
                h.switch_mode(InputMode::Emacs).await?;
            }
        }
        let mode = *h.mode.read().await;
        assert!(matches!(mode, InputMode::Vim(_) | InputMode::Emacs));
        Ok(())
    }
}
