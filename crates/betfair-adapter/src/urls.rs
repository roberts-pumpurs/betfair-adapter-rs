#[derive(Debug, Clone)]
pub struct BetfairUrl<T> {
    url: url::Url,
    _type: core::marker::PhantomData<T>,
}

impl<T> BetfairUrl<T> {
    #[must_use]
    pub fn new(url: url::Url) -> Self {
        Self {
            url,
            _type: core::marker::PhantomData,
        }
    }

    #[must_use]
    pub fn url(&self) -> &url::Url {
        &self.url
    }
}

impl<T> From<url::Url> for BetfairUrl<T> {
    fn from(val: url::Url) -> Self {
        Self::new(val)
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

#[derive(Debug, Clone)]
pub struct Stream;

pub mod jurisdiction {
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

    #[derive(Debug, Clone)]
    pub struct CustomUrl<T>(pub super::BetfairUrl<T>);

    impl<T> CustomUrl<T> {
        #[must_use]
        pub fn new(url: url::Url) -> Self {
            Self(super::BetfairUrl::new(url))
        }
    }

    impl<T> From<url::Url> for CustomUrl<T> {
        fn from(value: url::Url) -> Self {
            Self::new(value)
        }
    }
}

pub trait RetrieveUrl<T> {
    fn url(&self) -> BetfairUrl<T>;
}

mod rest_url {
    use super::*;

    impl RetrieveUrl<RestBase> for jurisdiction::Global {
        fn url(&self) -> BetfairUrl<RestBase> {
            BetfairUrl::new(
                url::Url::parse("https://api.betfair.com/exchange/betting/rest/v1.0/").unwrap(),
            )
        }
    }

    impl RetrieveUrl<RestBase> for jurisdiction::Italy {
        fn url(&self) -> BetfairUrl<RestBase> {
            BetfairUrl::new(
                url::Url::parse("https://api.betfair.it/exchange/betting/rest/v1.0/").unwrap(),
            )
        }
    }

    impl RetrieveUrl<RestBase> for jurisdiction::Spain {
        fn url(&self) -> BetfairUrl<RestBase> {
            BetfairUrl::new(
                url::Url::parse("https://api.betfair.com/exchange/betting/rest/v1.0/").unwrap(),
            )
        }
    }

    impl RetrieveUrl<RestBase> for jurisdiction::CustomUrl<RestBase> {
        fn url(&self) -> BetfairUrl<RestBase> {
            self.0.clone()
        }
    }
}

mod keep_alive_url {
    use super::*;

    impl RetrieveUrl<KeepAlive> for jurisdiction::Global {
        fn url(&self) -> BetfairUrl<KeepAlive> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso.betfair.com/api/keepAlive").unwrap(),
            )
        }
    }

    impl RetrieveUrl<KeepAlive> for jurisdiction::Italy {
        fn url(&self) -> BetfairUrl<KeepAlive> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso.betfair.it/api/keepAlive").unwrap(),
            )
        }
    }

    impl RetrieveUrl<KeepAlive> for jurisdiction::Spain {
        fn url(&self) -> BetfairUrl<KeepAlive> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso.betfair.es/api/keepAlive").unwrap(),
            )
        }
    }

    impl RetrieveUrl<KeepAlive> for jurisdiction::Romania {
        fn url(&self) -> BetfairUrl<KeepAlive> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso.betfair.ro/api/keepAlive").unwrap(),
            )
        }
    }

    impl RetrieveUrl<KeepAlive> for jurisdiction::Sweden {
        fn url(&self) -> BetfairUrl<KeepAlive> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso.betfair.se/api/keepAlive").unwrap(),
            )
        }
    }

    impl RetrieveUrl<KeepAlive> for jurisdiction::Australia {
        fn url(&self) -> BetfairUrl<KeepAlive> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso.betfair.au/api/keepAlive").unwrap(),
            )
        }
    }

    impl RetrieveUrl<KeepAlive> for jurisdiction::CustomUrl<KeepAlive> {
        fn url(&self) -> BetfairUrl<KeepAlive> {
            self.0.clone()
        }
    }
}

mod bot_login_url {
    use super::*;

    impl RetrieveUrl<BotLogin> for jurisdiction::Global {
        fn url(&self) -> BetfairUrl<BotLogin> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso-cert.betfair.com/api/certlogin").unwrap(),
            )
        }
    }

    impl RetrieveUrl<BotLogin> for jurisdiction::Italy {
        fn url(&self) -> BetfairUrl<BotLogin> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso-cert.betfair.it/api/certlogin").unwrap(),
            )
        }
    }

    impl RetrieveUrl<BotLogin> for jurisdiction::Spain {
        fn url(&self) -> BetfairUrl<BotLogin> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso-cert.betfair.es/api/certlogin").unwrap(),
            )
        }
    }

    impl RetrieveUrl<BotLogin> for jurisdiction::Romania {
        fn url(&self) -> BetfairUrl<BotLogin> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso-cert.betfair.ro/api/certlogin").unwrap(),
            )
        }
    }

    impl RetrieveUrl<BotLogin> for jurisdiction::Sweden {
        fn url(&self) -> BetfairUrl<BotLogin> {
            BetfairUrl::new(
                url::Url::parse("https://identitysso-cert.betfair.se/api/certlogin").unwrap(),
            )
        }
    }

    impl RetrieveUrl<BotLogin> for jurisdiction::CustomUrl<BotLogin> {
        fn url(&self) -> BetfairUrl<BotLogin> {
            self.0.clone()
        }
    }
}

mod interactive_login_url {
    use super::*;

    impl RetrieveUrl<InteractiveLogin> for jurisdiction::Global {
        fn url(&self) -> BetfairUrl<InteractiveLogin> {
            BetfairUrl::new(url::Url::parse("https://identitysso.betfair.com/api/login").unwrap())
        }
    }

    impl RetrieveUrl<InteractiveLogin> for jurisdiction::Italy {
        fn url(&self) -> BetfairUrl<InteractiveLogin> {
            BetfairUrl::new(url::Url::parse("https://identitysso.betfair.it/api/login").unwrap())
        }
    }

    impl RetrieveUrl<InteractiveLogin> for jurisdiction::Spain {
        fn url(&self) -> BetfairUrl<InteractiveLogin> {
            BetfairUrl::new(url::Url::parse("https://identitysso.betfair.es/api/login").unwrap())
        }
    }

    impl RetrieveUrl<InteractiveLogin> for jurisdiction::Romania {
        fn url(&self) -> BetfairUrl<InteractiveLogin> {
            BetfairUrl::new(url::Url::parse("https://identitysso.betfair.ro/api/login").unwrap())
        }
    }

    impl RetrieveUrl<InteractiveLogin> for jurisdiction::Sweden {
        fn url(&self) -> BetfairUrl<InteractiveLogin> {
            BetfairUrl::new(url::Url::parse("https://identitysso.betfair.se/api/login").unwrap())
        }
    }

    impl RetrieveUrl<InteractiveLogin> for jurisdiction::CustomUrl<InteractiveLogin> {
        fn url(&self) -> BetfairUrl<InteractiveLogin> {
            self.0.clone()
        }
    }
}

mod logout_url {
    use super::*;

    impl RetrieveUrl<Logout> for jurisdiction::Global {
        fn url(&self) -> BetfairUrl<Logout> {
            BetfairUrl::new(url::Url::parse("https://identitysso.betfair.com/api/logout").unwrap())
        }
    }
    impl RetrieveUrl<Logout> for jurisdiction::CustomUrl<Logout> {
        fn url(&self) -> BetfairUrl<Logout> {
            self.0.clone()
        }
    }
}

mod stream_url {
    use super::*;

    impl RetrieveUrl<Stream> for jurisdiction::Global {
        fn url(&self) -> BetfairUrl<Stream> {
            BetfairUrl::new(url::Url::parse("tcptls://stream-api.betfair.com:443").unwrap())
        }
    }
    impl RetrieveUrl<Stream> for jurisdiction::CustomUrl<Stream> {
        fn url(&self) -> BetfairUrl<Stream> {
            self.0.clone()
        }
    }
}
