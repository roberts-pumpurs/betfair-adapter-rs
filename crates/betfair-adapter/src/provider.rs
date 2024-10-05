pub(crate) mod authenticated;
mod unauthenticated;

use crate::{secret, urls, SessionToken};

#[derive(Debug, Clone)]
pub struct BetfairRpcProviderBase {
    pub bot_login_client: reqwest::Client,
    pub rest_base: urls::BetfairUrl<urls::RestBase>,
    pub keep_alive: urls::BetfairUrl<urls::KeepAlive>,
    pub bot_login: urls::BetfairUrl<urls::BotLogin>,
    pub logout: urls::BetfairUrl<urls::Logout>,
    pub login: urls::BetfairUrl<urls::InteractiveLogin>,
    pub stream: urls::BetfairUrl<urls::Stream>,
    pub secret_provider: secret::SecretProvider,
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct UnauthenticatedBetfairRpcProvider(BetfairRpcProviderBase);

#[derive(Debug, Clone)]
pub struct AuthenticatedBetfairRpcProvider {
    base: BetfairRpcProviderBase,
    session_token: SessionToken,
    authenticated_client: reqwest::Client,
}

impl AuthenticatedBetfairRpcProvider {
    #[must_use]
    pub const fn session_token(&self) -> &SessionToken {
        &self.session_token
    }

    #[must_use]
    pub const fn base(&self) -> &BetfairRpcProviderBase {
        &self.base
    }
}

impl UnauthenticatedBetfairRpcProvider {
    #[must_use]
    pub const fn base(&self) -> &BetfairRpcProviderBase {
        &self.0
    }
}
