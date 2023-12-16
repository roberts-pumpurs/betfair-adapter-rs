pub mod auth;
mod error;
mod secret;
pub mod urls;

use std::fmt::Debug;
use std::marker::PhantomData;

pub use betfair_types;
use betfair_types::keep_alive;
pub use betfair_types::rust_decimal;
use betfair_types::types::BetfairRpcRequest;
pub use error::ApiError;
use reqwest::Client;
pub use secret::{ApplicationKey, Identity, Password, SecretProvider, Username};

pub struct Authenticated;
pub struct Unauthenticated;

#[derive(Debug)]
pub struct BetfairConfigBuilder<
    'a,
    T: urls::RetrieveUrl<'a, urls::RestBase> + Debug,
    K: urls::RetrieveUrl<'a, urls::KeepAlive> + Debug,
    V: urls::RetrieveUrl<'a, urls::BotLogin> + Debug,
    Z: urls::RetrieveUrl<'a, urls::Logout> + Debug,
    X: urls::RetrieveUrl<'a, urls::InteractiveLogin> + Debug,
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

#[derive(Debug)]
pub struct BetfairRpcProvider<'a, T> {
    client: Client,
    rest_base: urls::BetfairUrl<'a, urls::RestBase>,
    keep_alive: urls::BetfairUrl<'a, urls::KeepAlive>,
    bot_login: urls::BetfairUrl<'a, urls::BotLogin>,
    logout: urls::BetfairUrl<'a, urls::Logout>,
    login: urls::BetfairUrl<'a, urls::InteractiveLogin>,
    secret_provider: secret::SecretProvider<'a>,
    _type: PhantomData<T>,
}

impl<'a> BetfairRpcProvider<'a, Unauthenticated> {
    pub fn new(secret_provider: secret::SecretProvider<'a>) -> Self {
        let config = new_global_config(secret_provider);
        Self::new_with_config(config)
    }

    pub fn new_with_config(
        config: BetfairConfigBuilder<
            'a,
            impl urls::RetrieveUrl<'a, urls::RestBase> + core::fmt::Debug,
            impl urls::RetrieveUrl<'a, urls::KeepAlive> + core::fmt::Debug,
            impl urls::RetrieveUrl<'a, urls::BotLogin> + core::fmt::Debug,
            impl urls::RetrieveUrl<'a, urls::Logout> + core::fmt::Debug,
            impl urls::RetrieveUrl<'a, urls::InteractiveLogin> + core::fmt::Debug,
        >,
    ) -> Self {
        let rest_base = config.rest.url();
        let keep_alive = config.keep_alive.url();
        let bot_login = config.bot_login.url();
        let login = config.login.url();
        let logout = config.logout.url();
        let secret_provider = config.secrets_provider;

        Self {
            client: Client::new(),
            _type: PhantomData,
            rest_base,
            keep_alive,
            bot_login,
            login,
            logout,
            secret_provider,
        }
    }
}

impl<'a> BetfairRpcProvider<'a, Authenticated> {
    #[tracing::instrument(skip_all, ret, err, fields(req = ?request))]
    pub async fn send_request<T>(&self, request: T) -> Result<T::Res, ApiError>
    where
        T: BetfairRpcRequest + serde::Serialize + std::fmt::Debug,
        T::Res: serde::de::DeserializeOwned + std::fmt::Debug,
        T::Error: serde::de::DeserializeOwned,
        ApiError: From<<T as BetfairRpcRequest>::Error>,
    {
        let endpoint = self.rest_base.url().join(T::method())?;
        tracing::debug!(endpoint = ?endpoint.to_string(), "Sending request");
        let full = self
            .client
            .post(endpoint.as_str())
            .json(&request)
            .send()
            .await?;

        if full.status().is_success() {
            let res = full.json::<T::Res>().await?;
            return Ok(res)
        } else {
            let res = full.json::<T::Error>().await?;
            return Err(res.into())
        }
    }

    /// You can use Keep Alive to extend the session timeout period. The minimum session time is
    /// currently 20 minutes (Italian Exchange). On the international (.com) Exchange the current
    /// session time is 24 hours. Therefore, you should request Keep Alive within this time to
    /// prevent session expiry. If you don't call Keep Alive within the specified timeout period,
    /// the session will expire. Session times aren't determined or extended based on API activity.
    #[tracing::instrument(skip_all, ret, err)]
    pub async fn keep_alive(&self) -> Result<(), ApiError> {
        let _res = self
            .client
            .get(self.keep_alive.url().clone())
            .send()
            .await?
            .error_for_status()?
            .json::<keep_alive::Response>()
            .await?
            .0
            .map_err(ApiError::KeepAliveError)?;

        Ok(())
    }

    /// You can use Logout to terminate your existing session.
    #[tracing::instrument(skip_all, ret, err)]
    pub async fn logout(&self) -> Result<(), ApiError> {
        let _res = self
            .client
            .get(self.logout.url().clone())
            .send()
            .await?
            .error_for_status()?
            .json::<keep_alive::Response>()
            .await?
            .0
            .map_err(ApiError::LogoutError)?;

        Ok(())
    }
}
