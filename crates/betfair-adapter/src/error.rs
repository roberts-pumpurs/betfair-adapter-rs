//! This module defines the `ApiError` enum, which represents various errors
//! that can occur while interacting with the Betfair API. Each variant
//! corresponds to a specific type of error, providing context for debugging
//! and error handling.

#[derive(Debug, thiserror::Error)]
/// Represents errors that can occur while interacting with the Betfair API.
pub enum ApiError {
    /// Represents an error from the Sports Aping API.
    #[error(transparent)]
    SportsApingException(#[from] betfair_types::types::sports_aping::ApingException),

    /// Represents an error from the Account Aping API.
    #[error(transparent)]
    AccountApingException(#[from] betfair_types::types::account_aping::AccountApingException),

    /// Represents an error from the Heartbeat Aping API.
    #[error(transparent)]
    HeartbeatApingException(#[from] betfair_types::types::heartbeat_aping::ApingException),

    /// Represents an error from the Reqwest HTTP client.
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),

    /// Represents an error from Serde JSON serialization/deserialization.
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),

    /// Represents a URL parsing error.
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),

    /// Represents an error from the Eyre error reporting library.
    #[error(transparent)]
    EyreError(#[from] eyre::ErrReport),

    /// Represents a keep-alive error with specific values.
    #[error("Keep alive error: {0:?}")]
    KeepAliveError(betfair_types::keep_alive::ErrorValues),

    /// Represents a bot login error with specific details.
    #[error("Bot login error: {0:?}")]
    BotLoginError(betfair_types::bot_login::LoginError),

    /// Represents a logout error with specific values.
    #[error("Logout error: {0:?}")]
    LogoutError(betfair_types::logout::ErrorValues),

    /// Represents an empty response from the server.
    #[error("Empty response from server")]
    EmptyResponse,
}
