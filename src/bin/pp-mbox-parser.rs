use phisher_phinder_rust::mbox::parse;
use std::io;

fn main() {
    let mut mbox_contents = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut mbox_contents) {
            break;
        }
    }

    let mails = parse(&mbox_contents);

    print!("{}", serde_json::to_string(&mails).unwrap());
}
