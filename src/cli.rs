use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    pub human: bool,
}

#[derive(Parser)]
pub struct FetchRunDetailsCli {
    #[arg(long)]
    pub authentication_results: bool,
    #[arg(long)]
    pub email_addresses: bool,
    #[arg(long)]
    pub message_source: bool,
    #[arg(long)]
    pub reportable_entities: bool,
    pub run_id: u32,
}
