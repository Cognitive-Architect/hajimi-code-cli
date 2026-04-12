use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};

/// Vim模式枚举，定义了终端支持的三种Vim操作模式
/// - Normal: 普通模式，用于导航和执行命令
/// - Insert: 插入模式，用于文本输入
/// - Visual: 可视模式，用于文本选择和操作
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimMode { Normal, Insert, Visual }

/// 方向枚举，定义了光标移动的所有方向
/// - Left/Right/Up/Down: 基本的hjkl导航方向
/// - DocumentStart/DocumentEnd: 文档首尾跳转（gg/GG命令）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction { Left, Right, Up, Down, DocumentStart, DocumentEnd }

/// 行范围枚举，用于指定删除等操作的目标范围
/// - Current: 当前行（dd命令）
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineRange { Current }

/// Vim动作枚举，表示按键处理后的输出动作
/// 用于将按键输入转换为编辑器可执行的操作指令
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum VimAction { Move(Direction), Delete(LineRange), Insert, ChangeMode(VimMode), None }

/// 边界检查结果枚举，用于光标位置验证时的返回值
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BoundaryResult { Valid, OutOfBounds, AtTop, AtBottom }

const DEBOUNCE: Duration = Duration::from_millis(200);
const TIMEOUT: Duration = Duration::from_millis(500);

/// Vim键位映射核心结构体，维护当前模式状态、按键序列缓冲区及时间戳
pub struct VimKeymap { mode: VimMode, seq: Vec<KeyEvent>, last_switch: Instant, last_key: Instant }

impl VimKeymap {
    pub fn new() -> Self { let now = Instant::now(); Self { mode: VimMode::Normal, seq: Vec::new(), last_switch: now, last_key: now } }
    pub fn mode(&self) -> VimMode { self.mode }
    pub fn status_indicator(&self) -> &'static str { match self.mode { VimMode::Normal => "", VimMode::Insert => "--INSERT--", VimMode::Visual => "--VISUAL--" } }

    /// 检查边界条件，验证给定位置是否在有效范围内
    /// current: 当前行/列索引, max: 最大允许值（不包含）
    pub fn check_boundary(current: usize, max: usize) -> BoundaryResult {
        if current >= max { BoundaryResult::OutOfBounds }
        else if current == 0 { BoundaryResult::AtTop }
        else if current + 1 >= max { BoundaryResult::AtBottom }
        else { BoundaryResult::Valid }
    }

    /// 验证位置有效性，确保光标不会越界，返回修正后的安全位置
    pub fn validate_position(pos: usize, limit: usize) -> usize {
        if pos >= limit { if limit == 0 { 0 } else { limit.saturating_sub(1) } } else { pos }
    }

    /// 处理带换行的光标移动，当光标到达行边界时自动处理换行逻辑
    pub fn move_cursor_with_wrap(current: usize, direction: Direction, line_length: usize) -> Option<usize> {
        match direction {
            Direction::Left if current > 0 => Some(current.saturating_sub(1)),
            Direction::Right if current + 1 < line_length => Some(current.saturating_add(1)),
            Direction::Up | Direction::Down => Some(current), _ => None,
        }
    }

    /// 处理边界边缘情况，当光标位于文档边界时提供特殊处理逻辑
    pub fn handle_edge_case(direction: Direction, boundary: BoundaryResult) -> bool {
        matches!((direction, boundary), (Direction::Up, BoundaryResult::AtTop) | (Direction::Down, BoundaryResult::AtBottom) | (Direction::Left, BoundaryResult::AtTop) | (Direction::DocumentStart, BoundaryResult::AtTop) | (Direction::DocumentEnd, BoundaryResult::AtBottom))
    }

    /// 处理按键输入的主入口函数，根据当前模式将按键分发给对应的处理函数
    pub fn handle_key(&mut self, key: KeyEvent) -> VimAction {
        let now = Instant::now();
        if now.duration_since(self.last_key) > TIMEOUT { self.seq.clear(); }
        self.last_key = now;
        match self.mode { VimMode::Normal => self.handle_normal(key, now), VimMode::Insert => self.handle_insert(key, now), VimMode::Visual => self.handle_visual(key, now) }
    }

    /// 处理Normal模式下的按键输入：包括hjkl导航、gg/GG跳转、dd删除、模式切换等
    fn handle_normal(&mut self, key: KeyEvent, now: Instant) -> VimAction {
        if !self.seq.is_empty() { return self.handle_seq(key); }
        match key.code {
            KeyCode::Char('h') => VimAction::Move(Direction::Left), KeyCode::Char('j') => VimAction::Move(Direction::Down),
            KeyCode::Char('k') => VimAction::Move(Direction::Up), KeyCode::Char('l') => VimAction::Move(Direction::Right),
            // 'G'命令：跳转到文档末尾。大写G表示文档最后一行，与gg（文档首行）对应
            // 实际编辑器中需要验证文档非空，避免越界访问
            KeyCode::Char('G') => VimAction::Move(Direction::DocumentEnd),
            KeyCode::Char('g') | KeyCode::Char('d') => { self.seq.push(key); VimAction::None }
            KeyCode::Char('i') | KeyCode::Char('I') => { self.change(VimMode::Insert, now); VimAction::Insert }
            KeyCode::Char('v') => { self.change(VimMode::Visual, now); VimAction::ChangeMode(VimMode::Visual) }
            _ => VimAction::None,
        }
    }

    /// 处理组合键序列（如gg、dd等双字符命令）
    fn handle_seq(&mut self, key: KeyEvent) -> VimAction {
        let action = match self.seq.first() {
            // gg命令：跳转到文档首行，在实际编辑器中需要验证文档非空
            Some(&KeyEvent { code: KeyCode::Char('g'), .. }) if key.code == KeyCode::Char('g') => VimAction::Move(Direction::DocumentStart),
            // dd命令：删除当前行
            // DEBT-LINES-B16-01: 需要在此处集成撤销管理器，TODO: 调用 undo_manager.mark_delete() 记录操作
            Some(&KeyEvent { code: KeyCode::Char('d'), .. }) if key.code == KeyCode::Char('d') => VimAction::Delete(LineRange::Current),
            _ => VimAction::None,
        }; self.seq.clear(); action
    }

    fn handle_insert(&mut self, key: KeyEvent, now: Instant) -> VimAction {
        if key.code == KeyCode::Esc || (key.code == KeyCode::Char('[') && key.modifiers == KeyModifiers::CONTROL) {
            self.change(VimMode::Normal, now); return VimAction::ChangeMode(VimMode::Normal);
        } VimAction::None
    }

    fn handle_visual(&mut self, key: KeyEvent, now: Instant) -> VimAction {
        if key.code == KeyCode::Esc || (key.code == KeyCode::Char('[') && key.modifiers == KeyModifiers::CONTROL) {
            self.change(VimMode::Normal, now); return VimAction::ChangeMode(VimMode::Normal);
        }
        match key.code { KeyCode::Char('h') => VimAction::Move(Direction::Left), KeyCode::Char('j') => VimAction::Move(Direction::Down), KeyCode::Char('k') => VimAction::Move(Direction::Up), KeyCode::Char('l') => VimAction::Move(Direction::Right), _ => VimAction::None }
    }

    fn change(&mut self, mode: VimMode, now: Instant) { if now.duration_since(self.last_switch) >= DEBOUNCE { self.mode = mode; self.last_switch = now; self.seq.clear(); } }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn key(c: char) -> KeyEvent { KeyEvent::from(KeyCode::Char(c)) }
    #[test] fn test_vim_navigation() {
        let mut v = VimKeymap::new();
        assert!(matches!(v.handle_key(key('h')), VimAction::Move(Direction::Left)));
        assert!(matches!(v.handle_key(key('j')), VimAction::Move(Direction::Down)));
        assert!(matches!(v.handle_key(key('k')), VimAction::Move(Direction::Up)));
        assert!(matches!(v.handle_key(key('l')), VimAction::Move(Direction::Right)));
        assert!(matches!(v.handle_key(key('G')), VimAction::Move(Direction::DocumentEnd)));
    }
    #[test] fn test_vim_mode_switch() {
        let mut v = VimKeymap::new();
        assert_eq!(v.mode(), VimMode::Normal);
        v.handle_key(key('i')); assert_eq!(v.mode(), VimMode::Insert); assert_eq!(v.status_indicator(), "--INSERT--");
        v.handle_key(KeyEvent::from(KeyCode::Esc)); assert_eq!(v.mode(), VimMode::Normal);
        v.handle_key(key('v')); assert_eq!(v.mode(), VimMode::Visual); assert_eq!(v.status_indicator(), "--VISUAL--");
    }
    #[test] fn test_vim_composite() {
        let mut v = VimKeymap::new();
        v.handle_key(key('g')); assert!(matches!(v.handle_key(key('g')), VimAction::Move(Direction::DocumentStart)));
        v.handle_key(key('d')); assert!(matches!(v.handle_key(key('d')), VimAction::Delete(LineRange::Current)));
        v.handle_key(key('g')); assert!(matches!(v.handle_key(key('G')), VimAction::None));
    }
    #[test] fn test_boundary_check() {
        assert_eq!(VimKeymap::check_boundary(0, 10), BoundaryResult::AtTop);
        assert_eq!(VimKeymap::check_boundary(5, 10), BoundaryResult::Valid);
        assert_eq!(VimKeymap::check_boundary(9, 10), BoundaryResult::AtBottom);
        assert_eq!(VimKeymap::check_boundary(10, 10), BoundaryResult::OutOfBounds);
        assert_eq!(VimKeymap::validate_position(5, 10), 5); assert_eq!(VimKeymap::validate_position(15, 10), 9);
        assert_eq!(VimKeymap::validate_position(0, 0), 0);
    }
}
