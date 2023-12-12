pub mod auth;
mod error;
mod secret;
pub mod urls;

use std::marker::PhantomData;

pub use betfair_types;
pub use betfair_types::rust_decimal;
use betfair_types::types::BetfairRpcRequest;
pub use error::ApiError;
use error::ApingException;
use reqwest::Client;
pub use secret::{ApplicationKey, Identity, Password, SecretProvider, Username};
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::{OnceCell, RwLock};

pub struct Authenticated;
pub struct Unauthenticated;

#[derive(Debug)]
pub struct BetfairRpcProvider<'a, T> {
    client: Client,
    rest_base: urls::BetfairUrl<'a, urls::RestBase>,
    keep_alive: urls::BetfairUrl<'a, urls::KeepAlive>,
    cert_login: urls::BetfairUrl<'a, urls::CertLogin>,
    auth_token: redact::Secret<String>,
    secret_provider: secret::SecretProvider<'a>,
    _type: PhantomData<T>,
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
        let endpoint = self.rest_base.clone().join(T::method())?;
        tracing::info!(endpoint = ?endpoint.to_string(), "Sending request");
        let res = self
            .client
            .post(endpoint.as_str())
            .header(auth::AUTH_HEADER, self.auth_token.expose_secret().as_str())
            .json(&request);

        let res = res.send().await?;
        let full = res.bytes().await?;
        let res = serde_json::from_slice::<T::Res>(&full);

        match res {
            Ok(res) => return Ok(res),
            Err(_) => {
                let res = serde_json::from_slice::<T::Error>(&full)
                    .map_err(|e| eyre::eyre!(format!("Response not valid JSON: {:?} {e}", full)))?;
                return Err(res.into())
            }
        };
    }
}

#[derive(serde::Deserialize)]
enum Response<T, E> {
    Ok(T),
    Err(E),
}
