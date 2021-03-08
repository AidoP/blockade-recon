use std::{thread, sync::mpsc::{self, Receiver}, ops::{Deref, DerefMut}};
use termion::{event::Key, input::{MouseTerminal, TermRead}, raw::{IntoRawMode, RawTerminal}, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
    style::{Style, Modifier, Color},
    text::Spans
};

pub type Backend = TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<std::io::Stdout>>>>;
pub type Terminal = tui::Terminal<Backend>;

pub struct Ui {
    pub input: Input,
    pub terminal: Terminal
}
impl Ui {
    pub fn new() -> Self {
        let backend = TermionBackend::new(
            AlternateScreen::from(
                MouseTerminal::from(
                    std::io::stdout().into_raw_mode().expect("Unable to switch stdout to raw mode")
                )
            )
        );
        let terminal = tui::Terminal::new(backend).expect("Unable to create TUI");
        let input = Input::new();
        Self {
            input,
            terminal
        }
    }
}

pub struct ListState{
    state: tui::widgets::ListState,
    item_count: usize
}
impl ListState {
    pub fn with_item_count(item_count: usize) -> Self {
        let mut state = tui::widgets::ListState::default();
        state.select(Some(0));
        Self {
            state,
            item_count
        }
    }
    pub fn set_item_count(&mut self, item_count: usize) {
        if let Some(selected) = self.state.selected() {
            if selected >= item_count {
                self.state.select(Some(selected.saturating_sub(1)))
            }
        }
        self.item_count = item_count;
    }
    pub fn up(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected <= 0 {
                self.state.select(Some(self.item_count.saturating_sub(1)))
            } else {
                self.state.select(Some(selected.saturating_sub(1)))
            }
        }
    }
    pub fn down(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected >= self.item_count.saturating_sub(1) {
                self.state.select(Some(0))
            } else {
                self.state.select(Some(selected.saturating_add(1)))
            }
        }
    }
    pub fn top(&mut self) {
        self.state.select(Some(0))
    }
    pub fn bottom(&mut self) {
        self.state.select(Some(self.item_count.saturating_sub(1)))
    }
}
impl Default for ListState {
    fn default() -> Self {
        let mut state = tui::widgets::ListState::default();
        state.select(Some(0));
        Self {
            state,
            item_count: 0
        }
    }
}
impl Deref for ListState {
    type Target = tui::widgets::ListState;
    fn deref(&self) -> &Self::Target {
        &self.state
    }
}
impl DerefMut for ListState {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.state
    }
}

pub struct TabState<'a> {
    pub titles: Vec<Spans<'a>>,
    pub index: usize
}
impl<'a> TabState<'a> {
    pub fn new(titles: Vec<Spans<'a>>) -> Self {
        Self {
            titles,
            index: 0
        }
    }
    pub fn select(&mut self, index: usize) {
        self.index = index.clamp(0, self.titles.len() - 1)
    }
    pub fn next(&mut self) {
        if self.index >= self.titles.len() - 1 {
            self.index = 0
        } else {
            self.index += 1
        }
    }
    pub fn previous(&mut self) {
        if self.index <= 0 {
            self.index = self.titles.len() - 1
        } else {
            self.index -= 1
        }
    }
}

pub struct Input {
    pub stdin: Receiver<Key>,
}
impl Input {
    pub fn new() -> Input {
        let (tx, rx) = mpsc::channel();
        thread::spawn(move || {
            let mut keys = std::io::stdin().keys();
            while let Some(key) = keys.next() {
                if let Ok(key) = key {
                    tx.send(key).expect("Input channel unexpectedly closed")
                }
            }
        });
        Self {
            stdin: rx
        }
    }
}

macro_rules! expect {
    ($result:expr => $msg:expr) => {
        match result {
            Ok(t) => t,
            Err(e) => {
                let message = format!("Error @ {}:{}:{}", file!(), line!(), column!())
            }
        }
    };
}

pub fn error(ui: &mut Ui, spans: Spans) {
    let _ = ui.input.stdin.try_iter().count();
    ui.input.stdin.recv();

    ui.terminal.draw(|frame| {
        frame.render_widget(
            Paragraph::new(spans)
                .block(Block::default().borders(Borders::BOTTOM))
                .style(Style::reset()),
            frame.size()
        );
    }).expect("Unable to draw to stdout");
}