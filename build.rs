use std::fs;

fn main() {
    let response = reqwest::blocking::get("https://gitlab.com/wireshark/wireshark/raw/master/manuf").expect("Unable to fetch OUI database");
    let oui_raw = response.text().expect("Unable to read OUI database contents");
    let db = oui::OuiDatabase::new_from_str(&oui_raw).expect("Fetched OUI database is invalid");
    let db_parsed = db.export().expect("Unable to export OUI database");
    fs::write("manuf", db_parsed).expect("Unable to save default OUI database");
}