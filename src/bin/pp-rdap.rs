use phisher_phinder_rust::data::OutputData;
use phisher_phinder_rust::populator::populate;
use phisher_phinder_rust::ui;
use rdap_client::Client;
use std::io;

#[tokio::main]
async fn main() {

    let mut raw_input = String::new();

    loop {
        if let Ok(0) = io::stdin().read_line(&mut raw_input) {
            break
        }
    }

    let mut output: OutputData = serde_json::from_str(&raw_input).unwrap();

    let mut client = Client::new();

    if let Ok(bootstrap_host) = std::env::var("RDAP_BOOTSTRAP_HOST") {
        client.set_base_bootstrap_url(&bootstrap_host)
    }

    let bootstrap = client.fetch_bootstrap().await.unwrap();

    populate(&bootstrap, &mut output).await;

    println!("{}", ui::display_sender_addresses_extended(&output).unwrap());
}
