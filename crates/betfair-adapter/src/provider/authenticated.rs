use core::marker::PhantomData;

use betfair_types::keep_alive;
use betfair_types::types::BetfairRpcRequest;
use tracing::instrument;

use crate::{ApiError, Authenticated, BetfairRpcClient};

impl BetfairRpcClient<Authenticated> {
    /// Sends a request and returns the response or an error.
    ///
    /// # Parameters
    /// - `request`: The request to be sent.
    ///
    /// # Returns
    /// A result containing either the response or an `ApiError`.
    #[tracing::instrument(skip_all, ret, err, fields(req = ?request))]
    pub async fn send_request<T>(&self, request: T) -> Result<T::Res, ApiError>
    where
        T: BetfairRpcRequest + serde::Serialize + core::fmt::Debug,
        T::Res: serde::de::DeserializeOwned + core::fmt::Debug,
        T::Error: serde::de::DeserializeOwned,
        ApiError: From<<T as BetfairRpcRequest>::Error>,
    {
        let endpoint = self.rest_base.url().join(T::method())?;
        let full = self
            .state
            .authenticated_client
            .post(endpoint.as_str())
            .json(&request)
            .send()
            .await?;

        if full.status().is_success() {
            let text = full.text().await?;
            if text.trim().is_empty() {
                tracing::warn!("Received empty response body");
                return Err(ApiError::EmptyResponse);
            }
            let res = serde_json::from_str::<T::Res>(&text)?;
            Ok(res)
        } else {
            let text = full.text().await?;
            let res = serde_json::from_str::<T::Error>(&text)?;
            Err(res.into())
        }
    }

    /// Create a request
    ///
    /// # Parameters
    /// - `request`: The request to be sent.
    ///
    /// # Returns
    /// A result containing either the response or an `ApiError`.
    pub fn build_request<T>(&self, request: T) -> Result<BetfairRequest<T::Res, T::Error>, ApiError>
    where
        T: BetfairRpcRequest + serde::Serialize + core::fmt::Debug,
        T::Res: serde::de::DeserializeOwned + core::fmt::Debug,
        T::Error: serde::de::DeserializeOwned,
    {
        let endpoint = self.rest_base.url().join(T::method())?;
        let client = self.state.authenticated_client.clone();
        let reqwest_req = client
            .request(reqwest::Method::POST, endpoint.as_str())
            .json(&request)
            .build()?;

        Ok(BetfairRequest {
            request: reqwest_req,
            client,
            result: PhantomData,
            err: PhantomData,
        })
    }

    /// You can use Keep Alive to extend the session timeout period. The minimum session time is
    /// currently 20 minutes (Italian Exchange). On the international (.com) Exchange the current
    /// session time is 24 hours. Therefore, you should request Keep Alive within this time to
    /// prevent session expiry. If you don't call Keep Alive within the specified timeout period,
    /// the session will expire. Session times aren't determined or extended based on API activity.
    #[tracing::instrument(skip_all, ret, err)]
    pub fn keep_alive(&self) -> Result<BetfairRequest<keep_alive::Response, ()>, ApiError> {
        let endpoint = self.keep_alive.url();
        let client = self.state.authenticated_client.clone();
        let reqwest_req = client
            .request(reqwest::Method::GET, endpoint.as_str())
            .build()?;

        Ok(BetfairRequest {
            request: reqwest_req,
            client,
            result: PhantomData,
            err: PhantomData,
        })
    }

    /// You can use Logout to terminate your existing session.
    #[tracing::instrument(skip_all, ret, err)]
    pub fn logout(&self) -> Result<BetfairRequest<keep_alive::Response, ()>, ApiError> {
        let endpoint = self.logout.url();
        let client = self.state.authenticated_client.clone();
        let reqwest_req = client
            .request(reqwest::Method::GET, endpoint.as_str())
            .build()?;

        Ok(BetfairRequest {
            request: reqwest_req,
            client,
            result: PhantomData,
            err: PhantomData,
        })
    }

    /// Update the internal client to get a new auth token
    pub async fn update_auth_token(&mut self) -> Result<(), ApiError> {
        let (session_token, authenticated_client) = self.bot_log_in().await?;
        self.state.session_token = session_token;
        self.state.authenticated_client = authenticated_client;
        Ok(())
    }
}

/// Encalpsulated HTTP request for the Betfair API
#[derive(Debug)]
pub struct BetfairRequest<T, E> {
    request: reqwest::Request,
    client: reqwest::Client,
    result: PhantomData<T>,
    err: PhantomData<E>,
}

impl<T, E> BetfairRequest<T, E> {
    /// execute an Amplifier API request
    #[instrument(name = "execute_request", skip(self), fields(method = %self.request.method(), url = %self.request.url()))]
    pub async fn execute(self) -> Result<BetfairResponse<T, E>, ApiError> {
        let response = self.client.execute(self.request).await?;

        // Capture the current span
        let span = tracing::Span::current();

        Ok(BetfairResponse {
            response,
            result: PhantomData,
            err: PhantomData,
            span,
        })
    }
}

/// The raw response of the Betfair API request
#[derive(Debug)]
pub struct BetfairResponse<T, E> {
    response: reqwest::Response,
    result: PhantomData<T>,
    err: PhantomData<E>,
    // this span carries the context of the `BetfairRequest`
    span: tracing::Span,
}

impl<T, E> BetfairResponse<T, E> {
    /// Only check if the returtned HTTP response is of error type; don't parse the data
    ///
    /// Useful when you don't care about the actual response besides if it was an error.
    #[instrument(name = "response_ok", skip(self), err, parent = &self.span)]
    pub fn ok(self) -> Result<(), ApiError> {
        self.response.error_for_status()?;
        Ok(())
    }

    /// Check if the returned HTTP result is an error;
    /// Only parse the error type if we received an error.
    ///
    /// Useful when you don't care about the actual response besides if it was an error.
    #[instrument(name = "parse_response_json_err", skip(self), err, parent = &self.span)]
    pub async fn json_err(self) -> Result<Result<(), E>, ApiError>
    where
        E: serde::de::DeserializeOwned,
    {
        let status = self.response.status();
        if status.is_success() {
            Ok(Ok(()))
        } else {
            let bytes = self.response.bytes().await?;
            let res = parse_betfair_error::<E>(&bytes, status)?;
            Ok(Err(res))
        }
    }

    /// Parse the response json
    #[instrument(name = "parse_response_json", skip(self), err, parent = &self.span)]
    pub async fn json(self) -> Result<Result<T, E>, ApiError>
    where
        T: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned,
    {
        let status = self.response.status();
        let bytes = self.response.bytes().await?;
        if status.is_success() {
            let json = String::from_utf8_lossy(bytes.as_ref());
            tracing::debug!(response_body = %json, "Response JSON");

            let res = serde_json::from_slice::<T>(&bytes)?;
            Ok(Ok(res))
        } else {
            let res = parse_betfair_error::<E>(&bytes, status)?;
            Ok(Err(res))
        }
    }
}

fn parse_betfair_error<E>(bytes: &[u8], status: reqwest::StatusCode) -> Result<E, ApiError>
where
    E: serde::de::DeserializeOwned,
{
    let json = String::from_utf8_lossy(bytes);
    tracing::error!(
        status = %status,
        body = %json,
        "Failed to execute request"
    );

    let error = serde_json::from_slice::<E>(bytes)?;
    Ok(error)
}
