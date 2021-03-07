use std::collections::{HashMap, HashSet};
use eui48::MacAddress;
use pcap::{Capture, Device};
use radiotap::Radiotap;
use oui::{OuiDatabase, OuiEntry};
use clap::{Arg, App};
use termion::{event::Key, input::{MouseTerminal, TermRead}, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, layout::Layout, widgets::{Block, Borders, List, ListItem, ListState}, text::{Span, Spans}};
use wifi::Frame;

mod ui;
mod wifi;

fn main() {
    let oui_db = OuiDatabase::new_from_str(include_str!("oui_database")).expect("Failed to parse MAC address lookup database");
    let backend = TermionBackend::new(
        AlternateScreen::from(
            MouseTerminal::from(
                std::io::stdout().into_raw_mode().expect("Unable to switch stdout to raw mode")
            )
        )
    );
    let mut terminal = tui::Terminal::new(backend).expect("Unable to create TUI");
    let mut input = ui::Input::new();

    let args = App::new("Blockade Recon 2")
        .version(env!("CARGO_PKG_VERSION"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .arg(
            Arg::with_name("interface")
            .short("i")
            .long("interface")
            .help("Don't pick a default wireless interface to sniff traffic on")
        )
        .get_matches();
    
    let mut device = if args.is_present("interface") {
        let devices = Device::list().expect("Unable to find devices");
        let mut list = ui::ItemList::new(devices.iter().map(|d| d.name.as_str()), "Select a WiFi Device");

        'select_device: loop {
            for key in input.stdin.try_iter() {
                match key {
                    Key::Esc => return,
                    Key::Up | Key::Char('w') => list.up(),
                    Key::Down | Key::Char('s') => list.down(),
                    Key::PageUp => list.top(),
                    Key::PageDown => list.bottom(),
                    Key::Char('\n') => break 'select_device devices[list.state.selected().unwrap()].clone(),
                    _ => ()
                }
            }
            terminal.draw(|f| {
                f.render_stateful_widget(list.items.clone(), f.size(), &mut list.state)
            }).expect("Unable to create list widget");
            std::thread::sleep(std::time::Duration::from_millis(1))
        }
    } else {
        Device::lookup().expect("Unable to choose a default device")
    };

    let mut capture = Capture::from_device(device).unwrap()
        .promisc(true)
        .rfmon(true)
        .immediate_mode(true)
        .open().unwrap();
    let mut savefile = capture.savefile("capture.pcap").unwrap();

    if capture.get_datalink() != pcap::Linktype::IEEE802_11_RADIOTAP {
        let mut ok = false;
        for datalink in capture.list_datalinks().expect("Unable to determine supported datalink layers") {
            if datalink == pcap::Linktype::IEEE802_11_RADIOTAP {
                ok = true;
                capture.set_datalink(datalink).expect("Unable to set the datalink layer")
            }
        }
        if !ok {
            panic!("The interface does not support the radiotap datalink layer required by this program")
        }
    }

    let mut known_macs = HashSet::new();
    let mut manufacturer_count = HashMap::new();
    'sniff: loop {
        for key in input.stdin.try_iter() {
            match key {
                Key::Char('q') => break 'sniff,
                _ => ()
            }
        }
        let packet = capture.next().unwrap();
        savefile.write(&packet);
        
        let (radiotap, data) = Radiotap::parse(packet.data).unwrap();
        use wifi::Frame::*;
        if let Ok(frame) = wifi::Frame::parse(data) {
            match frame {
                Beacon {
                    source,
                    ..
                } => if known_macs.insert(source) {
                    let name = oui_db.query_by_mac(&source)
                        .expect("Failed to query OUI database")
                        .map(|oui| oui.name_short)
                        .unwrap_or("Unknown".to_string());
                    if let Some(count) = manufacturer_count.get_mut(&name) {
                        *count += 1
                    } else {
                        manufacturer_count.insert(name, 1);
                    }
                },
                _ => ()
            }
        }
    }
    std::mem::drop(terminal);
    println!("Found:\n{:?}", manufacturer_count);
}
