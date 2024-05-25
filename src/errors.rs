use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    Config(#[from] confy::ConfyError),
    #[error("Cannot locate the config file")]
    ConfigFileLocation,
    #[error("Database path {0} appears to be incorrect")]
    DatabasePathIncorrect(String),
    #[error("Database path is not configured")]
    DatabasePathNotConfigured,
    #[error("The provided database path is unparseable")]
    DatabasePathUnparseable,
    #[error("Please pass in message source(s) via STDIN or use --reprocess-run")]
    NoMessageSource,
    #[error("Service has nothing to process")]
    NothingToProcess,
    #[error("Can not find a run with the given ID")]
    SpecifiedRunMissing,
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    UTF8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error),
    #[error("PersistenceError: {0}")]
    Persistence(String),
    #[error("Can not generate abuse notifications without PP_ABUSE_NOTIFICATIONS_FROM_ADDRESS")]
    NoFromAddressForNotifications,
    #[error("Can not generate abuse notifications without PP_ABUSE_NOTIFICATIONS_AUTHOR_NAME")]
    NoAuthorNameForNotifications,
    #[error("Encountered an irrecoverable error ,please contact the developer")]
    Irrecoverable,
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String)
}
