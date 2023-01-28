use std::io;
use mail_parser::*;

use phisher_phinder_rust::cli::Cli;
use phisher_phinder_rust::analyser::Analyser;
use phisher_phinder_rust::data::{OutputData, ParsedMail};
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

    let analyser = Analyser::new(&parsed_mail);

    let output = OutputData::new(
        ParsedMail::new(
            analyser.sender_email_addresses(), analyser.fulfillment_nodes(), analyser.subject(), 
        ),
        &mail,
    );

    if cli.human {
        println!("{}", parsed_mail.get_subject().unwrap());
        println!();
        println!("{}", ui::display_sender_addresses_extended(&output).unwrap());
        println!();
        println!("{}", ui::display_fulfillment_nodes(&output).unwrap());
        
    } else {
        print!("{}", serde_json::to_string(&output).unwrap());
    }
}
