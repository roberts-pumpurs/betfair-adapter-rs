use crate::keep_alive::ErrorValues;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    SportsApingException(#[from] betfair_types::types::sports_aping::ApingException),
    #[error(transparent)]
    AccountApingException(#[from] betfair_types::types::account_aping::AccountApingException),
    #[error(transparent)]
    HeartbeatApingException(#[from] betfair_types::types::heartbeat_aping::ApingException),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
    #[error(transparent)]
    EyreError(#[from] eyre::ErrReport),
    #[error("Keep alive error: {0:?}")]
    KeepAliveError(ErrorValues),
}
