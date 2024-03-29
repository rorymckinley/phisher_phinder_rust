use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Database path {0} appears to be incorrect")]
    DatabasePathIncorrect(String),
    #[error("The provided database path is unparseable")]
    DatabasePathUnparseable,
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
}
