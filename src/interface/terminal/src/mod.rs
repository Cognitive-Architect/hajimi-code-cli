//! Ink Framework - B-W11/04
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use ratatui::{
    backend::CrosstermBackend, layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::Style, widgets::{Block, Borders, Clear, Paragraph, Wrap}, Frame, Terminal,
};
use std::io::{self, stdout, Stdout};

pub mod animation;
pub mod config;
pub mod config_utils;
pub mod input_handler;
pub mod keymap_emacs;
pub mod keymap_vim;
pub mod layout;
pub mod pane;
pub mod pane_layout;
pub mod pane_manager;
pub mod pane_utils;
pub mod stream_output;
pub mod theme;
pub mod virtual_list;
pub use animation::{Animation, AnimationEngine, AnimationError, Easing};
pub use config::{load_theme_from_file, save_theme_to_file, watch_theme_file, ConfigError};
pub use config_utils::{atomic_write_file, is_json};
pub use input_handler::{Action, HandlerError, HandlerResult, InputHandler, TypeRacingAction, TypeRacingAdapter};
pub use stream_output::StreamOutput;
pub use keymap_emacs::EmacsKeymap;
pub use keymap_vim::{LineRange, VimAction, VimKeymap, VimMode};
pub use layout::{LayoutEngine, LayoutError};
pub use pane::{Pane};
pub use pane_layout::{SplitDirection, calculate_split, is_in_direction};
pub use pane_manager::{PaneManager, PaneError};
pub use pane_utils::{boundary_check, calc_center, calc_distance, resize_rect, translate_rect};
pub use virtual_list::{Item, VirtualList, VisibleRange};
pub use theme::{InputMode, Theme, ThemeError, ThemeManager};

pub struct InkApp {
    pub theme: Theme, pub mode: InputMode, pub running: bool, pub msg: String, pub show_help: bool,
}

impl InkApp {
    pub fn new() -> Self { Self { theme: Theme::default(), mode: InputMode::Normal, running: true, msg: "Ink Ready - q:quit h:help".into(), show_help: false } }
    pub fn with_theme(mut self, theme: Theme) -> Self { self.theme = theme; self }
    pub fn run(&mut self) -> io::Result<()> {
        let mut term = init_term()?;
        let res = self.run_loop(&mut term);
        restore_term()?; res
    }
    fn run_loop(&mut self, term: &mut Terminal<CrosstermBackend<Stdout>>) -> io::Result<()> {
        while self.running {
            term.draw(|f| self.draw(f))?;
            if event::poll(std::time::Duration::from_millis(16))? {
                if let Event::Key(key) = event::read()? { self.handle(key); }
            }
        }
        Ok(())
    }
    fn handle(&mut self, key: KeyEvent) {
        match self.mode {
            InputMode::Normal => match key.code {
                KeyCode::Char('q') => self.running = false,
                KeyCode::Char('h') => self.show_help = !self.show_help,
                KeyCode::Char('i') => { self.mode = InputMode::Insert; self.msg = "-- INSERT --".into(); }
                KeyCode::Char(':') => { self.mode = InputMode::Command; self.msg = ":".into(); }
                KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => self.running = false,
                _ => {}
            },
            InputMode::Insert | InputMode::Command => if key.code == KeyCode::Esc {
                self.mode = InputMode::Normal; self.msg = "-- NORMAL --".into();
            }
        }
    }
    fn draw(&self, frame: &mut Frame) {
        let layout = Layout::default().direction(Direction::Vertical).constraints([Constraint::Min(0), Constraint::Length(3)]).split(frame.size());
        let body = Paragraph::new(self.msg.clone()).block(Block::default().borders(Borders::ALL)).style(Style::default().fg(self.theme.foreground)).wrap(Wrap { trim: true });
        let status = Paragraph::new(format!(" {:?}", self.mode)).block(Block::default().borders(Borders::ALL)).style(self.mode.style(&self.theme)).alignment(Alignment::Left);
        frame.render_widget(body, layout[0]); frame.render_widget(status, layout[1]);
        if self.show_help { self.draw_help(frame); }
    }
    fn draw_help(&self, frame: &mut Frame) {
        let area = centered_rect(50, 40, frame.size());
        let help = Paragraph::new("q:quit | h:help | i:insert | ::command | Esc:normal").block(Block::default().borders(Borders::ALL).title("Help")).style(Style::default().fg(self.theme.foreground));
        frame.render_widget(Clear, area); frame.render_widget(help, area);
    }
}

fn init_term() -> io::Result<Terminal<CrosstermBackend<Stdout>>> {
    crossterm::terminal::enable_raw_mode()?;
    crossterm::execute!(stdout(), crossterm::terminal::EnterAlternateScreen)?;
    Terminal::new(CrosstermBackend::new(stdout()))
}
fn restore_term() -> io::Result<()> {
    crossterm::terminal::disable_raw_mode()?;
    crossterm::execute!(stdout(), crossterm::terminal::LeaveAlternateScreen)
}
fn centered_rect(pct_x: u16, pct_y: u16, r: Rect) -> Rect {
    let v = Layout::default().direction(Direction::Vertical).constraints([Constraint::Percentage((100 - pct_y) / 2), Constraint::Percentage(pct_y), Constraint::Percentage((100 - pct_y) / 2)]).split(r);
    Layout::default().direction(Direction::Horizontal).constraints([Constraint::Percentage((100 - pct_x) / 2), Constraint::Percentage(pct_x), Constraint::Percentage((100 - pct_x) / 2)]).split(v[1])[1]
}
pub fn run_ink() -> io::Result<()> { InkApp::new().run() }
