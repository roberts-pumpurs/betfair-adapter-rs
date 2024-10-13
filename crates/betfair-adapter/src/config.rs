use crate::{secret, urls};

/// A builder for Betfair configuration with various URL retrieval types.
#[derive(Debug, Clone)]
pub struct BetfairConfigBuilder<
    T: urls::RetrieveUrl<urls::RestBase> + core::fmt::Debug = urls::jurisdiction::Global,
    K: urls::RetrieveUrl<urls::KeepAlive> + core::fmt::Debug = urls::jurisdiction::Global,
    V: urls::RetrieveUrl<urls::BotLogin> + core::fmt::Debug = urls::jurisdiction::Global,
    Z: urls::RetrieveUrl<urls::Logout> + core::fmt::Debug = urls::jurisdiction::Global,
    X: urls::RetrieveUrl<urls::InteractiveLogin> + core::fmt::Debug = urls::jurisdiction::Global,
    A: urls::RetrieveUrl<urls::Stream> + core::fmt::Debug = urls::jurisdiction::Global,
> {
    /// The URL for REST API calls.
    pub rest: T,
    /// The URL for keep-alive requests.
    pub keep_alive: K,
    /// The URL for bot login.
    pub bot_login: V,
    /// The URL for logout.
    pub logout: Z,
    /// The URL for login.
    pub login: X,
    /// The URL for streaming data.
    pub stream: A,
    /// The provider for secret management.
    pub secrets_provider: secret::SecretProvider,
}

impl
    BetfairConfigBuilder<
        urls::jurisdiction::Global,
        urls::jurisdiction::Global,
        urls::jurisdiction::Global,
        urls::jurisdiction::Global,
        urls::jurisdiction::Global,
        urls::jurisdiction::Global,
    >
{
    /// Creates a new instance of `BetfairConfigBuilder` with global jurisdiction.
    ///
    /// # Parameters
    /// - `secret_provider`: The provider for managing secrets.
    #[must_use]
    pub const fn new_with_global_jurisdiction(secret_provider: secret::SecretProvider) -> Self {
        Self {
            rest: urls::jurisdiction::Global,
            keep_alive: urls::jurisdiction::Global,
            bot_login: urls::jurisdiction::Global,
            logout: urls::jurisdiction::Global,
            login: urls::jurisdiction::Global,
            stream: urls::jurisdiction::Global,
            secrets_provider: secret_provider,
        }
    }
}
