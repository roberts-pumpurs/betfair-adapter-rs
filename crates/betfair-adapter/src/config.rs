use crate::{secret, urls};

#[derive(Debug, Clone)]
pub struct BetfairConfigBuilder<
    T: urls::RetrieveUrl<urls::RestBase> + core::fmt::Debug,
    K: urls::RetrieveUrl<urls::KeepAlive> + core::fmt::Debug,
    V: urls::RetrieveUrl<urls::BotLogin> + core::fmt::Debug,
    Z: urls::RetrieveUrl<urls::Logout> + core::fmt::Debug,
    X: urls::RetrieveUrl<urls::InteractiveLogin> + core::fmt::Debug,
    A: urls::RetrieveUrl<urls::Stream> + core::fmt::Debug,
> {
    pub rest: T,
    pub keep_alive: K,
    pub bot_login: V,
    pub logout: Z,
    pub login: X,
    pub stream: A,
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
    #[must_use]
    pub fn new_with_global_jurisdiction(secret_provider: secret::SecretProvider) -> Self {
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
