pub(crate) mod authenticated;
mod unauthenticated;

use crate::{secret, urls, SessionToken};

#[derive(Debug, Clone)]
/// A base provider for Betfair RPC.
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

/// A provider for Betfair RPC that does not require authentication.
#[repr(transparent)]
#[derive(Debug, Clone)]
pub struct UnauthenticatedBetfairRpcProvider(BetfairRpcProviderBase);

/// A provider for Betfair RPC that requires authentication.
#[derive(Debug, Clone)]
pub struct AuthenticatedBetfairRpcProvider {
    base: BetfairRpcProviderBase,
    session_token: SessionToken,
    authenticated_client: reqwest::Client,
}

impl AuthenticatedBetfairRpcProvider {
    /// Returns a reference to the session token.
    #[must_use]
    pub const fn session_token(&self) -> &SessionToken {
        &self.session_token
    }

    /// Returns a reference to the base provider.
    #[must_use]
    pub const fn base(&self) -> &BetfairRpcProviderBase {
        &self.base
    }
}

impl UnauthenticatedBetfairRpcProvider {
    /// Returns a reference to the base provider.
    #[must_use]
    pub const fn base(&self) -> &BetfairRpcProviderBase {
        &self.0
    }
}
