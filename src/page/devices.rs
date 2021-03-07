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

pub struct Devices {
    device_state: ui::ListState
}
impl Devices {
    pub fn new() -> Self {
        Self {
            device_state: Default::default()
        }
    }
}
impl Page for Devices {
    fn name(&self) -> &'static str {
        "Devices"
    }

    fn render(&mut self, frame: &mut Frame<TermionBackend<AlternateScreen<MouseTerminal<RawTerminal<std::io::Stdout>>>>>, area: Rect, devices: &mut DeviceList) {
        self.device_state.set_item_count(devices.len());
        let device_list = List::new(
            devices.iter().map(|(mac, device)| {
                let mut spans = vec![];
                let colour = if device.sent {
                    Color::LightGreen
                } else {
                    Color::LightYellow
                };
                spans.push(Span::styled(mac.to_hex_string(), Style::reset().fg(colour)));
                if let Some(OuiEntry { name_short, name_long, ..}) = &device.manufacturer {
                    spans.push(Span::styled(format!(" | {:8} ", name_short), Style::reset()));
                    if let Some(name_long) = name_long {
                        spans.push(Span::styled(format!("{}", name_long), Style::reset().fg(Color::LightCyan)));
                    }
                }

                ListItem::new(vec![
                    Spans::from(spans)
                ])
            }
            ).collect::<Vec<_>>()
        )
            .block(Block::default().borders(Borders::ALL).title("Devices"))
            .highlight_style(Style::default().bg(Color::Reset).add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");

        frame.render_stateful_widget(device_list, area, &mut self.device_state);
    }

    fn up(&mut self) {
        self.device_state.up()
    }
    fn top(&mut self) {
        self.device_state.top()
    }
    fn down(&mut self) {
        self.device_state.down()
    }
    fn bottom(&mut self) {
        self.device_state.bottom()
    }
    fn left(&mut self) {
        
    }
    fn right(&mut self) {
        
    }
}