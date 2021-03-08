use termion::{input::MouseTerminal, raw::RawTerminal, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::Rect,
    terminal::Frame
};

mod devices;
mod manufacturers;

pub use devices::Devices;
pub use manufacturers::Manufacturers;

use crate::{DeviceList, ui};

pub trait Page {
    fn name(&self) -> &'static str;
    fn render(&mut self, frame: &mut Frame<ui::Backend>, area: Rect, devices: &mut DeviceList);
    fn up(&mut self);
    fn down(&mut self);
    fn top(&mut self);
    fn bottom(&mut self);
    fn left(&mut self);
    fn right(&mut self);
}