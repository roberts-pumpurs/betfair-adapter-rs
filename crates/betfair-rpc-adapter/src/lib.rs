pub mod auth;
mod error;
mod secret;
pub mod urls;

use std::marker::PhantomData;

pub use betfair_types;
use betfair_types::types::{BetfairRpcRequest, TransportLayer};
pub use error::ApiError;
use error::ApingException;
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::sync::RwLock;

pub struct Authenticated;
pub struct Unauthenticated;

#[derive(Debug)]
pub struct BetfairRpcProvider<T> {
    client: Client,
    rest_base: urls::RestBase,
    keep_alive: urls::KeepAlive,
    cert_login: urls::CertLogin,
    auth_token: RwLock<redact::Secret<String>>,
    secret_provider: secret::SecretProvider,
    _type: PhantomData<T>,
}

impl<T> TransportLayer<T> for BetfairRpcProvider<Authenticated>
where
    T: BetfairRpcRequest + Serialize + std::marker::Send + 'static + std::fmt::Debug,
    T::Res: DeserializeOwned + std::marker::Send,
    T::Error: DeserializeOwned + Into<ApiError> + std::fmt::Debug + error::ApingException,
{
    type Error = ApiError;

    #[tracing::instrument(skip(self), err)]
    async fn send_request(&self, request: T) -> Result<T::Res, Self::Error> {
        let endpoint = self.rest_base.0.join(T::method())?;
        let auth_token = self.auth_token.read().await;
        let res = self
            .client
            .post(endpoint.as_str())
            .header(auth::AUTH_HEADER, auth_token.expose_secret().as_str())
            .json(&request);
        let res = res.send().await?;
        let res = res.text().await?;
        let res = serde_json::from_str(&res).map_err(|_e| {
            serde_json::from_str::<T::Error>(&res)
                .map::<(bool, bool, ApiError), _>(|x| {
                    (x.invalid_app_key(), x.invalid_session(), x.into())
                })
                .map_err(ApiError::SerdeError)
        });
        match res {
            Err(Ok((invalid_app, invalid_session, err_res))) => {
                // TODO we need to extract the error type and see if it's a session token error. If
                // it is, we want to completely re-login into the app TODO:    it
                // can be attempted by calling the login method again and updating
                // the token.
                if invalid_app || invalid_session {
                    self.update_auth_token().await?;
                }
                return Err(err_res)
            }
            Ok(asd) => return Ok(asd),
            Err(Err(err)) => return Err(err),
        };
    }
}
