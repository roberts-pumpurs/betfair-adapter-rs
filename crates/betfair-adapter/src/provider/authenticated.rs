use core::marker::PhantomData;

use betfair_types::keep_alive;
use betfair_types::types::BetfairRpcRequest;
use bytes::Bytes;
use http_body_util::{BodyExt, Full};
use tracing::instrument;

use super::HyperClient;
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
        let uri: http::Uri = endpoint
            .as_str()
            .parse()
            .map_err(|e: http::uri::InvalidUri| eyre::eyre!(e))?;

        let mut buf = Vec::with_capacity(256);
        serde_json::to_writer(&mut buf, &request)?;

        let mut request_builder = http::Request::builder().method(http::Method::POST).uri(uri);

        for (key, value) in &self.state.default_headers {
            request_builder = request_builder.header(key, value);
        }

        let http_request = request_builder.body(Full::new(Bytes::from(buf)))?;

        let response = self
            .state
            .authenticated_client
            .request(http_request)
            .await?;
        let status = response.status();
        let body = response.into_body().collect().await.map_err(|e| {
            ApiError::EyreError(eyre::eyre!("failed to collect response body: {e}"))
        })?;
        let bytes = body.to_bytes();

        if status.is_success() {
            if bytes.is_empty() {
                tracing::warn!("Received empty response body");
                return Err(ApiError::EmptyResponse);
            }
            let res = serde_json::from_slice::<T::Res>(&bytes)?;
            Ok(res)
        } else {
            let res = serde_json::from_slice::<T::Error>(&bytes)?;
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
        let uri: http::Uri = endpoint
            .as_str()
            .parse()
            .map_err(|e: http::uri::InvalidUri| eyre::eyre!(e))?;
        let client = self.state.authenticated_client.clone();

        let mut buf = Vec::with_capacity(256);
        serde_json::to_writer(&mut buf, &request)?;

        let mut request_builder = http::Request::builder().method(http::Method::POST).uri(uri);

        for (key, value) in &self.state.default_headers {
            request_builder = request_builder.header(key, value);
        }

        let http_request = request_builder.body(Full::new(Bytes::from(buf)))?;

        Ok(BetfairRequest {
            request: http_request,
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
        let uri: http::Uri = endpoint
            .as_str()
            .parse()
            .map_err(|e: http::uri::InvalidUri| eyre::eyre!(e))?;
        let client = self.state.authenticated_client.clone();

        let mut request_builder = http::Request::builder().method(http::Method::GET).uri(uri);

        for (key, value) in &self.state.default_headers {
            request_builder = request_builder.header(key, value);
        }

        let http_request = request_builder.body(Full::new(Bytes::new()))?;

        Ok(BetfairRequest {
            request: http_request,
            client,
            result: PhantomData,
            err: PhantomData,
        })
    }

    /// You can use Logout to terminate your existing session.
    #[tracing::instrument(skip_all, ret, err)]
    pub fn logout(&self) -> Result<BetfairRequest<keep_alive::Response, ()>, ApiError> {
        let endpoint = self.logout.url();
        let uri: http::Uri = endpoint
            .as_str()
            .parse()
            .map_err(|e: http::uri::InvalidUri| eyre::eyre!(e))?;
        let client = self.state.authenticated_client.clone();

        let mut request_builder = http::Request::builder().method(http::Method::GET).uri(uri);

        for (key, value) in &self.state.default_headers {
            request_builder = request_builder.header(key, value);
        }

        let http_request = request_builder.body(Full::new(Bytes::new()))?;

        Ok(BetfairRequest {
            request: http_request,
            client,
            result: PhantomData,
            err: PhantomData,
        })
    }
}

/// Encapsulated HTTP request for the Betfair API
pub struct BetfairRequest<T, E> {
    request: http::Request<Full<Bytes>>,
    client: HyperClient,
    result: PhantomData<T>,
    err: PhantomData<E>,
}

impl<T, E> core::fmt::Debug for BetfairRequest<T, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BetfairRequest")
            .field("method", self.request.method())
            .field("uri", self.request.uri())
            .finish()
    }
}

impl<T, E> BetfairRequest<T, E> {
    /// execute a Betfair API request
    #[instrument(name = "execute_request", skip(self), fields(method = %self.request.method(), url = %self.request.uri()))]
    pub async fn execute(self) -> Result<BetfairResponse<T, E>, ApiError> {
        let response = self.client.request(self.request).await?;

        // Capture the current span
        let span = tracing::Span::current();

        let status = response.status();
        let body = response.into_body().collect().await.map_err(|e| {
            ApiError::EyreError(eyre::eyre!("failed to collect response body: {e}"))
        })?;
        let bytes = body.to_bytes();

        Ok(BetfairResponse {
            status,
            bytes,
            result: PhantomData,
            err: PhantomData,
            span,
        })
    }
}

/// The raw response of the Betfair API request
pub struct BetfairResponse<T, E> {
    status: http::StatusCode,
    bytes: Bytes,
    result: PhantomData<T>,
    err: PhantomData<E>,
    // this span carries the context of the `BetfairRequest`
    span: tracing::Span,
}

impl<T, E> core::fmt::Debug for BetfairResponse<T, E> {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("BetfairResponse")
            .field("status", &self.status)
            .field("bytes_len", &self.bytes.len())
            .finish()
    }
}

impl<T, E> BetfairResponse<T, E> {
    /// Only check if the returned HTTP response is of error type; don't parse the data
    ///
    /// Useful when you don't care about the actual response besides if it was an error.
    #[instrument(name = "response_ok", skip(self), err, parent = &self.span)]
    pub fn ok(self) -> Result<(), ApiError> {
        if self.status.is_success() {
            Ok(())
        } else {
            Err(ApiError::HttpStatus(self.status))
        }
    }

    /// Check if the returned HTTP result is an error;
    /// Only parse the error type if we received an error.
    ///
    /// Useful when you don't care about the actual response besides if it was an error.
    #[instrument(name = "parse_response_json_err", skip(self), err, parent = &self.span)]
    pub fn json_err(self) -> Result<Result<(), E>, ApiError>
    where
        E: serde::de::DeserializeOwned,
    {
        if self.status.is_success() {
            Ok(Ok(()))
        } else {
            let res = parse_betfair_error::<E>(&self.bytes, self.status)?;
            Ok(Err(res))
        }
    }

    /// Parse the response json
    #[instrument(name = "parse_response_json", skip(self), err, parent = &self.span)]
    pub fn json(self) -> Result<Result<T, E>, ApiError>
    where
        T: serde::de::DeserializeOwned,
        E: serde::de::DeserializeOwned,
    {
        if self.status.is_success() {
            let json = String::from_utf8_lossy(self.bytes.as_ref());
            tracing::debug!(response_body = %json, "Response JSON");

            let res = serde_json::from_slice::<T>(&self.bytes)?;
            Ok(Ok(res))
        } else {
            let res = parse_betfair_error::<E>(&self.bytes, self.status)?;
            Ok(Err(res))
        }
    }
}

fn parse_betfair_error<E>(bytes: &[u8], status: http::StatusCode) -> Result<E, ApiError>
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
