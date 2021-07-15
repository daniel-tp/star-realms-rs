use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Reqwest Error")]
    ReqwestError(#[from] reqwest::Error),
    #[error("Invalid API Response {0}")]
    InvalidAPIResponse(String),
    #[error("Unknown Player Name: {0}")]
    InvalidPlayerName(String),
    #[error("Unknown Core Version")]
    UnknownCoreVersion(),
    #[error("Unknown Star Realms Error")]
    Unknown,
}