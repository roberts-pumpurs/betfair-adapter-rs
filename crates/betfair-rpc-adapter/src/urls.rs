#[derive(Debug)]
pub struct RestBase(pub url::Url);

#[derive(Debug)]
pub struct KeepAlive(pub url::Url);

#[derive(Debug)]
pub struct CertLogin(pub url::Url);

impl Default for RestBase {
    fn default() -> Self {
        Self(url::Url::parse("https://api.betfair.com/exchange/betting/rest/v1.0").unwrap())
    }
}

impl Default for KeepAlive {
    fn default() -> Self {
        Self(url::Url::parse("https://identitysso.betfair.com/api/keepAlive").unwrap())
    }
}

impl Default for CertLogin {
    fn default() -> Self {
        Self(url::Url::parse("https://identitysso-cert.betfair.com/api/certlogin").unwrap())
    }
}
