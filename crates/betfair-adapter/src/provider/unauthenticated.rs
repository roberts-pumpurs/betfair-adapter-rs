use std::marker::PhantomData;

use betfair_types::bot_login::BotLoginResponse;
use reqwest::{header, Client};

use crate::secret::{self, SessionToken};
use crate::{
    new_global_config, urls, ApiError, ApplicationKey, Authenticated, BetfairConfigBuilder,
    BetfairRpcProvider, Identity, Unauthenticated,
};

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

    pub async fn authenticate(mut self) -> Result<BetfairRpcProvider<'a, Authenticated>, ApiError> {
        self.bot_log_in().await?;

        let instance: BetfairRpcProvider<'a, Authenticated> = unsafe { std::mem::transmute(self) };

        Ok(instance)
    }
}

impl<'a, T> BetfairRpcProvider<'a, T> {
    /// Also known as "non interactive login"
    #[tracing::instrument(skip(self), err)]
    pub(super) async fn bot_log_in(&mut self) -> Result<(), ApiError> {
        // Use this to get the session token
        let temp_client = login_client(
            &self.secret_provider.application_key,
            &self.secret_provider.identity,
        )?;

        let login_response = temp_client
            .post(self.bot_login.url().as_str())
            .form(&[
                ("username", self.secret_provider.username.0.expose_secret()),
                ("password", self.secret_provider.password.0.expose_secret()),
            ])
            .send()
            .await?
            .json::<BotLoginResponse>()
            .await?;
        let login_response = login_response.0.map_err(ApiError::BotLoginError)?;

        self.client = default_client(
            &self.secret_provider.application_key,
            &SessionToken(login_response.session_token),
        )?;

        Ok(())
    }

    // TODO implement interactive login
}

fn default_client(
    app_key: &ApplicationKey,
    session_token: &SessionToken,
) -> eyre::Result<reqwest::Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "X-Application",
        header::HeaderValue::from_str(app_key.0.expose_secret().as_str())?,
    );
    headers.insert(
        "Accept",
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "Accept-Encoding",
        header::HeaderValue::from_static("gzip, deflate"),
    );
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "X-Authentication",
        header::HeaderValue::from_str(session_token.0.expose_secret().as_str())?,
    );
    Ok(reqwest::Client::builder()
        .use_rustls_tls()
        .default_headers(headers)
        .build()?)
}

fn login_client(application_key: &ApplicationKey, identity: &Identity) -> Result<Client, ApiError> {
    const KEEP_ALIVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "X-Application",
        header::HeaderValue::from_str(application_key.0.expose_secret().as_str()).unwrap(),
    );
    headers.insert(
        "Accept",
        header::HeaderValue::from_static("application/json"),
    );
    headers.insert(
        "Content-Type",
        header::HeaderValue::from_static("application/x-www-form-urlencoded"),
    );
    let temp_client = reqwest::Client::builder()
        .use_rustls_tls()
        .identity(identity.0.expose_secret().clone())
        .default_headers(headers)
        .http2_keep_alive_interval(KEEP_ALIVE_INTERVAL)
        .http2_keep_alive_while_idle(true)
        .build()?;
    Ok(temp_client)
}
