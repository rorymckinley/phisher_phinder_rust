use std::io;
use mail_parser::*;

use phisher_phinder_rust::cli::Cli;
use phisher_phinder_rust::analyser::Analyser;
use phisher_phinder_rust::ui;

use clap::Parser;

fn main() {
    let cli = Cli::parse();

    let mut mail = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut mail) {
            break
        }
    }

    let parsed_mail = Message::parse(mail.as_bytes()).unwrap();

    if cli.subject {
        println!("{}", parsed_mail.get_subject().unwrap())
    } else if cli.sender_email_addresses {
        let analyser = Analyser::new(&parsed_mail);
        println!("{}", ui::display_sender_addresses(&analyser.sender_email_addresses()).unwrap())
    }
}
