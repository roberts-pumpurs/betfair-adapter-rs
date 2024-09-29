/// Errors that can occur in the stream processing module.
#[expect(clippy::module_name_repetitions)]
#[non_exhaustive]
#[derive(Debug, thiserror::Error)]
/// Represents errors that can occur in the stream processing module.
pub enum StreamError {
    /// Represents an I/O error.
    #[error("IO Error {0}")]
    IoError(#[from] std::io::Error),
    /// Represents a JSON parsing error.
    #[error("JSON Error {0}")]
    JsonError(#[from] serde_json::Error),
    /// Indicates that no data was received.
    #[error("No data")]
    NoData,
    /// Indicates an unexpected response from the stream.
    #[error("Unexpected response {0}")]
    UnexpectedResponse(String),
    /// Indicates that the connection ID is not present.
    #[error("Connection ID not present")]
    ConnectionIdNotPresent,
    /// Indicates a misconfigured stream URL.
    #[error("Misconfigured stream URL")]
    MisconfiguredStreamURL,
    /// Indicates a malfunction in the stream processor.
    #[error("Stream processor malfunctioned")]
    StreamProcessorMalfunction,
    /// Indicates that the host string is not present in the stream URL.
    #[error("Host string not present in the Stream URL")]
    HostStringNotPresent,
    /// Indicates an inability to look up the host.
    #[error("Unable to look up host {host}:{port}")]
    UnableToLookUpHost { host: String, port: u16 },
    /// Indicates an inability to convert a domain to a server name.
    #[error("Unable to convert domain to server name")]
    UnableConvertDomainToServerName,
    /// Indicates an inability to connect to a TLS stream.
    #[error("Unable to connect to TLS stream")]
    UnableConnectToTlsStream,
    /// Indicates an inability to set the native certificate.
    #[error("Unable to set native certificate")]
    CannotSetNativeCertificate,
    /// Indicates an inability to set a custom certificate.
    #[error("Unable to set custom certificate")]
    UnableToSetCustomCertificate,
    /// Indicates that a custom certificate is not set.
    #[error("Unable to set custom certificate")]
    CustomCertificateNotSet,
    /// Indicates that the custom certificate is invalid.
    #[error("Invalid custom certificate")]
    InvalidCustomCertificate,
    /// Indicates an inability to load a local certificate.
    #[error("Unable to load local certificate")]
    LocalCertificateLoadError,
}
