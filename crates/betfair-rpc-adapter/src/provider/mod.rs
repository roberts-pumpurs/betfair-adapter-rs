mod authenticated;
mod unauthenticated;

use std::marker::PhantomData;

use crate::{secret, urls};

#[derive(Debug)]
pub struct BetfairRpcProvider<'a, T> {
    client: reqwest::Client,
    rest_base: urls::BetfairUrl<'a, urls::RestBase>,
    keep_alive: urls::BetfairUrl<'a, urls::KeepAlive>,
    bot_login: urls::BetfairUrl<'a, urls::BotLogin>,
    logout: urls::BetfairUrl<'a, urls::Logout>,
    login: urls::BetfairUrl<'a, urls::InteractiveLogin>,
    secret_provider: secret::SecretProvider<'a>,
    _type: PhantomData<T>,
}

pub struct Authenticated;
pub struct Unauthenticated;
