use std::{collections::HashMap, ops::{Deref, DerefMut}};
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
    println!("Parsing Manufacturer Names");
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

        let mut draw = |list: &mut ui::ItemList| terminal.draw(|f| {
            f.render_stateful_widget(list.items.clone(), f.size(), &mut list.state)
        }).expect("Unable to create list widget");
        draw(&mut list);
        'select_device: loop {
            for key in input.stdin.iter() {
                match key {
                    Key::Esc => return,
                    Key::Up | Key::Char('w') => { list.up(); draw(&mut list) }
                    Key::Down | Key::Char('s') => { list.down(); draw(&mut list) }
                    Key::PageUp => { list.top(); draw(&mut list) }
                    Key::PageDown => { list.bottom(); draw(&mut list) }
                    Key::Char('\n') => break 'select_device devices[list.state.selected().unwrap()].clone(),
                    _ => ()
                }
            }
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

    let mut devices = DeviceList::default();
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
                    destination,
                    ssid,
                    ..
                } => {
                    devices.get_or_default(source, &oui_db)
                        .sent(true)
                        .beacon(ssid);
                    devices.get_or_default(destination, &oui_db)
                        .sent(false);
                }
                Ack {
                    receiver
                } => {
                    devices.get_or_default(receiver, &oui_db)
                        .sent(false);
                }
                _ => ()
            }
        }
    }
    std::mem::drop(terminal);
    println!("Found:\n{:?}", devices);
}

/// A device tracked by blockade
/// Tracks metadata relating to the device
#[derive(Debug)]
pub struct KnownDevice {
    manufacturer: Option<OuiEntry>,
    /// The SSID of the beacon, or None if not a beacon
    beacon: Option<String>,
    /// False if this device is known only by reference from another device, ie. has not sent any data
    sent: bool,
}
impl KnownDevice {
    fn new(address: MacAddress, oui_db: &OuiDatabase) -> Self {
        Self {
            manufacturer: oui_db.query_by_mac(&address).unwrap(/* Library should never be able to return an error */),
            beacon: None,
            sent: false
        }
    }
    fn sent(&mut self, sent: bool) -> &mut Self {
        self.sent = sent;
        self
    }
    fn beacon(&mut self, ssid: String) -> &mut Self {
        self.beacon = Some(ssid);
        self
    }
}

#[derive(Debug, Default)]
struct DeviceList(HashMap<MacAddress, KnownDevice>);
impl DeviceList {
    fn get_or_default(&mut self, address: MacAddress, oui_db: &OuiDatabase) -> &mut KnownDevice {
        if self.contains_key(&address) {
            self.get_mut(&address).unwrap()
        } else {
            self.insert(address, KnownDevice::new(address, oui_db));
            self.get_mut(&address).unwrap()
        }
    }
}
impl Deref for DeviceList {
    type Target = HashMap<MacAddress, KnownDevice>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
impl DerefMut for DeviceList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}