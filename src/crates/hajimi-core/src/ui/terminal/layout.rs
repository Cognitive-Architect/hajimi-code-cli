//! Flex-like Layout Engine - T14-4

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rect { pub x: u16, pub y: u16, pub width: u16, pub height: u16 }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction { Horizontal, Vertical }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Constraint { Percentage(u16), Length(u16), Min(u16), Max(u16) }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Alignment { Left, Center, Right }

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LayoutError { InvalidConstraint(&'static str) }

#[derive(Debug, Clone)]
pub struct LayoutEngine { direction: Direction, constraints: Vec<Constraint>, margin: u16, alignment: Alignment }

impl LayoutEngine {
    pub fn new() -> Self { Self { direction: Direction::Vertical, constraints: Vec::new(), margin: 0, alignment: Alignment::Left } }
    pub fn direction(mut self, d: Direction) -> Self { self.direction = d; self }
    pub fn constraints(mut self, c: Vec<Constraint>) -> Self { self.constraints = c; self }
    pub fn margin(mut self, m: u16) -> Self { self.margin = m; self }
    pub fn alignment(mut self, a: Alignment) -> Self { self.alignment = a; self }

    pub fn calculate_layout(&self, parent: Rect) -> Result<Vec<Rect>, LayoutError> {
        if parent.width == 0 || parent.height == 0 { return Ok(Vec::new()); }
        let (ix, iy) = (parent.x.saturating_add(self.margin), parent.y.saturating_add(self.margin));
        let (iw, ih) = (parent.width.saturating_sub(self.margin * 2), parent.height.saturating_sub(self.margin * 2));
        if iw == 0 || ih == 0 { return Ok(Vec::new()); }

        let total = if self.direction == Direction::Horizontal { iw } else { ih };
        let mut sizes = Vec::with_capacity(self.constraints.len());
        let mut pct_sum: u16 = 0;

        for c in &self.constraints {
            match c {
                Constraint::Percentage(p) => { if *p > 100 { return Err(LayoutError::InvalidConstraint("Percentage > 100")); } pct_sum = pct_sum.saturating_add(*p); }
                Constraint::Length(l) | Constraint::Min(l) | Constraint::Max(l) => { if *l == 0 { return Err(LayoutError::InvalidConstraint("Zero constraint")); } }
            }
        }

        let scale = if pct_sum > 100 { 100.0 / pct_sum as f32 } else { 1.0 };
        let mut used: u16 = 0;

        for c in &self.constraints {
            let s = match c {
                Constraint::Percentage(p) => ((total as u32 * ((*p as f32 * scale) as u16) as u32 / 100).max(1)) as u16,
                Constraint::Length(l) | Constraint::Min(l) | Constraint::Max(l) => (*l).min(total),
            };
            sizes.push(s);
            used = used.saturating_add(s);
        }

        if used < total {
            let rem = total - used;
            for (i, c) in self.constraints.iter().enumerate() {
                if let Constraint::Min(m) = c { sizes[i] += rem.min(*m - sizes[i]); }
            }
        }

        let mut rects = Vec::with_capacity(self.constraints.len());
        let mut pos = 0u16;
        for s in sizes {
            rects.push(if self.direction == Direction::Horizontal {
                Rect { x: ix + pos, y: iy, width: s, height: ih }
            } else {
                Rect { x: ix, y: iy + pos, width: iw, height: s }
            });
            pos = pos.saturating_add(s);
        }
        Ok(rects)
    }
}

impl Default for LayoutEngine { fn default() -> Self { Self::new() } }

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn test_basic() -> Result<(), LayoutError> {
        let e = LayoutEngine::new().direction(Direction::Vertical).constraints(vec![Constraint::Percentage(30), Constraint::Min(10)]);
        assert_eq!(e.calculate_layout(Rect { x: 0, y: 0, width: 100, height: 100 })?.len(), 2);
        Ok(())
    }
    #[test] fn test_zero() -> Result<(), LayoutError> {
        assert!(LayoutEngine::new().calculate_layout(Rect { x: 0, y: 0, width: 0, height: 100 })?.is_empty());
        Ok(())
    }
    #[test] fn test_invalid() {
        assert!(LayoutEngine::new().constraints(vec![Constraint::Percentage(150)]).calculate_layout(Rect { x: 0, y: 0, width: 100, height: 100 }).is_err());
    }
}
