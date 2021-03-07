use oui::{OuiDatabase, OuiEntry};
use termion::{event::Key, input::{MouseTerminal, TermRead}, raw::RawTerminal, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Rect, Constraint, Direction, Layout},
    widgets::{BarChart, Block, Borders, List, ListItem, Tabs},
    style::{Style, Modifier, Color},
    text::{Span, Spans},
    terminal::Frame
};

use super::Page;
use crate::{DeviceList, ui};

pub struct Manufacturers {

}
impl Manufacturers {
    pub fn new() -> Self {
        Self {
            
        }
    }
}
impl Page for Manufacturers {
    fn name(&self) -> &'static str {
        "Manufacturers"
    }

    fn render(&mut self, frame: &mut Frame<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<std::io::Stdout>>>>>, area: Rect, devices: &mut DeviceList) {
        let bar_data = devices.bar_data();
        let barchart = BarChart::default()
            .block(Block::default().borders(Borders::ALL).title("Manufacturers"))
            .data(&bar_data)
            .bar_width(8)
            .bar_gap(1)
            .bar_style(Style::reset().fg(Color::Blue))
            .value_style(Style::reset().fg(Color::Blue).add_modifier(Modifier::REVERSED));

        frame.render_widget(barchart, area);
    }

    fn up(&mut self) {
        
    }
    fn top(&mut self) {
        
    }
    fn down(&mut self) {
        
    }
    fn bottom(&mut self) {
        
    }
    fn left(&mut self) {
        
    }
    fn right(&mut self) {
        
    }
}