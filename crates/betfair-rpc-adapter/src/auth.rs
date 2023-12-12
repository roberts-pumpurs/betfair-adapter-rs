use std::collections::HashMap;
use std::marker::PhantomData;

use hyper::header;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::{OnceCell, RwLock};

use crate::{ApiError, Authenticated, BetfairRpcProvider, Unauthenticated};

pub const AUTH_HEADER: &str = "X-Authentication";

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
            auth_token: redact::Secret::new("".to_string()),
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
    #[tracing::instrument(skip(self), err)]
    async fn cert_log_in(&mut self) -> Result<(), ApiError> {
        const KEEP_ALIVE_INTERVAL: std::time::Duration = std::time::Duration::from_secs(15);

        // Use this to get the session token
        let mut headers = header::HeaderMap::new();
        headers.insert(
            "X-Application",
            header::HeaderValue::from_str(
                self.secret_provider.application_key.0.expose_secret().as_str(),
            )
            .unwrap(),
        );
        headers.insert(
            "Content-Type",
            header::HeaderValue::from_static("application/x-www-form-urlencoded"),
        );

        let temp_client = reqwest::Client::builder()
            .use_rustls_tls()
            .identity(self.secret_provider.identity.0.expose_secret().clone())
            .default_headers(headers)
            .http2_keep_alive_interval(KEEP_ALIVE_INTERVAL)
            .http2_keep_alive_while_idle(true)
            .build()?;

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

        self.auth_token = login_response.session_token;
        self.client =
            default_client(self.secret_provider.application_key.0.expose_secret().clone())?;

        // TODO keep this loop running in the background and close on error
        // let handle = tokio::spawn(async {
        //     loop {
        //         tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        //         if let Err(e) = keep_alive().await {
        //             tracing::error!("Error keeping alive: {:?}", e);
        //             break
        //         }
        //     }
        // });
        // TODO we need to abort the handle when we drop the client. Maybe we can use JoinSet and
        // push the handle in there

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

pub fn default_client(app_key: String) -> eyre::Result<reqwest::Client> {
    let mut headers = header::HeaderMap::new();
    headers.insert("X-Application", header::HeaderValue::from_str(app_key.as_str()).unwrap());
    headers.insert("Accept", header::HeaderValue::from_static("application/json"));
    headers.insert("Accept-Encoding", header::HeaderValue::from_static("gzip, deflate"));
    headers.insert("Content-Type", header::HeaderValue::from_static("application/json"));
    Ok(reqwest::Client::builder().default_headers(headers).build()?)
}

// pub async fn keep_alive() -> Result<(), ApiError> {
//     // TODO get everything from args
//     let auth_token = self.auth_token.read().await;
//     let _res = self
//         .client
//         .get(self.keep_alive.0.as_str())
//         .header(AUTH_HEADER, auth_token.expose_secret().as_str())
//         .send()
//         .await?;

//     Ok(())
// }
