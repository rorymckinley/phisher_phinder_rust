use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    pub sender_email_addresses: bool,
    #[arg(long)]
    human: bool,
    #[arg(long)]
    pub subject: bool,
}
