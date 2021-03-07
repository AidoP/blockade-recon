use oui::{OuiDatabase, OuiEntry};
use termion::{event::Key, input::{MouseTerminal, TermRead}, raw::RawTerminal, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Rect, Constraint, Direction, Layout},
    widgets::{Paragraph, Block, Borders, List, ListItem, Tabs},
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
        fn format_string(value: &str) -> Span {
            Span::styled(format!("{:?}", value), Style::reset().fg(Color::LightBlue))
        }
        fn format_header(title: &str) -> Spans {
            Spans::from(vec![Span::styled(title, Style::default().fg(Color::Blue).add_modifier(Modifier::BOLD))])
        }
        
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
        
        if let Some((device_mac, device)) = devices.iter().nth(self.device_state.selected().unwrap()) {
            let areas = Layout::default()
                .direction(Direction::Horizontal)
                .margin(0)
                .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
                .split(area);
            let mut device_info = vec![];

            if let Some(ssid) = &device.beacon {
                device_info.push(format_header("Beacon"));
                device_info.push(Spans::from(vec![
                    Span::raw("  SSID: "),
                    format_string(ssid)
                ]));
                
            }
            if let Some(manufacturer) = &device.manufacturer {
                device_info.push(format_header("Manufacturer"));
                device_info.push(Spans::from(vec![
                    Span::raw("  Short Name: "),
                    format_string(&manufacturer.name_short)
                ]));
                if let Some(name_long) = &manufacturer.name_long {
                    device_info.push(Spans::from(vec![
                        Span::raw("  Long Name: "),
                        format_string(name_long)
                    ]))
                }
                if let Some(comment) = &manufacturer.comment {
                    device_info.push(Spans::from(vec![
                        Span::raw("  Comment: "),
                        format_string(comment)
                    ]))
                }
            }

            let device_info = Paragraph::new(device_info)
                .block(Block::default().borders(Borders::ALL).title(device_mac.to_hex_string()));
            frame.render_stateful_widget(device_list, areas[0], &mut self.device_state);
            frame.render_widget(device_info, areas[1])
        } else {
            frame.render_stateful_widget(device_list, area, &mut self.device_state);
        }
        
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