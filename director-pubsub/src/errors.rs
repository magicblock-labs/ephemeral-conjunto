use thiserror::Error;

pub type DirectorPubsubResult<T> = Result<T, DirectorPubsubError>;

#[derive(Debug, Error)]
pub enum DirectorPubsubError {
    #[error("StdIoError")]
    StdIoError(#[from] std::io::Error),
    #[error("TunsgeniteWsError")]
    WsError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("UrlParseError")]
    URLParseError(#[from] url::ParseError),
    #[error("SerdeJSONError")]
    SerdeJSONError(#[from] serde_json::Error),

    #[error("ParseClientSubscription error: {0}")]
    ParseClientSubscription(String),
}
