mod config;
mod error;
mod provider;
mod secret;
mod urls;

pub use betfair_types;
pub use config::BetfairConfigBuilder;
pub use error::ApiError;
pub use provider::authenticated::{BetfairRequest, BetfairResponse};
pub use provider::{Authenticated, BetfairRpcClient, Unauthenticated};
pub use secret::{ApplicationKey, Identity, Password, SecretProvider, SessionToken, Username};
pub use urls::{
    BetfairUrl, BotLogin, InteractiveLogin, KeepAlive, Logout, RestBase, RetrieveUrl, Stream,
    jurisdiction,
};
