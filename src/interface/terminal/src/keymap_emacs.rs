//! Emacs Keymap - B-15/03
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

const MULTI_KEY_TIMEOUT: Duration = Duration::from_millis(500);

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction { Forward, Backward, Up, Down }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Distance { Char, Line, LineEnd }

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DeleteRange { Char, ToLineEnd }

#[derive(Debug, Clone, PartialEq)]
pub enum EmacsAction {
    Move(Direction, Distance),
    Delete(DeleteRange),
    Cancel, Interrupt,
    Insert(char), None,
}

pub struct EmacsKeymap {
    last_key_time: Instant,
    multi_key_buffer: Vec<KeyEvent>,
}

impl EmacsKeymap {
    pub fn new() -> Self {
        Self { last_key_time: Instant::now(), multi_key_buffer: Vec::with_capacity(4) }
    }
    pub fn is_control_pressed(&self, key: &KeyEvent) -> bool {
        key.modifiers.contains(KeyModifiers::CONTROL)
    }
    fn check_timeout(&mut self) {
        if self.last_key_time.elapsed() > MULTI_KEY_TIMEOUT { self.multi_key_buffer.clear(); }
    }
    fn record_key(&mut self, key: KeyEvent) {
        self.check_timeout();
        self.multi_key_buffer.push(key);
        self.last_key_time = Instant::now();
    }
    fn is_double_ctrl_c(&self) -> bool {
        let len = self.multi_key_buffer.len();
        if len < 2 { return false; }
        let is_c = |k: &KeyEvent| matches!(k.code, KeyCode::Char('c')) && k.modifiers.contains(KeyModifiers::CONTROL);
        is_c(&self.multi_key_buffer[len - 1]) && is_c(&self.multi_key_buffer[len - 2])
    }
    pub fn handle_key(&mut self, key: KeyEvent) -> EmacsAction {
        self.record_key(key);
        if self.is_double_ctrl_c() { return EmacsAction::Interrupt; }
        if self.is_control_pressed(&key) {
            match key.code {
                KeyCode::Char('f') => EmacsAction::Move(Direction::Forward, Distance::Char),
                KeyCode::Char('b') => EmacsAction::Move(Direction::Backward, Distance::Char),
                KeyCode::Char('n') => EmacsAction::Move(Direction::Down, Distance::Line),
                KeyCode::Char('p') => EmacsAction::Move(Direction::Up, Distance::Line),
                KeyCode::Char('a') => EmacsAction::Move(Direction::Backward, Distance::LineEnd),
                KeyCode::Char('e') => EmacsAction::Move(Direction::Forward, Distance::LineEnd),
                KeyCode::Char('d') => EmacsAction::Delete(DeleteRange::Char),
                KeyCode::Char('k') => EmacsAction::Delete(DeleteRange::ToLineEnd),
                KeyCode::Char('g') => { self.multi_key_buffer.clear(); EmacsAction::Cancel }
                KeyCode::Char('c') => EmacsAction::Cancel,
                _ => EmacsAction::None,
            }
        } else {
            match key.code { KeyCode::Char(c) => EmacsAction::Insert(c), _ => EmacsAction::None }
        }
    }
}

impl Default for EmacsKeymap { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyEventKind;
    fn ck(c: char) -> KeyEvent { KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::CONTROL, kind: KeyEventKind::Press, state: crossterm::event::KeyEventState::empty() } }
    fn pk(c: char) -> KeyEvent { KeyEvent { code: KeyCode::Char(c), modifiers: KeyModifiers::empty(), kind: KeyEventKind::Press, state: crossterm::event::KeyEventState::empty() } }
    #[test]
    fn test_emacs_navigation() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(ck('f')), EmacsAction::Move(Direction::Forward, Distance::Char));
        assert_eq!(km.handle_key(ck('b')), EmacsAction::Move(Direction::Backward, Distance::Char));
        assert_eq!(km.handle_key(ck('n')), EmacsAction::Move(Direction::Down, Distance::Line));
        assert_eq!(km.handle_key(ck('p')), EmacsAction::Move(Direction::Up, Distance::Line));
        assert_eq!(km.handle_key(ck('a')), EmacsAction::Move(Direction::Backward, Distance::LineEnd));
        assert_eq!(km.handle_key(ck('e')), EmacsAction::Move(Direction::Forward, Distance::LineEnd));
    }
    #[test]
    fn test_emacs_editing() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(ck('d')), EmacsAction::Delete(DeleteRange::Char));
        assert_eq!(km.handle_key(ck('k')), EmacsAction::Delete(DeleteRange::ToLineEnd));
        assert_eq!(km.handle_key(ck('g')), EmacsAction::Cancel);
        assert_eq!(km.handle_key(pk('x')), EmacsAction::Insert('x'));
        let km2 = EmacsKeymap::new();
        assert!(km2.is_control_pressed(&ck('x')));
        assert!(!km2.is_control_pressed(&pk('x')));
    }
    #[test]
    fn test_emacs_boundary() {
        let mut km = EmacsKeymap::new();
        assert_eq!(km.handle_key(pk('f')), EmacsAction::Insert('f'));
        assert_eq!(km.handle_key(pk('n')), EmacsAction::Insert('n'));
        assert_eq!(km.handle_key(pk('a')), EmacsAction::Insert('a'));
        assert_eq!(km.handle_key(pk('e')), EmacsAction::Insert('e'));
        assert_eq!(km.handle_key(ck('c')), EmacsAction::Cancel);
        assert_eq!(km.handle_key(ck('c')), EmacsAction::Interrupt);
    }
}
