use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct BetfairUrl<'a, T> {
    url: Cow<'a, url::Url>,
    _type: std::marker::PhantomData<T>,
}

impl<'a, T> BetfairUrl<'a, T> {
    pub fn new(url: Cow<'a, url::Url>) -> Self {
        Self {
            url,
            _type: std::marker::PhantomData,
        }
    }

    pub fn url(&self) -> &url::Url {
        &self.url
    }
}

#[derive(Debug, Clone)]
pub struct RestBase;

#[derive(Debug, Clone)]
pub struct KeepAlive;

#[derive(Debug, Clone)]
pub struct BotLogin;

#[derive(Debug, Clone)]
pub struct InteractiveLogin;

#[derive(Debug, Clone)]
pub struct Logout;

pub mod jurisdictions {
    #[derive(Debug)]
    pub struct Global;
    #[derive(Debug)]
    pub struct Italy;
    #[derive(Debug)]
    pub struct Spain;
    #[derive(Debug)]
    pub struct Romania;
    #[derive(Debug)]
    pub struct Sweden;
    #[derive(Debug)]
    pub struct Australia;
}

pub trait RetrieveUrl<'a, T> {
    fn url(&self) -> BetfairUrl<'a, T>;
}

pub mod rest_url {
    use super::*;

    impl<'a> RetrieveUrl<'a, RestBase> for jurisdictions::Global {
        fn url(&self) -> BetfairUrl<'a, RestBase> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://api.betfair.com/exchange/betting/rest/v1.0").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, RestBase> for jurisdictions::Italy {
        fn url(&self) -> BetfairUrl<'a, RestBase> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://api.betfair.it/exchange/betting/rest/v1.0").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, RestBase> for jurisdictions::Spain {
        fn url(&self) -> BetfairUrl<'a, RestBase> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://api.betfair.com/exchange/betting/rest/v1.0").unwrap(),
            ))
        }
    }
}

pub mod keep_alive_url {
    use super::*;

    impl<'a> RetrieveUrl<'a, KeepAlive> for jurisdictions::Global {
        fn url(&self) -> BetfairUrl<'a, KeepAlive> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.com/api/keepAlive").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, KeepAlive> for jurisdictions::Italy {
        fn url(&self) -> BetfairUrl<'a, KeepAlive> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.it/api/keepAlive").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, KeepAlive> for jurisdictions::Spain {
        fn url(&self) -> BetfairUrl<'a, KeepAlive> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.es/api/keepAlive").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, KeepAlive> for jurisdictions::Romania {
        fn url(&self) -> BetfairUrl<'a, KeepAlive> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.ro/api/keepAlive").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, KeepAlive> for jurisdictions::Sweden {
        fn url(&self) -> BetfairUrl<'a, KeepAlive> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.se/api/keepAlive").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, KeepAlive> for jurisdictions::Australia {
        fn url(&self) -> BetfairUrl<'a, KeepAlive> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.au/api/keepAlive").unwrap(),
            ))
        }
    }
}

pub mod bot_login_url {
    use super::*;

    impl<'a> RetrieveUrl<'a, BotLogin> for jurisdictions::Global {
        fn url(&self) -> BetfairUrl<'a, BotLogin> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso-cert.betfair.com/api/certlogin").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, BotLogin> for jurisdictions::Italy {
        fn url(&self) -> BetfairUrl<'a, BotLogin> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso-cert.betfair.it/api/certlogin").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, BotLogin> for jurisdictions::Spain {
        fn url(&self) -> BetfairUrl<'a, BotLogin> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso-cert.betfair.es/api/certlogin").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, BotLogin> for jurisdictions::Romania {
        fn url(&self) -> BetfairUrl<'a, BotLogin> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso-cert.betfair.ro/api/certlogin").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, BotLogin> for jurisdictions::Sweden {
        fn url(&self) -> BetfairUrl<'a, BotLogin> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso-cert.betfair.se/api/certlogin").unwrap(),
            ))
        }
    }
}

pub mod interactive_login_url {
    use super::*;

    impl<'a> RetrieveUrl<'a, InteractiveLogin> for jurisdictions::Global {
        fn url(&self) -> BetfairUrl<'a, InteractiveLogin> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.com/api/login").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, InteractiveLogin> for jurisdictions::Italy {
        fn url(&self) -> BetfairUrl<'a, InteractiveLogin> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.it/api/login").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, InteractiveLogin> for jurisdictions::Spain {
        fn url(&self) -> BetfairUrl<'a, InteractiveLogin> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.es/api/login").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, InteractiveLogin> for jurisdictions::Romania {
        fn url(&self) -> BetfairUrl<'a, InteractiveLogin> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.ro/api/login").unwrap(),
            ))
        }
    }

    impl<'a> RetrieveUrl<'a, InteractiveLogin> for jurisdictions::Sweden {
        fn url(&self) -> BetfairUrl<'a, InteractiveLogin> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.se/api/login").unwrap(),
            ))
        }
    }
}

pub mod logout_url {
    use super::*;

    impl<'a> RetrieveUrl<'a, Logout> for jurisdictions::Global {
        fn url(&self) -> BetfairUrl<'a, Logout> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.com/api/logout").unwrap(),
            ))
        }
    }
}
