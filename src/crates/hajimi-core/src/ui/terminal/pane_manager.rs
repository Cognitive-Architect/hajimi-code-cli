//! PaneManager - B-17/02: Terminal Split System
//! Supports Horizontal/Vertical splits, C-w hjkl navigation, max 8 panes
use crate::ui::terminal::layout::Rect;
use crate::ui::terminal::pane::Pane;
use crate::ui::terminal::keymap_vim::Direction;
use crate::ui::terminal::{SplitDirection, calculate_split, is_in_direction};

/// Pane manager errors
#[derive(Debug, Clone, PartialEq)]
pub enum PaneError {
    PaneNotFound(u8), MaxPanesReached, CannotCloseLastPane, InvalidSplit,
}

impl std::fmt::Display for PaneError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PaneError::PaneNotFound(id) => write!(f, "Pane {} not found", id),
            PaneError::MaxPanesReached => write!(f, "Maximum number of panes (8) reached"),
            PaneError::CannotCloseLastPane => write!(f, "Cannot close the last pane"),
            PaneError::InvalidSplit => write!(f, "Invalid split operation"),
        }
    }
}
impl std::error::Error for PaneError {}

/// Manages multiple terminal panes with split functionality
#[derive(Debug, Clone)]
pub struct PaneManager {
    pub panes: Vec<Pane>, pub active_id: u8, pub max_panes: u8,
}

impl PaneManager {
    pub const MAX_PANES: u8 = 8;

    pub fn new() -> Self {
        let initial_rect = Rect { x: 0, y: 0, width: 160, height: 48 };
        Self { panes: vec![Pane::new(0, initial_rect)], active_id: 0, max_panes: Self::MAX_PANES }
    }

    fn get_pane_mut(&mut self, id: u8) -> Result<&mut Pane, PaneError> {
        self.panes.iter_mut().find(|p| p.id == id).ok_or(PaneError::PaneNotFound(id))
    }
    fn get_pane(&self, id: u8) -> Result<&Pane, PaneError> {
        self.panes.iter().find(|p| p.id == id).ok_or(PaneError::PaneNotFound(id))
    }
    fn next_id(&self) -> u8 { self.panes.iter().map(|p| p.id).max().unwrap_or(0).saturating_add(1) }

    /// Generic split (used by both horizontal and vertical)
    fn split(&mut self, pane_id: u8, dir: SplitDirection) -> Result<(), PaneError> {
        if self.panes.len() >= self.max_panes as usize { return Err(PaneError::MaxPanesReached); }
        let pane = self.get_pane(pane_id)?;
        let (orig_rect, new_rect) = calculate_split(pane.rect, dir, 4).ok_or(PaneError::InvalidSplit)?;
        let new_id = self.next_id();
        let mut new_pane = Pane::new(new_id, new_rect);
        new_pane.set_active(true);
        if let Ok(p) = self.get_pane_mut(pane_id) { p.set_rect(orig_rect); p.set_active(false); }
        self.panes.push(new_pane);
        self.active_id = new_id; Ok(())
    }

    /// Split pane horizontally (C-w v) - creates left/right panes
    pub fn split_horizontal(&mut self, pane_id: u8) -> Result<(), PaneError> {
        self.split(pane_id, SplitDirection::Horizontal)
    }

    /// Split pane vertically (C-w s) - creates top/bottom panes
    pub fn split_vertical(&mut self, pane_id: u8) -> Result<(), PaneError> {
        self.split(pane_id, SplitDirection::Vertical)
    }

    /// Switch to pane in given direction (C-w hjkl)
    pub fn switch_pane(&mut self, direction: Direction) -> Result<(), PaneError> {
        let current = self.get_pane(self.active_id)?;
        let (cx, cy) = current.center();
        let mut best: Option<(u8, u32)> = None;
        for pane in &self.panes {
            if pane.id == self.active_id { continue; }
            let (px, py) = pane.center();
            if is_in_direction(direction, cx, cy, px, py) {
                let dist = ((px as i32 - cx as i32).pow(2) + (py as i32 - cy as i32).pow(2)) as u32;
                if best.map_or(true, |(_, d)| dist < d) { best = Some((pane.id, dist)); }
            }
        }
        match best {
            Some((new_id, _)) => {
                if let Ok(p) = self.get_pane_mut(self.active_id) { p.set_active(false); }
                if let Ok(p) = self.get_pane_mut(new_id) { p.set_active(true); }
                self.active_id = new_id; Ok(())
            }
            None => Err(PaneError::PaneNotFound(self.active_id))
        }
    }

    /// Close a pane (C-w c) - prevents closing last pane
    pub fn close_pane(&mut self, pane_id: u8) -> Result<(), PaneError> {
        if self.panes.len() <= 1 { return Err(PaneError::CannotCloseLastPane); }
        let idx = self.panes.iter().position(|p| p.id == pane_id).ok_or(PaneError::PaneNotFound(pane_id))?;
        self.panes.remove(idx);
        if self.active_id == pane_id {
            let new_active = self.panes.iter().map(|p| p.id).min().unwrap_or(0);
            self.active_id = new_active;
            if let Ok(p) = self.get_pane_mut(new_active) { p.set_active(true); }
        }
        Ok(())
    }

    pub fn get_active_pane(&self) -> Option<&Pane> { self.panes.iter().find(|p| p.id == self.active_id).or_else(|| self.panes.first()) }
    pub fn pane_count(&self) -> usize { self.panes.len() }
    pub fn is_full(&self) -> bool { self.panes.len() >= self.max_panes as usize }
}

impl Default for PaneManager { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_new_manager() {
        let pm = PaneManager::new();
        assert_eq!(pm.pane_count(), 1); assert_eq!(pm.active_id, 0); assert_eq!(pm.max_panes, 8);
    }
    #[test] fn test_split_horizontal() {
        let mut pm = PaneManager::new();
        assert!(pm.split_horizontal(0).is_ok()); assert_eq!(pm.pane_count(), 2); assert_eq!(pm.active_id, 1);
    }
    #[test] fn test_split_vertical() {
        let mut pm = PaneManager::new();
        assert!(pm.split_vertical(0).is_ok()); assert_eq!(pm.pane_count(), 2);
    }
    #[test] fn test_max_panes_limit() {
        let mut pm = PaneManager::new();
        for i in 0..7 {
            let active = pm.active_id;
            if i % 2 == 0 { assert!(pm.split_horizontal(active).is_ok()); }
            else { assert!(pm.split_vertical(active).is_ok()); }
        }
        assert_eq!(pm.pane_count(), 8);
        assert!(pm.split_horizontal(pm.active_id) == Err(PaneError::MaxPanesReached));
    }
    #[test] fn test_cannot_close_last_pane() {
        let mut pm = PaneManager::new();
        assert_eq!(pm.close_pane(0), Err(PaneError::CannotCloseLastPane));
    }
    #[test] fn test_close_pane_switch() -> Result<(), PaneError> {
        let mut pm = PaneManager::new();
        pm.split_horizontal(0)?;
        let old_active = pm.active_id;
        assert!(pm.close_pane(old_active).is_ok()); assert_eq!(pm.pane_count(), 1);
        Ok(())
    }
    #[test] fn test_switch_pane_hjkl() -> Result<(), PaneError> {
        let mut pm = PaneManager::new();
        pm.split_horizontal(0)?; pm.active_id = 0;
        assert!(pm.switch_pane(Direction::Right).is_ok()); assert_eq!(pm.active_id, 1);
        assert!(pm.switch_pane(Direction::Left).is_ok()); assert_eq!(pm.active_id, 0);
        Ok(())
    }
    #[test] fn test_get_active_pane() -> Result<(), PaneError> {
        let mut pm = PaneManager::new();
        let pane = pm.get_active_pane().ok_or(PaneError::PaneNotFound(0))?; assert_eq!(pane.id, 0);
        pm.split_vertical(0)?;
        let pane = pm.get_active_pane().ok_or(PaneError::PaneNotFound(1))?; assert_eq!(pane.id, 1);
        Ok(())
    }
}
