use pcap::{Capture, Device};
use radiotap::Radiotap;
use clap::{Arg, App};
use termion::{event::Key, input::{MouseTerminal, TermRead}, raw::IntoRawMode, screen::AlternateScreen};
use tui::{backend::TermionBackend, layout::Layout, widgets::{Block, Borders, List, ListItem, ListState}, text::{Span, Spans}};

mod ui;
mod wifi;

fn main() {
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

    std::mem::drop(terminal);

    let mut capture = Capture::from_device(device).unwrap()
        .promisc(true)
        .rfmon(true)
        .immediate_mode(true)
        .open().unwrap();
    let mut savefile = capture.savefile("capture.pcap").unwrap();

    println!("Current datalink: {:?}", capture.get_datalink().get_name());
    for dl in capture.list_datalinks().unwrap() {
        println!("Datalink: {:?}", dl)
    }
    loop {
        for key in input.stdin.try_iter() {
            match key {
                Key::Char('q') => return,
                _ => ()
            }
        }
        let packet = capture.next().unwrap();
        savefile.write(&packet);
        
        let (radiotap, data) = Radiotap::parse(packet.data).unwrap();
        println!("{:?}", wifi::Frame::parse(data));
    }
}
