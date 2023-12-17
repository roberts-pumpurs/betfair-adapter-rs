mod config;
mod error;
mod provider;
mod secret;
mod urls;

pub use betfair_types;
pub use betfair_types::rust_decimal;
pub use config::{new_global_config, BetfairConfigBuilder};
pub use error::ApiError;
pub use provider::{Authenticated, BetfairRpcProvider, Unauthenticated};
pub use secret::{ApplicationKey, Identity, Password, SecretProvider, SessionToken, Username};
pub use urls::{
    jurisdictions, BetfairUrl, BotLogin, InteractiveLogin, KeepAlive, Logout, RestBase,
    RetrieveUrl, Stream,
};
