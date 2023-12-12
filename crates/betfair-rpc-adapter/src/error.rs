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
    #[error(transparent)]
    EyreError(#[from] eyre::ErrReport),
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
            ApiError::EyreError(e) => write!(f, "{}", e),
        }
    }
}

pub trait ApingException {
    fn invalid_app_key(&self) -> bool;
    fn invalid_session(&self) -> bool;
}

impl ApingException for betfair_types::types::sports_aping::ApingException {
    fn invalid_app_key(&self) -> bool {
        self.error_code.map_or(false, |x| {
            matches!(
                x,
                betfair_types::types::sports_aping::ErrorCode::NoAppKey |
                    betfair_types::types::sports_aping::ErrorCode::InvalidAppKey
            )
        })
    }

    fn invalid_session(&self) -> bool {
        self.error_code.map_or(false, |x| {
            matches!(
                x,
                betfair_types::types::sports_aping::ErrorCode::InvalidSessionInformation |
                    betfair_types::types::sports_aping::ErrorCode::NoSession
            )
        })
    }
}
impl ApingException for betfair_types::types::heartbeat_aping::ApingException {
    fn invalid_app_key(&self) -> bool {
        self.error_code.map_or(false, |x| {
            matches!(
                x,
                betfair_types::types::heartbeat_aping::ErrorCode::NoAppKey |
                    betfair_types::types::heartbeat_aping::ErrorCode::InvalidAppKey
            )
        })
    }

    fn invalid_session(&self) -> bool {
        self.error_code.map_or(false, |x| {
            matches!(
                x,
                betfair_types::types::heartbeat_aping::ErrorCode::InvalidSessionInformation |
                    betfair_types::types::heartbeat_aping::ErrorCode::NoSession
            )
        })
    }
}
impl ApingException for betfair_types::types::account_aping::AccountApingException {
    fn invalid_app_key(&self) -> bool {
        self.error_code.map_or(false, |x| {
            matches!(
                x,
                betfair_types::types::account_aping::ErrorCode::NoAppKey |
                    betfair_types::types::account_aping::ErrorCode::InvalidAppKey
            )
        })
    }

    fn invalid_session(&self) -> bool {
        self.error_code.map_or(false, |x| {
            matches!(
                x,
                betfair_types::types::account_aping::ErrorCode::InvalidSessionInformation |
                    betfair_types::types::account_aping::ErrorCode::NoSession
            )
        })
    }
}
