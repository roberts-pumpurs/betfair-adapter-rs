use betfair_types::types::{BetfairRpcRequest, TransportLayer};
use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Serialize;

pub struct BetfairRpcProvider {
    client: Client,
}

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error(transparent)]
    SportsApingException(#[from] betfair_types::types::sports_aping::ApingException),
    #[error(transparent)]
    AccountApingException(#[from] betfair_types::types::account_aping::AccountApingException),
    HeartbeatApingException(#[from] betfair_types::types::heartbeat_aping::ApingException),
    #[error(transparent)]
    ReqwestError(#[from] reqwest::Error),
    #[error(transparent)]
    SerdeError(#[from] serde_json::Error),
}

impl<T> TransportLayer<T> for BetfairRpcProvider
where
    T: BetfairRpcRequest + Serialize + std::marker::Send + 'static,
    T::Res: DeserializeOwned + 'static,
    T::Error: DeserializeOwned + 'static + Into<ApiError>,
{
    type Error = ApiError;

    async fn send_request(&self, request: T) -> Result<T::Res, Self::Error> {
        let res = self.client.post("example.com").json(&request);
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

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ApiError::SportsApingException(e) => write!(f, "{}", e),
            ApiError::AccountApingException(e) => write!(f, "{}", e),
            ApiError::HeartbeatApingException(e) => write!(f, "{}", e),
            ApiError::ReqwestError(e) => write!(f, "{}", e),
            ApiError::SerdeError(e) => write!(f, "{}", e),
        }
    }
}

// async fn asd(client: &BetfairRpcProvider) {
//     let res = client.send_request(betfair_types::types::sports_aping::replace_orders::Parameters {
//         market_id: todo!(),
//         instructions: todo!(),
//         customer_ref: todo!(),
//         market_version: todo!(),
//         r_async: todo!(),
//     }).await;
//     match res {
//         Ok(res) => todo!(),
//         Err(e) => todo!(),
//     }
// }
