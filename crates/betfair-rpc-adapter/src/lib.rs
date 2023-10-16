pub mod auth;
mod error;
pub mod urls;

use std::collections::HashMap;
use std::marker::PhantomData;

pub use betfair_types;
use betfair_types::types::{BetfairRpcRequest, TransportLayer};
pub use error::ApiError;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct Authenticated;
pub struct Unauthenticated;

#[derive(Debug)]
pub struct BetfairRpcProvider<T> {
    client: Client,
    rest_base: urls::RestBase,
    keep_alive: urls::KeepAlive,
    cert_login: urls::CertLogin,
    auth_token: redact::Secret<String>,
    _type: PhantomData<T>,
}

impl<T> TransportLayer<T> for BetfairRpcProvider<Authenticated>
where
    T: BetfairRpcRequest + Serialize + std::marker::Send + 'static + std::fmt::Debug,
    T::Res: DeserializeOwned + 'static,
    T::Error: DeserializeOwned + 'static + Into<ApiError> + std::fmt::Debug,
{
    type Error = ApiError;

    #[tracing::instrument(skip(self), err)]
    async fn send_request(&self, request: T) -> Result<T::Res, Self::Error> {
        let endpoint = self.rest_base.0.join(T::method())?;
        let res = self
            .client
            .post(endpoint.as_str())
            .header("X-Authentication", self.auth_token.expose_secret().as_str())
            .json(&request);
        let res = res.send().await?;
        let res = res.text().await?;
        let res = serde_json::from_str(&res).map_err(|_e| {
            let err_res = serde_json::from_str::<T::Error>(&res).map(|x| x.into());
            match err_res {
                Ok(err_res) => err_res,
                Err(e) => ApiError::SerdeError(e),
            }
        })?;
        Ok(res)
    }
}
