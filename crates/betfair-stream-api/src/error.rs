#[expect(clippy::module_name_repetitions)]
#[non_exhaustive]
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
    #[error("Host string not present in the Stream URL")]
    HostStringNotPresent,
    #[error("Unable to look up host {host}:{port}")]
    UnableToLookUpHost { host: String, port: u16 },
    #[error("Unable to convert domain to server name")]
    UnableConvertDomainToServerName,
    #[error("Unable to connect to TLS stream")]
    UnableConnectToTlsStream,
    #[error("Unable to set native certificate")]
    CannotSetNativeCertificate,
    #[error("Unable to set custom certificate")]
    UnableToSetCustomCertificate,
    #[error("Unable to set custom certificate")]
    CustomCertificateNotSet,
    #[error("Invalid custom certificate")]
    InvalidCustomCertificate,
    #[error("Unable to load local certificate")]
    LocalCertificateLoadError,
}
