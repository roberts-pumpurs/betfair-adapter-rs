use std::sync::Arc;
use std::time::Duration;

use backon::{BackoffBuilder, ExponentialBuilder};
use betfair_types::bot_login::BotLoginResponse;
use reqwest::{Client, header};
use tokio::task::JoinHandle;
use tokio::time::{Instant, interval, sleep};

use super::BetfairRpcClient;
use crate::secret::{self, SessionToken};
use crate::{
    ApiError, ApplicationKey, Authenticated, BetfairConfigBuilder, Identity, Unauthenticated, urls,
};

/// Represents an unauthenticated Betfair RPC provider.
impl BetfairRpcClient<Unauthenticated> {
    /// Creates a new instance of `UnauthenticatedBetfairRpcProvider` using the provided secret
    /// provider.
    pub fn new(secret_provider: secret::SecretProvider) -> Result<Self, ApiError> {
        let config = BetfairConfigBuilder::new_with_global_jurisdiction(secret_provider);
        Self::new_with_config(config)
    }

    /// Authenticates the user and returns an `AuthenticatedBetfairRpcProvider`.
    pub async fn authenticate(
        self,
    ) -> Result<
        (
            Arc<BetfairRpcClient<Authenticated>>,
            JoinHandle<Result<(), ApiError>>,
        ),
        ApiError,
    > {
        let mut backoff = ExponentialBuilder::new().build();
        let mut first_call = true;
        let (session_token, authenticated_client) = loop {
            if !first_call {
                // add exponential recovery
                let next = backoff.next();
                let Some(delay) = next else {
                    return Err(ApiError::EyreError(eyre::eyre!("could not authenticate")));
                };
                sleep(delay).await;
            }
            first_call = true;
            let Ok(res) = self.bot_log_in().await else {
                continue;
            };

            break res;
        };

        let state = Authenticated {
            session_token,
            authenticated_client,
        };
        let client = Arc::new(BetfairRpcClient {
            state,
            bot_login_client: self.bot_login_client,
            rest_base: self.rest_base,
            keep_alive: self.keep_alive,
            bot_login: self.bot_login,
            logout: self.logout,
            login: self.login,
            stream: self.stream,
            secret_provider: self.secret_provider,
        });

        let keep_alive = tokio::spawn({
            let client = Arc::downgrade(&client);
            async move {
                const MINUTES_15: u64 = 60 * 15;
                let mut interval = interval(Duration::from_secs(MINUTES_15));

                loop {
                    interval.tick().await;
                    let Some(client) = client.upgrade() else {
                        // all references to the client have been dropped, we can exit the `keep-alive` loop
                        return Ok(());
                    };

                    let t1 = Instant::now();
                    let _res = client.keep_alive()?.execute().await?;
                    let t2 = Instant::now();
                    let diff = t2.saturating_duration_since(t1);
                    tracing::info!(?diff, "latency");
                }
            }
        });

        Ok((client, keep_alive))
    }

    /// Creates a new instance of `UnauthenticatedBetfairRpcProvider` with a specific configuration.
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
            state: Unauthenticated,
            bot_login_client,
            rest_base,
            keep_alive,
            bot_login,
            logout,
            login,
            stream,
            secret_provider,
        })
    }
}

impl<T> BetfairRpcClient<T> {
    /// Performs a non-interactive login to obtain a session token and authenticated client.
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

/// Creates a logged-in HTTP client with the specified application key and session token.
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

/// Creates a login client with the specified application key and identity.
fn login_client(application_key: &ApplicationKey, identity: &Identity) -> Result<Client, ApiError> {
    const KEEP_ALIVE_INTERVAL: core::time::Duration = core::time::Duration::from_secs(15);
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
