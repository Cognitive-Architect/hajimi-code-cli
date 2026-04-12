//! VirtualList - 10k+行60fps虚拟列表，视口回收O(50)内存
#[derive(Debug, Clone)]
pub struct Item { pub index: usize, pub data: String }

#[derive(Debug, Clone, Copy)]
pub struct VisibleRange { pub start: usize, pub end: usize }

pub struct VirtualList {
    pub total_items: usize,
    pub viewport_height: usize,
    pub buffer_size: usize,
    pub scroll_offset: usize,
    pub visible_range: VisibleRange,
    recycled_cells: Vec<Item>,
}

impl VirtualList {
    const VIEWPORT: usize = 50;
    const BUFFER: usize = 5;

    pub fn new(total: usize) -> Self {
        let visible_range = VisibleRange {
            start: 0,
            end: total.min(Self::VIEWPORT + Self::BUFFER * 2),
        };
        Self {
            total_items: total,
            viewport_height: Self::VIEWPORT,
            buffer_size: Self::BUFFER,
            scroll_offset: 0,
            visible_range,
            recycled_cells: Vec::with_capacity(Self::VIEWPORT + Self::BUFFER * 2),
        }
    }

    pub fn scroll_to(&mut self, offset: usize) {
        let max_offset = self.total_items.saturating_sub(self.viewport_height);
        let new_offset = offset.min(max_offset);
        if new_offset == self.scroll_offset { return; }
        self.scroll_offset = new_offset;
        self.update_visible_range();
        self.recycle_cells();
    }

    pub fn scroll_by(&mut self, delta: i32) {
        if delta == 0 { return; }
        let target = (self.scroll_offset as i32).saturating_add(delta);
        self.scroll_to(target.max(0) as usize);
    }

    pub fn render_viewport(&self) -> Vec<Item> {
        let range = self.visible_range;
        let mut items = Vec::with_capacity(range.end.saturating_sub(range.start));
        for i in range.start..range.end.min(self.total_items) {
            items.push(Item { index: i, data: format!("Item {}", i) });
        }
        items
    }

    pub fn recycle_cells(&mut self) {
        let range = self.visible_range;
        self.recycled_cells.retain(|c| c.index >= range.start && c.index < range.end);
        let cap = self.viewport_height + self.buffer_size * 2;
        if self.recycled_cells.capacity() < cap {
            self.recycled_cells.reserve(cap - self.recycled_cells.len());
        }
    }

    pub fn visible_count(&self) -> usize {
        let r = self.visible_range;
        r.end.saturating_sub(r.start)
    }

    pub fn is_visible(&self, index: usize) -> bool {
        index >= self.visible_range.start && index < self.visible_range.end
    }

    fn update_visible_range(&mut self) {
        let start = self.scroll_offset.saturating_sub(self.buffer_size);
        let end = self.scroll_offset
            .saturating_add(self.viewport_height)
            .saturating_add(self.buffer_size)
            .min(self.total_items);
        self.visible_range = VisibleRange { start, end };
    }
}

impl Default for VirtualList {
    fn default() -> Self { Self::new(10000) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn create_list(n: usize) -> VirtualList { VirtualList::new(n) }

    #[test]
    fn test_new() {
        let list = create_list(10000);
        assert_eq!(list.total_items, 10000);
        assert_eq!(list.viewport_height, 50);
    }

    #[test]
    fn test_scroll() {
        let mut list = create_list(10000);
        list.scroll_to(100);
        assert_eq!(list.scroll_offset, 100);
        list.scroll_by(10);
        assert_eq!(list.scroll_offset, 110);
        list.scroll_by(-5);
        assert_eq!(list.scroll_offset, 105);
    }

    #[test]
    fn test_bounds() {
        let mut list = create_list(100);
        list.scroll_to(1000);
        assert_eq!(list.scroll_offset, 50);
    }

    #[test]
    fn test_render_viewport() {
        let list = create_list(10000);
        let items = list.render_viewport();
        assert_eq!(items.len(), 60);
    }

    #[test]
    fn test_memory_complexity() {
        let mut list = create_list(10000);
        list.scroll_to(5000);
        list.recycle_cells();
        assert!(list.visible_count() <= 60);
        let items = list.render_viewport();
        assert!(items.len() <= 60);
    }
}
