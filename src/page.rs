use termion::{event::Key, input::{MouseTerminal, TermRead}, raw::RawTerminal, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Rect, Constraint, Direction, Layout},
    widgets::{BarChart, Block, Borders, List, ListItem},
    style::{Style, Modifier, Color},
    text::{Span, Spans},
    terminal::Frame
};

mod devices;
mod manufacturers;

pub use devices::Devices;
pub use manufacturers::Manufacturers;

use crate::DeviceList;

pub trait Page {
    fn name(&self) -> &'static str;
    fn render(&mut self, frame: &mut Frame<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<std::io::Stdout>>>>>, area: Rect, devices: &mut DeviceList) {

    }
    fn up(&mut self);
    fn down(&mut self);
    fn top(&mut self);
    fn bottom(&mut self);
    fn left(&mut self);
    fn right(&mut self);
}