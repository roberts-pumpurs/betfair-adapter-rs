use betfair_types::keep_alive;
use betfair_types::types::BetfairRpcRequest;

use crate::{ApiError, AuthenticatedBetfairRpcProvider};

impl AuthenticatedBetfairRpcProvider {
    #[tracing::instrument(skip_all, ret, err, fields(req = ?request))]
    pub async fn send_request<T>(&self, request: T) -> Result<T::Res, ApiError>
    where
        T: BetfairRpcRequest + serde::Serialize + std::fmt::Debug,
        T::Res: serde::de::DeserializeOwned + std::fmt::Debug,
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

        if full.status().is_success() {
            let res = full.json::<T::Res>().await?;
            return Ok(res)
        } else {
            let res = full.json::<T::Error>().await?;
            return Err(res.into())
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
