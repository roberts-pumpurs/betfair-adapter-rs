use crate::{secret, urls};

#[derive(Debug)]
pub struct BetfairConfigBuilder<
    'a,
    T: urls::RetrieveUrl<'a, urls::RestBase> + std::fmt::Debug,
    K: urls::RetrieveUrl<'a, urls::KeepAlive> + std::fmt::Debug,
    V: urls::RetrieveUrl<'a, urls::BotLogin> + std::fmt::Debug,
    Z: urls::RetrieveUrl<'a, urls::Logout> + std::fmt::Debug,
    X: urls::RetrieveUrl<'a, urls::InteractiveLogin> + std::fmt::Debug,
> {
    pub rest: T,
    pub keep_alive: K,
    pub bot_login: V,
    pub logout: Z,
    pub login: X,
    pub secrets_provider: secret::SecretProvider<'a>,
}

pub fn new_global_config(
    secret_provider: secret::SecretProvider<'_>,
) -> BetfairConfigBuilder<
    urls::jurisdictions::Global,
    urls::jurisdictions::Global,
    urls::jurisdictions::Global,
    urls::jurisdictions::Global,
    urls::jurisdictions::Global,
> {
    BetfairConfigBuilder {
        rest: urls::jurisdictions::Global,
        keep_alive: urls::jurisdictions::Global,
        bot_login: urls::jurisdictions::Global,
        logout: urls::jurisdictions::Global,
        login: urls::jurisdictions::Global,
        secrets_provider: secret_provider,
    }
}
