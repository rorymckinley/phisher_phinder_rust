use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error(transparent)]
    IO(#[from] std::io::Error),
    #[error(transparent)]
    UTF8(#[from] std::string::FromUtf8Error),
    #[error(transparent)]
    Rusqlite(#[from] rusqlite::Error)
}
