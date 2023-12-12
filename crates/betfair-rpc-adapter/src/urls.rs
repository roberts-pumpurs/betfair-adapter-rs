use std::borrow::Cow;

#[derive(Debug)]
pub struct BetfairUrl<'a, T> {
    url: Cow<'a, url::Url>,
    _type: std::marker::PhantomData<T>,
}

impl<'a, T> BetfairUrl<'a, T> {
    pub fn new(url: Cow<'a, url::Url>) -> Self {
        Self { url, _type: std::marker::PhantomData }
    }

    pub fn url(&self) -> &url::Url {
        &self.url
    }
}

impl<'a, T> std::ops::Deref for BetfairUrl<'a, T> {
    type Target = url::Url;

    fn deref(&self) -> &Self::Target {
        &self.url
    }
}

#[derive(Debug)]
pub struct RestBase;

#[derive(Debug)]
pub struct KeepAlive;

#[derive(Debug)]
pub struct CertLogin;

impl Default for BetfairUrl<'static, RestBase> {
    fn default() -> Self {
        Self::new(Cow::Owned(
            url::Url::parse("https://api.betfair.com/exchange/betting/rest/v1.0").unwrap(),
        ))
    }
}

impl Default for BetfairUrl<'static, KeepAlive> {
    fn default() -> Self {
        Self::new(Cow::Owned(
            url::Url::parse("https://identitysso.betfair.com/api/keepAlive").unwrap(),
        ))
    }
}

impl Default for BetfairUrl<'static, CertLogin> {
    fn default() -> Self {
        Self::new(Cow::Owned(
            url::Url::parse("https://identitysso-cert.betfair.com/api/certlogin").unwrap(),
        ))
    }
}
