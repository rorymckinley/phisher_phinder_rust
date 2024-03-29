use clap::Parser;
use phisher_phinder_rust::cli::Cli;
use phisher_phinder_rust::data::OutputData;
use phisher_phinder_rust::populator::populate;
use phisher_phinder_rust::ui;
use std::io;
use std::sync::Arc;
use test_friendly_rdap_client::Client;

#[tokio::main]
async fn main() {
    let mut raw_input = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut raw_input) {
            break;
        }
    }

    let input: OutputData = serde_json::from_str(&raw_input).unwrap();

    let mut client = Client::new();

    if let Ok(bootstrap_host) = std::env::var("RDAP_BOOTSTRAP_HOST") {
        client.set_base_bootstrap_url(&bootstrap_host)
    }

    let bootstrap = client.fetch_bootstrap().await.unwrap();

    let b_strap = Arc::new(bootstrap);

    let output = populate(Arc::clone(&b_strap), input).await;

    let cli = Cli::parse();

    if cli.human {
        println!(
            "{}",
            ui::display_sender_addresses_extended(&output.parsed_mail.email_addresses).unwrap()
        );
        println!();
        println!("{}", ui::display_fulfillment_nodes(&output).unwrap());
        println!();
        println!("{}", ui::display_delivery_nodes(&output).unwrap())
    } else {
        print!("{}", serde_json::to_string(&output).unwrap());
    }
}
