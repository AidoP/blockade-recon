use std::{collections::{HashMap, HashSet}, ops::{Deref, DerefMut}, fs};
use eui48::MacAddress;
use pcap::{Capture, Device};
use radiotap::Radiotap;
use oui::{OuiDatabase, OuiEntry};
use clap::{Arg, App};
use termion::{event::Key, input::MouseTerminal, raw::IntoRawMode, screen::AlternateScreen};
use tui::{
    backend::TermionBackend,
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, List, ListItem, Tabs},
    style::{Style, Modifier, Color},
    text::Spans
};

mod ui;
mod wifi;
mod page;

fn main() {
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
        .arg(
            Arg::with_name("dont_monitor")
                .short("m")
                .long("dont-monitor")
                .help("Don't try entering monitor mode using libpcap")
        )
        .arg(
            Arg::with_name("database")
                .short("-d")
                .long("database")
                .help("Specify the path to the OUI database file")
                .value_name("FILE")
        )
        .get_matches();

    let mut ui = ui::Ui::new();

    let oui_db = if let Some(oui_path) = args.value_of("database") {
        let user_db = expect!(ui => fs::read_to_string(oui_path), "Unable to open specified OUI database file");
        expect!(ui => OuiDatabase::new_from_str(&user_db), "Unable to parse specified OUI database file")
    } else {
        expect!(ui => OuiDatabase::new_from_export(include_bytes!("../manuf")), "Unable to parse default OUI database")
    };
    
    let device = if args.is_present("interface") {
        let devices = expect!(ui => Device::list(), "Unable to find devices");
        let devices_names: Vec<_> = devices.iter().map(|d| ListItem::new(vec![Spans::from(d.name.as_str())])).collect();
        let list = List::new(devices_names)
            .block(Block::default().borders(Borders::ALL).title("Select a WiFi Device"))
            .highlight_style(Style::default().bg(Color::Reset).add_modifier(Modifier::REVERSED))
            .highlight_symbol("> ");
        let mut list_state = ui::ListState::with_item_count(devices.len());

        //let ui::Ui { terminal, input, ..} = &mut ui;
        fn draw(ui: &mut ui::Ui, list: &List, list_state: &mut ui::ListState) {
            expect!(
                ui =>
                    ui.terminal.draw(|f| f.render_stateful_widget(list.clone(), f.size(), list_state)), 
                    "Unable to create list widget"
            )
        }
        draw(&mut ui, &list, &mut list_state);
        'select_device: loop {
            for key in ui.input.stdin.iter() {
                match key {
                    Key::Esc => return,
                    Key::Up | Key::Char('w') => list_state.up(),
                    Key::Down | Key::Char('s') => list_state.down(),
                    Key::PageUp => list_state.top(),
                    Key::PageDown => list_state.bottom(),
                    Key::Char('\n') => break 'select_device devices[list_state.selected().unwrap()].clone(),
                    _ => continue
                }
                // Control flow will return after mutably borrowing the ui
                break
            }
            draw(&mut ui, &list, &mut list_state);
        }
    } else {
        expect!(ui => Device::lookup(), "Unable to choose a default device")
    };

    let capture = expect!(ui => Capture::from_device(device), "Unable to open capture device")
        .promisc(true)
        .rfmon(!args.is_present("dont_monitor"))
        .immediate_mode(true);
    let capture = expect!(ui => capture.open(), "Unable to start listening on capture device");
    let mut capture = expect!(ui => capture.setnonblock(), "Unable to capture packets in a non-blocking fashion");
    let mut savefile = expect!(ui => capture.savefile("capture.pcap"), "Unable to create save file for packet capture");

    if capture.get_datalink() != pcap::Linktype::IEEE802_11_RADIOTAP {
        let mut ok = false;
        for datalink in expect!(ui => capture.list_datalinks(), "Unable to determine supported datalink layers") {
            if datalink == pcap::Linktype::IEEE802_11_RADIOTAP {
                ok = true;
                expect!(ui => capture.set_datalink(datalink), "Unable to set the datalink layer")
            }
        }
        if !ok {
            let _: () = expect!(ui => Err(""), "The interface does not support the radiotap datalink layer required by this program");
        }
    }

    let mut devices = DeviceList::default();
    let pages: &mut [&mut dyn page::Page] = &mut [&mut page::Devices::new(), &mut page::Manufacturers::new()];
    let mut tabs = ui::TabState::new(pages.iter().map(|p| Spans::from(p.name())).collect());
    'sniff: loop {
        for key in ui.input.stdin.try_iter() {
            match key {
                Key::Esc => break 'sniff,
                Key::F(i) => tabs.select(i as usize),
                Key::Char('\t') => tabs.next(),
                Key::Up | Key::Char('w') => pages[tabs.index].up(),
                Key::Down | Key::Char('s') => pages[tabs.index].down(),
                Key::PageUp => pages[tabs.index].top(),
                Key::PageDown => pages[tabs.index].bottom(),
                _ => ()
            }
        }

        expect!(
            ui =>
                ui.terminal.draw(|frame| {
                    let areas = Layout::default()
                        .direction(Direction::Vertical)
                        .margin(0)
                        .constraints([Constraint::Length(2), Constraint::Min(0)])
                        .split(frame.size());
                    frame.render_widget(
                        Tabs::new(tabs.titles.clone())
                            .block(Block::default().borders(Borders::BOTTOM))
                            .select(tabs.index)
                            .style(Style::reset())
                            .highlight_style(Style::reset().add_modifier(Modifier::BOLD | Modifier::REVERSED)),
                        areas[0]
                    );
                    pages[tabs.index].render(frame, areas[1], &mut devices)
                }),
                "Unable to draw to stdout"
        );

        match capture.next() {
            Err(pcap::Error::NoMorePackets) | Err(pcap::Error::TimeoutExpired) => (),
            Err(error) => panic!("Error: {:?}", error),
            Ok(packet) => {
                savefile.write(&packet);
        
                let (radiotap, data) = expect!(ui => Radiotap::parse(packet.data), "Unable to parse radiotap header");
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
                                .sent()
                                .beacon(ssid)
                                .knows(destination);
                            devices.get_or_default(destination, &oui_db);
                        }
                        ProbeRequest {
                            source,
                            destination,
                            ..
                        } => {
                            devices.get_or_default(source, &oui_db)
                                .sent()
                                .knows(destination);
                        }
                        Ack {
                            receiver
                        } => {
                            devices.get_or_default(receiver, &oui_db);
                        }
                        _ => ()
                    }
                }
            }
        }
    }
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
    /// The devices that this one has referenced
    knows: HashSet<MacAddress>
}
impl KnownDevice {
    fn new(address: MacAddress, oui_db: &OuiDatabase) -> Self {
        Self {
            manufacturer: oui_db.query_by_mac(&address).unwrap(/* Library should never be able to return an error */),
            beacon: None,
            sent: false,
            knows: HashSet::new()
        }
    }
    fn sent(&mut self) -> &mut Self {
        self.sent = true;
        self
    }
    fn knows(&mut self, address: MacAddress) -> &mut Self {
        self.knows.insert(address);
        self
    }
    fn beacon(&mut self, ssid: String) -> &mut Self {
        self.beacon = Some(ssid);
        self
    }
}

#[derive(Debug, Default)]
pub struct DeviceList(HashMap<MacAddress, KnownDevice>);
impl DeviceList {
    fn get_or_default(&mut self, address: MacAddress, oui_db: &OuiDatabase) -> &mut KnownDevice {
        if self.contains_key(&address) {
            self.get_mut(&address).unwrap()
        } else {
            self.insert(address, KnownDevice::new(address, oui_db));
            self.get_mut(&address).unwrap()
        }
    }
    pub fn bar_data(&self) -> Vec<(&str, u64)> {
        let mut manufacturers = HashMap::new();
        for device in self.values() {
            if let Some(OuiEntry { name_short, ..}) = &device.manufacturer {
                if let Some(count) = manufacturers.get_mut(name_short.as_str()) {
                    *count += 1
                } else {
                    manufacturers.insert(name_short.as_str(), 1u64);
                }
            }
        }
        let mut values: Vec<(&str, u64)> = manufacturers.iter().map(|(&name, &count)| (name, count)).collect();
        values.sort_by(|(nl, l), (nr, r)| l.cmp(r).then_with(|| nl.cmp(nr)));
        values.reverse();
        values
        //&[("Apples and Oranges", 3)]
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
