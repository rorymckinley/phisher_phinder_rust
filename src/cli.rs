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

#[derive(Debug, PartialEq, Parser)]
pub struct SingleCli {
    #[command(subcommand)]
    pub command: SingleCliCommands,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum SingleCliCommands {
    /// Configuration commands
    Config(ConfigArgs),
    /// Process an email
    Process(ProcessArgs)
}

#[derive(Debug, PartialEq, Args)]
pub struct ProcessArgs {
    #[arg(long, value_name = "RUN_ID")]
    pub reprocess_run: Option<i64>,
}

#[derive(Debug, PartialEq, Args)]
pub struct ConfigArgs {
    #[command(subcommand)]
    pub command: ConfigCommands,
}

#[derive(Debug, PartialEq, Subcommand)]
pub enum ConfigCommands {
    /// Show configuration file location
    Location,
    /// Set configuration
    Set(SetConfigArgs),
    /// Show existing configuration
    Show,
}

#[derive(Debug, PartialEq, Args)]
pub struct SetConfigArgs {
    #[arg(long)]
    /// Author name for abuse notifications
    pub abuse_notifications_author_name: Option<String>,
    #[arg(long)]
    /// Source email address for abuse notifications
    pub abuse_notifications_from_address: Option<String>,
    #[arg(long)]
    /// Path to SQLITE3 database
    pub db_path: Option<String>,
    #[arg(long)]
    /// Alternative RDAP bootstrap host - only used for testing purposes
    pub rdap_bootstrap_host: Option<String>,
    #[arg(long)]
    /// Host URI for SMTP server used to send abuse notifications
    pub smtp_host_uri: Option<String>,
    #[arg(long)]
    /// Password for SMTP server used to send abuse notifications
    pub smtp_password: Option<String>,
    #[arg(long)]
    /// Username for SMTP server used to send abuse notifications
    pub smtp_username: Option<String>,
    #[arg(long)]
    /// Host that is considered trusted when parsing `Received` headers (e.g `mx.google.com`)
    pub trusted_recipient: Option<String>,
}

impl SetConfigArgs {
    pub fn has_values(&self) -> bool {
        self != &Self {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: None,
            db_path: None,
            rdap_bootstrap_host: None,
            smtp_host_uri: None,
            smtp_password: None,
            smtp_username: None,
            trusted_recipient: None,
        }
    }
}

#[cfg(test)]
mod set_config_args_tests {
    use super::*;

    #[test]
    fn indicates_if_it_has_values() {
        let args = SetConfigArgs {
            abuse_notifications_author_name: Some("John Doe".into()),
            abuse_notifications_from_address: None,
            db_path: None,
            rdap_bootstrap_host: None,
            smtp_host_uri: None,
            smtp_password: None,
            smtp_username: None,
            trusted_recipient: None,
        };
        assert!(args.has_values());

        let args = SetConfigArgs {
            abuse_notifications_author_name: Some("John Doe".into()),
            abuse_notifications_from_address: None,
            db_path: None,
            rdap_bootstrap_host: None,
            smtp_host_uri: Some("smtp.unobtanium.com".into()),
            smtp_password: None,
            smtp_username: None,
            trusted_recipient: None,
        };
        assert!(args.has_values());

        let empty_args = SetConfigArgs {
            abuse_notifications_author_name: None,
            abuse_notifications_from_address: None,
            db_path: None,
            rdap_bootstrap_host: None,
            smtp_host_uri: None,
            smtp_password: None,
            smtp_username: None,
            trusted_recipient: None,
        };
        assert!(!empty_args.has_values());
    }
}
