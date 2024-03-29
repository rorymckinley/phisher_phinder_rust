use clap::Parser;
use phisher_phinder_rust::cli::Cli;
use phisher_phinder_rust::data::OutputData;
use phisher_phinder_rust::enumerator::enumerate;
use phisher_phinder_rust::ui;
use std::io;

#[tokio::main]
async fn main() {
    let mut raw_input = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut raw_input) {
            break;
        }
    }

    let input: OutputData = serde_json::from_str(&raw_input).unwrap();

    let output = enumerate(input).await;

    let cli = Cli::parse();

    if cli.human {
        println!("{}", ui::display_fulfillment_nodes(&output).unwrap());
    } else {
        print!("{}", serde_json::to_string(&output).unwrap());
    }
}
