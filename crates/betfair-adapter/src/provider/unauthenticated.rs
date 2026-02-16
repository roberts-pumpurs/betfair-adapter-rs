use std::sync::Arc;
use std::time::Duration;

use backon::{BackoffBuilder, ExponentialBuilder};
use betfair_types::bot_login::BotLoginResponse;
use bytes::Bytes;
use http::header;
use http_body_util::{BodyExt, Full};
use tokio::task::JoinHandle;
use tokio::time::{Instant, interval, sleep};

use super::{BetfairRpcClient, HyperClient};
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
        let (session_token, authenticated_client, default_headers) = loop {
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
            default_headers,
        };
        let client = Arc::new(BetfairRpcClient {
            state,
            bot_login_client: self.bot_login_client,
            bot_login_headers: self.bot_login_headers,
            root_cert_store: self.root_cert_store,
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

        // Load native root certificates once — reused for all TLS clients
        let root_cert_store = Arc::new(load_native_root_certs()?);

        // Use this to get the session token
        let (bot_login_client, bot_login_headers) = login_client(
            &secret_provider.application_key,
            &secret_provider.identity,
            &root_cert_store,
        )?;

        Ok(Self {
            state: Unauthenticated,
            bot_login_client,
            bot_login_headers,
            root_cert_store,
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
    pub(super) async fn bot_log_in(
        &self,
    ) -> Result<(SessionToken, HyperClient, http::HeaderMap), ApiError> {
        let url: http::Uri = self
            .bot_login
            .url()
            .as_str()
            .parse()
            .map_err(|e: http::uri::InvalidUri| eyre::eyre!(e))?;

        let form_body = serde_urlencoded::to_string([
            ("username", self.secret_provider.username.0.expose_secret()),
            ("password", self.secret_provider.password.0.expose_secret()),
        ])
        .map_err(|e| eyre::eyre!(e))?;

        let mut request_builder = http::Request::builder().method(http::Method::POST).uri(url);

        for (key, value) in &self.bot_login_headers {
            request_builder = request_builder.header(key, value);
        }

        let request = request_builder.body(Full::new(Bytes::from(form_body)))?;

        let response = self.bot_login_client.request(request).await?;
        let status = response.status();
        let body = response.into_body().collect().await.map_err(|e| {
            ApiError::EyreError(eyre::eyre!("failed to collect response body: {e}"))
        })?;
        let bytes = body.to_bytes();

        if !status.is_success() {
            return Err(ApiError::EyreError(eyre::eyre!(
                "bot login failed with status {status}: {}",
                String::from_utf8_lossy(&bytes)
            )));
        }

        let login_response = serde_json::from_slice::<BotLoginResponse>(&bytes)?;
        let login_response = login_response.0.map_err(ApiError::BotLoginError)?;
        let session_token = SessionToken(login_response.session_token);
        let (authenticated_client, default_headers) = logged_in_client(
            &self.secret_provider.application_key,
            &session_token,
            &self.root_cert_store,
        )?;
        Ok((session_token, authenticated_client, default_headers))
    }

    // TODO implement interactive login
}

/// Creates a logged-in HTTP client with the specified application key and session token.
fn logged_in_client(
    app_key: &ApplicationKey,
    session_token: &SessionToken,
    root_cert_store: &Arc<rustls::RootCertStore>,
) -> eyre::Result<(HyperClient, http::HeaderMap)> {
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

    let client = build_https_client(None, root_cert_store)?;
    Ok((client, headers))
}

/// Creates a login client with the specified application key and identity (client certificate).
fn login_client(
    application_key: &ApplicationKey,
    identity: &Identity,
    root_cert_store: &Arc<rustls::RootCertStore>,
) -> Result<(HyperClient, http::HeaderMap), ApiError> {
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

    let pem_identity = identity.0.expose_secret();
    let client = build_https_client(Some(pem_identity), root_cert_store)
        .map_err(|e| ApiError::EyreError(eyre::eyre!(e)))?;
    Ok((client, headers))
}

/// Loads native root certificates from the OS certificate store.
/// This should be called once and the result cached/shared.
fn load_native_root_certs() -> Result<rustls::RootCertStore, ApiError> {
    let mut root_store = rustls::RootCertStore::empty();
    for cert in rustls_native_certs::load_native_certs().certs {
        root_store
            .add(cert)
            .map_err(|e| ApiError::EyreError(eyre::eyre!(e)))?;
    }
    Ok(root_store)
}

/// Builds a hyper HTTPS client, optionally with client certificate authentication.
fn build_https_client(
    client_cert: Option<&secret::PemIdentity>,
    root_cert_store: &Arc<rustls::RootCertStore>,
) -> eyre::Result<HyperClient> {
    let tls_config = build_tls_config(client_cert, root_cert_store)?;

    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_tls_config(tls_config)
        .https_or_http()
        .enable_http1()
        .enable_http2()
        .build();

    let client = hyper_util::client::legacy::Client::builder(hyper_util::rt::TokioExecutor::new())
        .pool_idle_timeout(Duration::from_secs(90))
        .pool_max_idle_per_host(4)
        .build(https_connector);

    Ok(client)
}

/// Builds a rustls ClientConfig, optionally with client certificate for mTLS.
fn build_tls_config(
    client_cert: Option<&secret::PemIdentity>,
    root_cert_store: &Arc<rustls::RootCertStore>,
) -> eyre::Result<rustls::ClientConfig> {
    let builder =
        rustls::ClientConfig::builder().with_root_certificates(Arc::clone(root_cert_store));

    let config = if let Some(identity) = client_cert {
        builder.with_client_auth_cert(identity.certs.clone(), identity.key.clone_key())?
    } else {
        builder.with_no_client_auth()
    };

    Ok(config)
}
