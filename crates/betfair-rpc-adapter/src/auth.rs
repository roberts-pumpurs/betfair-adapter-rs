use std::collections::HashMap;
use std::marker::PhantomData;

use hyper::header;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{ApiError, Authenticated, BetfairRpcProvider, Unauthenticated};

#[derive(Debug)]
pub struct ApplicationKey(redact::Secret<String>);

#[derive(Debug)]
pub struct Username(redact::Secret<String>);

#[derive(Debug)]
pub struct Password(redact::Secret<String>);

#[derive(Debug)]
pub struct Identity(redact::Secret<reqwest::Identity>);

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
            auth_token: redact::Secret::new("".to_string()),
        }
    }

    pub fn new() -> Self {
        Self::new_with_urls(
            crate::urls::RestBase::default(),
            crate::urls::KeepAlive::default(),
            crate::urls::CertLogin::default(),
        )
    }

    pub async fn authenticate(
        mut self,
        application_key: ApplicationKey,
        username: Username,
        password: Password,
        identity: Identity,
    ) -> Result<BetfairRpcProvider<Authenticated>, ApiError> {
        self.cert_log_in(application_key, username, password, identity).await?;

        Ok(BetfairRpcProvider::<Authenticated>::new_with_client(
            self.client,
            self.rest_base,
            self.keep_alive,
            self.cert_login,
            self.auth_token,
        ))
    }
}

impl BetfairRpcProvider<Authenticated> {
    pub(crate) fn new_with_client(
        client: Client,
        rest_base: crate::urls::RestBase,
        keep_alive: crate::urls::KeepAlive,
        cert_login: crate::urls::CertLogin,
        auth_token: redact::Secret<String>,
    ) -> Self {
        Self { client, _type: PhantomData, rest_base, keep_alive, cert_login, auth_token }
    }
}

impl<T> BetfairRpcProvider<T> {
    pub(crate) async fn cert_log_in(
        &mut self,
        application_key: ApplicationKey,
        username: Username,
        password: Password,
        identity: Identity,
    ) -> Result<(), ApiError> {
        const KEEP_ALIVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);

        // Use this to get the session token
        let temp_client = reqwest::Client::builder()
            .use_rustls_tls()
            .identity(identity.0.expose_secret().clone())
            .default_headers(header_tuple_to_map([
                ("X-Application".to_string(), application_key.0.expose_secret().to_string()),
                ("Content-Type".to_string(), "application/x-www-form-urlencoded".to_string()),
            ]))
            .http2_keep_alive_interval(KEEP_ALIVE_INTERVAL)
            .http2_keep_alive_while_idle(true)
            .build()?;

        let login_response = temp_client
            .post(self.cert_login.0.as_str())
            .form(&[
                ("username", username.0.expose_secret().to_owned()),
                ("password", password.0.expose_secret().to_owned()),
            ])
            .send()
            .await?
            .json::<SessionTokenInfo>()
            .await?;

        self.auth_token = login_response.session_token.clone();

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

impl ApplicationKey {
    pub fn new(application_key: String) -> Self {
        Self(redact::Secret::new(application_key))
    }
}

impl Username {
    pub fn new(username: String) -> Self {
        Self(redact::Secret::new(username))
    }
}

impl Password {
    pub fn new(password: String) -> Self {
        Self(redact::Secret::new(password))
    }
}

impl Identity {
    pub fn new(identity: reqwest::Identity) -> Self {
        Self(redact::Secret::new(identity))
    }
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
