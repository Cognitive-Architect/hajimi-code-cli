//! Pane Layout Engine - B-17/02
use crate::layout::Rect;
use crate::keymap_vim::Direction;

/// Split direction for pane operations
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SplitDirection { Horizontal, Vertical }

/// Calculate split layout for a given pane rect
pub fn calculate_split(rect: Rect, dir: SplitDirection, min_size: u16) -> Option<(Rect, Rect)> {
    match dir {
        SplitDirection::Horizontal if rect.width >= min_size * 2 => {
            let nw = rect.width / 2;
            Some((Rect { width: nw, ..rect }, Rect { x: rect.x + nw, y: rect.y, width: rect.width - nw, height: rect.height }))
        }
        SplitDirection::Vertical if rect.height >= min_size * 2 => {
            let nh = rect.height / 2;
            Some((Rect { height: nh, ..rect }, Rect { x: rect.x, y: rect.y + nh, width: rect.width, height: rect.height - nh }))
        }
        _ => None,
    }
}

/// Check if pane at (px, py) is in given direction from center (cx, cy)
pub fn is_in_direction(direction: Direction, cx: u16, cy: u16, px: u16, py: u16) -> bool {
    match direction {
        Direction::Left => px < cx, Direction::Right => px > cx,
        Direction::Up => py < cy, Direction::Down => py > cy, _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_split_horizontal() -> Result<(), &'static str> {
        let r = Rect { x: 0, y: 0, width: 10, height: 5 };
        let (l, r) = calculate_split(r, SplitDirection::Horizontal, 4).ok_or("horizontal split failed")?;
        assert_eq!(l.width, 5); assert_eq!(r.width, 5);
        Ok(())
    }
    #[test] fn test_split_vertical() -> Result<(), &'static str> {
        let r = Rect { x: 0, y: 0, width: 10, height: 8 };
        let (t, b) = calculate_split(r, SplitDirection::Vertical, 4).ok_or("vertical split failed")?;
        assert_eq!(t.height, 4); assert_eq!(b.height, 4);
        Ok(())
    }
    #[test] fn test_direction_check() {
        assert!(is_in_direction(Direction::Right, 5, 5, 10, 5));
        assert!(is_in_direction(Direction::Down, 5, 5, 5, 10));
    }
}
