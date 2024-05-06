use crate::{secret, urls};

#[derive(Debug, Clone)]
pub struct BetfairConfigBuilder<
    T: urls::RetrieveUrl<urls::RestBase> + std::fmt::Debug,
    K: urls::RetrieveUrl<urls::KeepAlive> + std::fmt::Debug,
    V: urls::RetrieveUrl<urls::BotLogin> + std::fmt::Debug,
    Z: urls::RetrieveUrl<urls::Logout> + std::fmt::Debug,
    X: urls::RetrieveUrl<urls::InteractiveLogin> + std::fmt::Debug,
    A: urls::RetrieveUrl<urls::Stream> + std::fmt::Debug,
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
    pub fn new_with_global_jurisdiction(
        secret_provider: secret::SecretProvider,
    ) -> BetfairConfigBuilder<
        urls::jurisdiction::Global,
        urls::jurisdiction::Global,
        urls::jurisdiction::Global,
        urls::jurisdiction::Global,
        urls::jurisdiction::Global,
        urls::jurisdiction::Global,
    > {
        BetfairConfigBuilder {
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
