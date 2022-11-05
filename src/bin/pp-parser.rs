use std::io;
use mail_parser::*;
use serde::Serialize;

use phisher_phinder_rust::cli::Cli;
use phisher_phinder_rust::analyser::{Analyser, SenderAddresses};
use phisher_phinder_rust::ui;

use clap::Parser;

#[derive(Serialize)]
struct Output {
    parsed_mail: ParsedMailOutput,
}

#[derive(Serialize)]
struct ParsedMailOutput {
    sender_addresses: SenderAddresses,
    subject: Option<String>,
}

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

    if cli.human {
        println!("{}", parsed_mail.get_subject().unwrap());
        println!();
        println!("{}", ui::display_sender_addresses(&analyser.sender_email_addresses()).unwrap())
    } else {
        let output = Output {
            parsed_mail: ParsedMailOutput {
                subject: analyser.subject(),
                sender_addresses: analyser.sender_email_addresses()
            }
        };
        print!("{}", serde_json::to_string(&output).unwrap());
    }
}
