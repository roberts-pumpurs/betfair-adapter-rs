use betfair_types::bot_login::BotLoginResponse;
use reqwest::{header, Client};

use super::BetfairRpcProviderBase;
use crate::secret::{self, SessionToken};
use crate::{
    urls, ApiError, ApplicationKey, AuthenticatedBetfairRpcProvider, BetfairConfigBuilder,
    Identity, UnauthenticatedBetfairRpcProvider,
};

impl UnauthenticatedBetfairRpcProvider {
    pub fn new(secret_provider: secret::SecretProvider) -> Result<Self, ApiError> {
        let config = BetfairConfigBuilder::new_with_global_jurisdiction(secret_provider);
        Self::new_with_config(config)
    }

    pub fn new_with_config(
        config: BetfairConfigBuilder<
            impl urls::RetrieveUrl<urls::RestBase> + core::fmt::Debug,
            impl urls::RetrieveUrl<urls::KeepAlive> + core::fmt::Debug,
            impl urls::RetrieveUrl<urls::BotLogin> + core::fmt::Debug,
            impl urls::RetrieveUrl<urls::Logout> + core::fmt::Debug,
            impl urls::RetrieveUrl<urls::InteractiveLogin> + core::fmt::Debug,
            impl urls::RetrieveUrl<urls::Stream> + core::fmt::Debug,
        >,
    ) -> Result<Self, ApiError> {
        BetfairRpcProviderBase::new(config).map(Self)
    }

    pub async fn authenticate(self) -> Result<AuthenticatedBetfairRpcProvider, ApiError> {
        let (session_token, authenticated_client) = self.0.bot_log_in().await?;

        Ok(AuthenticatedBetfairRpcProvider {
            base: self.0,
            session_token,
            authenticated_client,
        })
    }
}

impl BetfairRpcProviderBase {
    pub fn new(
        config: BetfairConfigBuilder<
            impl urls::RetrieveUrl<urls::RestBase> + core::fmt::Debug,
            impl urls::RetrieveUrl<urls::KeepAlive> + core::fmt::Debug,
            impl urls::RetrieveUrl<urls::BotLogin> + core::fmt::Debug,
            impl urls::RetrieveUrl<urls::Logout> + core::fmt::Debug,
            impl urls::RetrieveUrl<urls::InteractiveLogin> + core::fmt::Debug,
            impl urls::RetrieveUrl<urls::Stream> + core::fmt::Debug,
        >,
    ) -> Result<Self, ApiError> {
        let rest_base = config.rest.url();
        let keep_alive = config.keep_alive.url();
        let bot_login = config.bot_login.url();
        let login = config.login.url();
        let logout = config.logout.url();
        let stream = config.stream.url();
        let secret_provider = config.secrets_provider;

        // Use this to get the session token
        let bot_login_client =
            login_client(&secret_provider.application_key, &secret_provider.identity)?;

        Ok(Self {
            bot_login_client,
            rest_base,
            keep_alive,
            bot_login,
            login,
            logout,
            secret_provider,
            stream,
        })
    }

    /// Also known as "non interactive login"
    #[tracing::instrument(skip(self), err)]
    pub(super) async fn bot_log_in(&self) -> Result<(SessionToken, reqwest::Client), ApiError> {
        let login_response = self
            .bot_login_client
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
        let session_token = SessionToken(login_response.session_token);
        let authenticated_client =
            logged_in_client(&self.secret_provider.application_key, &session_token)?;
        Ok((session_token, authenticated_client))
    }

    // TODO implement interactive login
}

fn logged_in_client(
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
