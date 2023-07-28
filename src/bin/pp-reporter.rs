use phisher_phinder_rust::data::OutputData;
use phisher_phinder_rust::reporter::add_reportable_entities;
use std::io;

fn main() {
    let mut raw_input = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut raw_input) {
            break;
        }
    }

    let input: OutputData = serde_json::from_str(&raw_input).unwrap();

    let output = add_reportable_entities(input);

    print!("{}", serde_json::to_string(&output).unwrap());
}
