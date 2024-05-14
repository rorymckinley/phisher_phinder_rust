use clap::{Args, Parser, Subcommand};

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
    pub pipe_message_source: bool,
    #[arg(long)]
    pub reportable_entities: bool,
    pub run_id: i64,
}

#[derive(Parser)]
pub struct FindOtherRunsCli {
    pub run_id: i64,
}

#[derive(Parser)]
pub struct SingleCli {
    #[command(subcommand)]
    pub command: SingleCliCommands,
}

#[derive(Subcommand)]
pub enum SingleCliCommands {
    /// Configuration commands
    Config(ConfigArgs),
    /// Process an email
    Process(ProcessArgs)
}

#[derive(Args)]
pub struct ProcessArgs {
    #[arg(long, value_name = "RUN_ID")]
    pub reprocess_run: Option<i64>,
}

#[derive(Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Subcommand)]
pub enum ConfigCommands {
    /// Show configuration file location
    Location,
    /// Set configuration
    Set,
    /// Show existing configuration
    Show,
}
