pub(crate) mod authenticated;
mod unauthenticated;

use std::sync::Arc;

use bytes::Bytes;
use http_body_util::Full;
use hyper_rustls::HttpsConnector;
use hyper_util::client::legacy::Client;
use hyper_util::client::legacy::connect::HttpConnector;

use crate::{SessionToken, secret, urls};

/// Type alias for the hyper HTTP client used throughout the adapter.
pub(crate) type HyperClient = Client<HttpsConnector<HttpConnector>, Full<Bytes>>;

#[derive(Debug, Clone)]
pub struct BetfairRpcClient<T> {
    pub bot_login_client: HyperClient,
    pub bot_login_headers: http::HeaderMap,
    /// Cached root certificate store (loaded once from OS, reused for all clients).
    pub(crate) root_cert_store: Arc<rustls::RootCertStore>,
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
    authenticated_client: HyperClient,
    default_headers: http::HeaderMap,
}

impl BetfairRpcClient<Authenticated> {
    #[must_use]
    pub const fn session_token(&self) -> &SessionToken {
        &self.state.session_token
    }
}
