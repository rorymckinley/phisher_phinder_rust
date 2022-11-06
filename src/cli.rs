use clap::Parser;

#[derive(Parser)]
pub struct Cli {
    #[arg(long)]
    pub human: bool,
}
