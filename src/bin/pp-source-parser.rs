use phisher_phinder_rust::message_source::parse;
use std::io;

fn main() {
    let mut message_source = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut message_source) {
            break;
        }
    }

    let mails = parse(&message_source);

    print!("{}", serde_json::to_string(&mails).unwrap());
}
