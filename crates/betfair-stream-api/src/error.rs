#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    JsonError(#[from] serde_json::Error),
    #[error("No data")]
    NoData,
    #[error("Unexpected response {0}")]
    UnexpectedResponse(String),
    #[error("Connection ID not present")]
    ConnectionIdNotPresent,
    #[error("Misconfigured stream URL")]
    MisconfiguredStreamURL,
}
