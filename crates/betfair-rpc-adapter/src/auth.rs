use std::collections::HashMap;
use std::marker::PhantomData;

use hyper::header;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;

use crate::{ApiError, Authenticated, BetfairRpcProvider, Unauthenticated};

pub const AUTH_HEADER: &str = "X-Authentication";

impl Default for BetfairRpcProvider<Unauthenticated> {
    fn default() -> Self {
        Self::new()
    }
}

impl BetfairRpcProvider<Unauthenticated> {
    pub fn new_with_urls(
        rest_base: crate::urls::RestBase,
        keep_alive: crate::urls::KeepAlive,
        cert_login: crate::urls::CertLogin,
    ) -> Self {
        Self {
            client: Client::new(),
            _type: PhantomData,
            rest_base,
            keep_alive,
            cert_login,
            auth_token: RwLock::new(redact::Secret::new("".to_string())),
            secret_provider: crate::secret::SecretProvider,
        }
    }

    pub fn new() -> Self {
        Self::new_with_urls(
            crate::urls::RestBase::default(),
            crate::urls::KeepAlive::default(),
            crate::urls::CertLogin::default(),
        )
    }

    pub async fn authenticate(self) -> Result<BetfairRpcProvider<Authenticated>, ApiError> {
        self.cert_log_in().await?;

        let instance: BetfairRpcProvider<Authenticated> = unsafe { std::mem::transmute(self) };

        Ok(instance)
    }
}

impl BetfairRpcProvider<Authenticated> {
    pub async fn update_auth_token(&self) -> Result<(), ApiError> {
        self.cert_log_in().await?;
        Ok(())
    }
}

impl<T> BetfairRpcProvider<T> {
    #[tracing::instrument(skip(self), err)]
    async fn cert_log_in(&self) -> Result<(), ApiError> {
        const KEEP_ALIVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);

        // Use this to get the session token
        let temp_client = reqwest::Client::builder()
            .use_rustls_tls()
            .identity(self.secret_provider.identity().0.expose_secret().clone())
            .default_headers(header_tuple_to_map([
                (
                    "X-Application".to_string(),
                    self.secret_provider.application_key().0.expose_secret().to_string(),
                ),
                ("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string()),
            ]))
            .http2_keep_alive_interval(KEEP_ALIVE_INTERVAL)
            .http2_keep_alive_while_idle(true)
            .build()?;

        let login_response = temp_client
            .post(self.cert_login.0.as_str())
            .form(&[
                ("username", self.secret_provider.username().0.expose_secret().to_owned()),
                ("password", self.secret_provider.password().0.expose_secret().to_owned()),
            ])
            .send()
            .await?
            .json::<SessionTokenInfo>()
            .await?;

        let mut w = self.auth_token.write().await;
        *w = login_response.session_token.clone();

        // TODO keep this loop running in the background and close on error
        let handle = tokio::spawn(async {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(60)).await;
                if let Err(e) = keep_alive().await {
                    tracing::error!("Error keeping alive: {:?}", e);
                    break
                }
            }
        });
        // TODO we need to abort the handle when we drop the client. Maybe we can use JoinSet and push the handle in there

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionTokenInfo {
    #[serde(serialize_with = "redact::expose_secret")]
    session_token: redact::Secret<String>,
    login_status: String,
}

fn header_tuple_to_map<const SIZE: usize>(
    header_keys: [(String, String); SIZE],
) -> header::HeaderMap {
    let map = HashMap::from(header_keys);
    let headers: header::HeaderMap = (&map).try_into().expect("valid headers");
    headers
}

pub fn default_client(app_key: String) -> eyre::Result<reqwest::Client> {
    let headers = header_tuple_to_map([
        ("X-Application".to_string(), app_key),
        ("Accept".to_string(), "application/json".to_string()),
        ("Content-type".to_string(), "application/json".to_string()),
        ("Accept-Encoding".to_string(), "gzip, deflate".to_string()),
    ]);
    Ok(reqwest::Client::builder().default_headers(headers).build()?)
}

pub async fn keep_alive() -> Result<(), ApiError> {
    // TODO get everything from args
    let auth_token = self.auth_token.read().await;
    let _res = self
        .client
        .get(self.keep_alive.0.as_str())
        .header(AUTH_HEADER, auth_token.expose_secret().as_str())
        .send()
        .await?;

    Ok(())
}
