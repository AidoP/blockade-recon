use std::{io::Write, thread::{self, JoinHandle}, sync::mpsc::{self, Sender, Receiver}};
use termion::{event::Key, input::{MouseTerminal, TermRead}, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::Layout,
    widgets::{Block, Borders, List, ListItem, ListState},
    style::{Style, Modifier, Color},
    text::{Span, Spans}
};

pub struct ItemList<'a> {
    pub item_count: usize,
    pub items: List<'a>,
    pub state: ListState
}
impl<'a> ItemList<'a> {
    pub fn new<I: Iterator<Item=&'a str>>(items: I, title: &'a str) -> Self {
        let items: Vec<_> = items.into_iter().map(|i| ListItem::new(vec![Spans::from(i)])).collect();
        let mut state = ListState::default();
        state.select(Some(0));
        Self {
            item_count: items.len(),
            items: List::new(items)
                .block(Block::default().borders(Borders::ALL).title(title))
                .highlight_style(Style::default().bg(Color::Reset).add_modifier(Modifier::REVERSED))
                .highlight_symbol("> "),
            state
        }
    }
    pub fn up(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected <= 0 {
                self.state.select(Some(self.item_count - 1))
            } else {
                self.state.select(Some(selected - 1))
            }
        }
    }
    pub fn down(&mut self) {
        if let Some(selected) = self.state.selected() {
            if selected >= self.item_count - 1 {
                self.state.select(Some(0))
            } else {
                self.state.select(Some(selected + 1))
            }
        }
    }
    pub fn top(&mut self) {
        self.state.select(Some(0))
    }
    pub fn bottom(&mut self) {
        self.state.select(Some(self.item_count - 1))
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