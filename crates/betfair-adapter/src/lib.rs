#![warn(missing_docs, unreachable_pub, unused_crate_dependencies)]
#![deny(unused_must_use, rust_2018_idioms)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

mod config;
mod error;
mod provider;
mod secret;
mod urls;

pub use betfair_types;
pub use betfair_types::rust_decimal;
pub use config::BetfairConfigBuilder;
pub use error::ApiError;
pub use provider::authenticated::{BetfairRequest, BetfairResponse};
pub use provider::{AuthenticatedBetfairRpcProvider, UnauthenticatedBetfairRpcProvider};
pub use secret::{ApplicationKey, Identity, Password, SecretProvider, SessionToken, Username};
pub use urls::{
    jurisdiction, BetfairUrl, BotLogin, InteractiveLogin, KeepAlive, Logout, RestBase, RetrieveUrl,
    Stream,
};
