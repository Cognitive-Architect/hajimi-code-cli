//! Pane Structure - B-16/04: Terminal Pane (Compressed B-17/01)

use crate::ui::terminal::layout::Rect;
use crate::ui::terminal::pane_utils::{calc_center, calc_distance, resize_rect, translate_rect, boundary_check};

/// Single terminal pane representation
#[derive(Debug, Clone, PartialEq)]
pub struct Pane {
    pub id: u8,
    pub rect: Rect,
    pub buffer: String,
    pub is_active: bool,
}

impl Default for Pane {
    fn default() -> Self {
        Self { id: 0, rect: Rect { x: 0, y: 0, width: 80, height: 24 }, buffer: String::with_capacity(1024), is_active: false }
    }
}

impl Pane {
    pub fn new(id: u8, rect: Rect) -> Self { Self { id, rect, ..Default::default() } }
    pub fn set_active(&mut self, active: bool) { self.is_active = active; }
    pub fn append(&mut self, content: &str) { self.buffer.push_str(content); }
    pub fn clear(&mut self) { self.buffer.clear(); }
    pub fn set_rect(&mut self, rect: Rect) { self.rect = rect; }
    pub fn len(&self) -> usize { self.buffer.len() }
    pub fn is_empty(&self) -> bool { self.buffer.is_empty() }
    pub fn content(&self) -> &str { &self.buffer }
    pub fn contains(&self, x: u16, y: u16) -> bool { boundary_check(&self.rect, x, y) }
    pub fn center(&self) -> (u16, u16) { calc_center(&self.rect) }
    pub fn distance_to(&self, other: &Pane) -> u32 { calc_distance(&self.rect, &other.rect) }
    pub fn area(&self) -> u32 { self.rect.width as u32 * self.rect.height as u32 }
    pub fn resize(&mut self, dw: i16, dh: i16) { resize_rect(&mut self.rect, dw, dh); }
    pub fn translate(&mut self, dx: i16, dy: i16) { translate_rect(&mut self.rect, dx, dy); }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pane_new() {
        let rect = Rect { x: 0, y: 0, width: 80, height: 24 };
        let pane = Pane::new(1, rect);
        assert_eq!(pane.id, 1);
        assert_eq!(pane.rect, rect);
        assert!(pane.buffer.is_empty());
        assert!(!pane.is_active);
    }

    #[test]
    fn test_pane_active() {
        let mut pane = Pane::new(1, Rect { x: 0, y: 0, width: 80, height: 24 });
        pane.set_active(true);
        assert!(pane.is_active);
        pane.set_active(false);
        assert!(!pane.is_active);
    }

    #[test]
    fn test_pane_buffer() {
        let mut pane = Pane::new(1, Rect { x: 0, y: 0, width: 80, height: 24 });
        pane.append("hello");
        assert_eq!(pane.content(), "hello");
        pane.append(" world");
        assert_eq!(pane.content(), "hello world");
        pane.clear();
        assert!(pane.is_empty());
    }

    #[test]
    fn test_pane_contains() {
        let pane = Pane::new(1, Rect { x: 10, y: 10, width: 20, height: 10 });
        assert!(pane.contains(15, 15));
        assert!(!pane.contains(5, 5));
        assert!(!pane.contains(35, 25));
    }

    #[test]
    fn test_pane_center_distance() {
        let pane1 = Pane::new(1, Rect { x: 0, y: 0, width: 10, height: 10 });
        let pane2 = Pane::new(2, Rect { x: 10, y: 0, width: 10, height: 10 });
        assert_eq!(pane1.center(), (5, 5));
        assert_eq!(pane2.center(), (15, 5));
        assert_eq!(pane1.distance_to(&pane2), 100);
    }
}
