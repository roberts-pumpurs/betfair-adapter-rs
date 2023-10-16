#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    SportsApingException(#[from] betfair_types::types::sports_aping::ApingException),
    #[error(transparent)]
    AccountApingException(#[from] betfair_types::types::account_aping::AccountApingException),
    HeartbeatApingException(#[from] betfair_types::types::heartbeat_aping::ApingException),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
    #[error(transparent)]
    UrlParseError(#[from] url::ParseError),
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::SportsApingException(e) => write!(f, "{}", e),
            ApiError::AccountApingException(e) => write!(f, "{}", e),
            ApiError::HeartbeatApingException(e) => write!(f, "{}", e),
            ApiError::ReqwestError(e) => write!(f, "{}", e),
            ApiError::SerdeError(e) => write!(f, "{}", e),
            ApiError::UrlParseError(e) => write!(f, "{}", e),
        }
    }
}
