//! Pane utilities - B-17/01: Extracted from pane.rs

use crate::ui::terminal::layout::Rect;

/// Calculate center coordinates of a rect
pub fn calc_center(rect: &Rect) -> (u16, u16) {
    (rect.x + rect.width / 2, rect.y + rect.height / 2)
}

/// Calculate distance squared between two rect centers
pub fn calc_distance(r1: &Rect, r2: &Rect) -> u32 {
    let (x1, y1) = calc_center(r1);
    let (x2, y2) = calc_center(r2);
    let dx = x1 as i32 - x2 as i32;
    let dy = y1 as i32 - y2 as i32;
    (dx * dx + dy * dy) as u32
}

/// Resize rect by delta, ensuring minimum of 1
pub fn resize_rect(rect: &mut Rect, dw: i16, dh: i16) {
    rect.width = (rect.width as i16 + dw).max(1) as u16;
    rect.height = (rect.height as i16 + dh).max(1) as u16;
}

/// Translate rect by delta, ensuring minimum of 0
pub fn translate_rect(rect: &mut Rect, dx: i16, dy: i16) {
    rect.x = (rect.x as i16 + dx).max(0) as u16;
    rect.y = (rect.y as i16 + dy).max(0) as u16;
}

/// Check if point is inside rect boundaries
pub fn boundary_check(rect: &Rect, x: u16, y: u16) -> bool {
    x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
}
