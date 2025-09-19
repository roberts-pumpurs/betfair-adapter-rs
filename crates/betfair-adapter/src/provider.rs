pub(crate) mod authenticated;
mod unauthenticated;

use crate::{SessionToken, secret, urls};

#[derive(Debug, Clone)]
pub struct BetfairRpcClient<T> {
    pub bot_login_client: reqwest::Client,
    pub rest_base: urls::BetfairUrl<urls::RestBase>,
    pub keep_alive: urls::BetfairUrl<urls::KeepAlive>,
    pub bot_login: urls::BetfairUrl<urls::BotLogin>,
    pub logout: urls::BetfairUrl<urls::Logout>,
    pub login: urls::BetfairUrl<urls::InteractiveLogin>,
    pub stream: urls::BetfairUrl<urls::Stream>,
    pub secret_provider: secret::SecretProvider,
    pub state: T,
}

#[derive(Debug, Clone)]
#[repr(transparent)]
pub struct Unauthenticated;

#[derive(Debug, Clone)]
pub struct Authenticated {
    session_token: SessionToken,
    authenticated_client: reqwest::Client,
}

impl BetfairRpcClient<Authenticated> {
    #[must_use]
    pub const fn session_token(&self) -> &SessionToken {
        &self.state.session_token
    }
}
