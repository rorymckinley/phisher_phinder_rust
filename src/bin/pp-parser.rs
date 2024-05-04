use mail_parser::*;
use std::io;

use phisher_phinder_rust::analyser::Analyser;
use phisher_phinder_rust::cli::Cli;
use phisher_phinder_rust::data::{OutputData, ParsedMail};
use phisher_phinder_rust::message_source::MessageSource;
use phisher_phinder_rust::ui;

use clap::Parser;

fn main() {
    let trusted_recipient = std::env::var("PP_TRUSTED_RECIPIENT")
        .expect("Please supply `PP_TRUSTED_RECIPIENT` ENV var");

    let cli = Cli::parse();

    let mut input = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut input) {
            break;
        }
    }

    let message_source: MessageSource = serde_json::from_str(&input).unwrap();

    let raw_mail = message_source.data.clone();

    let parsed_mail = Message::parse(raw_mail.as_bytes()).unwrap();

    let analyser = Analyser::new(&parsed_mail);

    let output = OutputData::new(
        ParsedMail::new(
            analyser.authentication_results(),
            analyser.delivery_nodes(Some(&trusted_recipient)),
            analyser.sender_email_addresses(),
            analyser.fulfillment_nodes(),
            analyser.subject(),
        ),
        message_source,
    );

    if cli.human {
        println!("{}", parsed_mail.subject().unwrap());
        println!();
        println!("Sender Addresses");
        println!(
            "{}",
            ui::display_sender_addresses_extended(&output.parsed_mail.email_addresses).unwrap()
        );
        println!();
        println!("Fulfillment Nodes");
        println!("{}", ui::display_fulfillment_nodes(&output).unwrap());
        println!();
        println!("Delivery Nodes");
        println!("{}", ui::display_delivery_nodes(&output).unwrap());
        println!();
        println!("Authentication Results");
        println!("{}", ui::display_authentication_results(&output).unwrap())
    } else {
        print!("{}", serde_json::to_string(&output).unwrap());
    }
}
