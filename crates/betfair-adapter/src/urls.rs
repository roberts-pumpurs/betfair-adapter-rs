use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct BetfairUrl<'a, T, K = url::Url>
where
    K: std::clone::Clone,
{
    url: Cow<'a, K>,
    _type: std::marker::PhantomData<T>,
}

impl<'a, T, K: std::clone::Clone> BetfairUrl<'a, T, K> {
    pub fn new(url: Cow<'a, K>) -> Self {
        Self {
            url,
            _type: std::marker::PhantomData,
        }
    }

    pub fn url(&self) -> &K {
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

#[derive(Debug, Clone)]
pub struct Stream;

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

    #[derive(Debug, Clone)]
    pub struct CustomUrl<T, K = url::Url>(pub super::BetfairUrl<'static, T, K>)
    where
        K: std::clone::Clone + 'static;

    impl<T, K> CustomUrl<T, K>
    where
        K: std::clone::Clone,
    {
        pub fn new(url: K) -> Self {
            Self(super::BetfairUrl::new(std::borrow::Cow::Owned(url)))
        }
    }
}

pub trait RetrieveUrl<'a, T, K = url::Url>
where
    K: std::clone::Clone,
{
    fn url(&self) -> BetfairUrl<'a, T, K>;
}

mod rest_url {
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

    impl<'a> RetrieveUrl<'a, RestBase> for jurisdictions::CustomUrl<RestBase> {
        fn url(&self) -> BetfairUrl<'a, RestBase> {
            self.0.clone()
        }
    }
}

mod keep_alive_url {
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

    impl<'a> RetrieveUrl<'a, KeepAlive> for jurisdictions::CustomUrl<KeepAlive> {
        fn url(&self) -> BetfairUrl<'a, KeepAlive> {
            self.0.clone()
        }
    }
}

mod bot_login_url {
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

    impl<'a> RetrieveUrl<'a, BotLogin> for jurisdictions::CustomUrl<BotLogin> {
        fn url(&self) -> BetfairUrl<'a, BotLogin> {
            self.0.clone()
        }
    }
}

mod interactive_login_url {
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

    impl<'a> RetrieveUrl<'a, InteractiveLogin> for jurisdictions::CustomUrl<InteractiveLogin> {
        fn url(&self) -> BetfairUrl<'a, InteractiveLogin> {
            self.0.clone()
        }
    }
}

mod logout_url {
    use super::*;

    impl<'a> RetrieveUrl<'a, Logout> for jurisdictions::Global {
        fn url(&self) -> BetfairUrl<'a, Logout> {
            BetfairUrl::new(Cow::Owned(
                url::Url::parse("https://identitysso.betfair.com/api/logout").unwrap(),
            ))
        }
    }
    impl<'a> RetrieveUrl<'a, Logout> for jurisdictions::CustomUrl<Logout> {
        fn url(&self) -> BetfairUrl<'a, Logout> {
            self.0.clone()
        }
    }
}

mod stream_url {
    use super::*;

    impl<'a> RetrieveUrl<'a, Stream, std::net::SocketAddr> for jurisdictions::Global {
        fn url(&self) -> BetfairUrl<'a, Stream, std::net::SocketAddr> {
            use std::net::ToSocketAddrs;
            let addr = ("stream-api.betfair.com", 443)
                .to_socket_addrs()
                .unwrap()
                .next()
                .unwrap();
            BetfairUrl::new(Cow::Owned(addr))
        }
    }
    impl<'a> RetrieveUrl<'a, Stream, std::net::SocketAddr>
        for jurisdictions::CustomUrl<Stream, std::net::SocketAddr>
    {
        fn url(&self) -> BetfairUrl<'a, Stream, std::net::SocketAddr> {
            self.0.clone()
        }
    }
}
