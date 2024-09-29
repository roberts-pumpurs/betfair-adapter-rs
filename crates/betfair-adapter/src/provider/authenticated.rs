use betfair_types::keep_alive;
use betfair_types::types::BetfairRpcRequest;

use crate::{ApiError, AuthenticatedBetfairRpcProvider};

impl AuthenticatedBetfairRpcProvider {
    
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
        let endpoint = self.base.rest_base.url().join(T::method())?;
        let full = self
            .authenticated_client
            .post(endpoint.as_str())
            .json(&request)
            .send()
            .await?;
        tracing::debug!("Response status: {:?}", full.status());
        tracing::debug!("Response headers: {:?}", full.headers());
    
        if full.status().is_success() {
            let text = full.text().await?;
            tracing::debug!("Response body: {}", text);
            if text.trim().is_empty() {
                tracing::warn!("Received empty response body");
                return Err(ApiError::EmptyResponse);
            }
            let res = serde_json::from_str::<T::Res>(&text)?;
            Ok(res)
        } else {
            let text = full.text().await?;
            tracing::error!("Error response body: {}", text);
            let res = serde_json::from_str::<T::Error>(&text)?;
            Err(res.into())
        }
    }

    /// You can use Keep Alive to extend the session timeout period. The minimum session time is
    /// currently 20 minutes (Italian Exchange). On the international (.com) Exchange the current
    /// session time is 24 hours. Therefore, you should request Keep Alive within this time to
    /// prevent session expiry. If you don't call Keep Alive within the specified timeout period,
    /// the session will expire. Session times aren't determined or extended based on API activity.
    #[tracing::instrument(skip_all, ret, err)]
    pub async fn keep_alive(&self) -> Result<(), ApiError> {
        let _res = self
            .authenticated_client
            .get(self.base.keep_alive.url().clone())
            .send()
            .await?
            .error_for_status()?
            .json::<keep_alive::Response>()
            .await?
            .0
            .map_err(ApiError::KeepAliveError)?;

        Ok(())
    }

    /// You can use Logout to terminate your existing session.
    #[tracing::instrument(skip_all, ret, err)]
    pub async fn logout(&self) -> Result<(), ApiError> {
        let _res = self
            .authenticated_client
            .get(self.base.logout.url().clone())
            .send()
            .await?
            .error_for_status()?
            .json::<keep_alive::Response>()
            .await?
            .0
            .map_err(ApiError::LogoutError)?;

        Ok(())
    }

    pub async fn update_auth_token(&mut self) -> Result<(), ApiError> {
        let (session_token, authenticated_client) = self.base.bot_log_in().await?;
        self.session_token = session_token;
        self.authenticated_client = authenticated_client;
        Ok(())
    }
}
