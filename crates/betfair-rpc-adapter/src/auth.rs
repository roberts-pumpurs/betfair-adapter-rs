use std::marker::PhantomData;

use hyper::header;
use redact::Secret;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::{
    ApiError, ApplicationKey, Authenticated, BetfairRpcProvider, Identity, Unauthenticated,
};

impl<'a> BetfairRpcProvider<'a, Unauthenticated> {
    pub fn new_with_urls(
        rest_base: crate::urls::BetfairUrl<'a, crate::urls::RestBase>,
        keep_alive: crate::urls::BetfairUrl<'a, crate::urls::KeepAlive>,
        cert_login: crate::urls::BetfairUrl<'a, crate::urls::CertLogin>,
        secret_provider: crate::secret::SecretProvider<'a>,
    ) -> Self {
        Self {
            client: Client::new(),
            _type: PhantomData,
            rest_base,
            keep_alive,
            cert_login,
            secret_provider,
        }
    }

    pub fn new(secret_provider: crate::secret::SecretProvider<'a>) -> Self {
        Self::new_with_urls(
            crate::urls::BetfairUrl::default(),
            crate::urls::BetfairUrl::default(),
            crate::urls::BetfairUrl::default(),
            secret_provider,
        )
    }

    pub async fn authenticate(mut self) -> Result<BetfairRpcProvider<'a, Authenticated>, ApiError> {
        self.cert_log_in().await?;

        let instance: BetfairRpcProvider<Authenticated> = unsafe { std::mem::transmute(self) };

        Ok(instance)
    }
}

impl<'a> BetfairRpcProvider<'a, Authenticated> {
    pub async fn update_auth_token(&mut self) -> Result<(), ApiError> {
        self.cert_log_in().await?;
        Ok(())
    }
}

impl<'a, T> BetfairRpcProvider<'a, T> {
    // #[tracing::instrument(skip(self), err)]
    async fn cert_log_in(&mut self) -> Result<(), ApiError> {
        // Use this to get the session token
        let temp_client = login_client(
            &self.secret_provider.application_key,
            &self.secret_provider.identity,
        )?;

        let login_response = temp_client
            .post(self.cert_login.as_str())
            .form(&[
                ("username", self.secret_provider.username.0.expose_secret()),
                ("password", self.secret_provider.password.0.expose_secret()),
            ])
            .send()
            .await?
            .json::<SessionTokenInfo>()
            .await?;

        self.client = default_client(
            &self.secret_provider.application_key.0,
            &login_response.session_token,
        )?;

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

pub fn default_client(
    app_key: &Secret<String>,
    session_token: &Secret<String>,
) -> eyre::Result<reqwest::Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert(
        "X-Application",
        header::HeaderValue::from_str(app_key.expose_secret().as_str())?,
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
        header::HeaderValue::from_str(session_token.expose_secret().as_str())?,
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
