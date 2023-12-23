#[derive(Debug, thiserror::Error)]
pub enum StreamError {
    #[error("IO Error {0}")]
    IoError(#[from] std::io::Error),
    #[error("JSON Error {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("No data")]
    NoData,
    #[error("Unexpected response {0}")]
    UnexpectedResponse(String),
    #[error("Connection ID not present")]
    ConnectionIdNotPresent,
    #[error("Misconfigured stream URL")]
    MisconfiguredStreamURL,
    #[error("Stream processor malfunctioned")]
    StreamProcessorMalfunction,
}
