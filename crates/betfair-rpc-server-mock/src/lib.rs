use betfair_adapter::jurisdiction::CustomUrl;
use betfair_adapter::{
    ApplicationKey, Authenticated, BetfairConfigBuilder, BetfairRpcClient, BotLogin, Identity,
    InteractiveLogin, KeepAlive, Logout, Password, RestBase, SecretProvider, Stream,
    Unauthenticated, Username,
};
use betfair_types::types::BetfairRpcRequest;
use serde_json::json;
pub use wiremock;
use wiremock::matchers::{PathExactMatcher, method, path};
use wiremock::{Mock, MockServer, ResponseTemplate};

mod urlencoded_matcher;
use urlencoded_matcher::FormEncodedBodyMatcher;

pub const USERNAME: &str = "usrn";
pub const PASSWORD: &str = "pasw";
pub const APP_KEY: &str = "qa{n}pCPTV]EYTLGVO";
pub const LOGOUT: &str = "/login/";
pub const LOGIN_URL: &str = "/login/";
pub const BOT_LOGIN_URL: &str = "/cert-login/";
pub const KEEP_ALIVE_URL: &str = "/keep-alive/";
pub const REST_URL: &str = "/rpc/v1/";
pub const STREAM_URL: &str = "/stream/";
pub const SESSION_TOKEN: &str = "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx";

#[must_use]
pub fn rpc_path<T: BetfairRpcRequest>() -> String {
    format!("{REST_URL}{}", T::method())
}

pub struct Server {
    pub bf_api_mock_server: MockServer,
    pub mock_settings: MockSettings,
}

#[derive(Debug, Clone)]
pub struct MockSettings {
    pub keep_alive_period: core::time::Duration,
    pub health_check_period: core::time::Duration,
    pub stream_url: CustomUrl<Stream>,
}

impl Default for MockSettings {
    fn default() -> Self {
        Self {
            keep_alive_period: core::time::Duration::from_secs(10),
            health_check_period: core::time::Duration::from_secs(10),
            stream_url: CustomUrl::new("http://localhost:80/stream".parse().unwrap()),
        }
    }
}

impl Server {
    pub async fn new() -> Self {
        Self::new_with_settings(MockSettings::default()).await
    }

    pub async fn new_with_stream_url(stream_url: CustomUrl<Stream>) -> Self {
        let mut settings = MockSettings::default();
        settings.stream_url = stream_url;
        Self::new_with_settings(settings).await
    }

    pub async fn new_with_settings(mock_settings: MockSettings) -> Self {
        let mock_server = MockServer::start().await;
        let login_response = json!(
            {
                "sessionToken": SESSION_TOKEN,
                "loginStatus": "SUCCESS"
            }
        );
        Mock::given(method("POST"))
            .and(path(BOT_LOGIN_URL))
            .and(FormEncodedBodyMatcher::new(vec![
                ("username".to_owned(), USERNAME.to_owned()),
                ("password".to_owned(), PASSWORD.to_owned()),
            ]))
            .respond_with(ResponseTemplate::new(200).set_body_json(login_response))
            .named("Login")
            .mount(&mock_server)
            .await;

        Self {
            bf_api_mock_server: mock_server,
            mock_settings,
        }
    }

    /// Create a betfair client with the mock server as the base url
    pub async fn client(&self) -> BetfairRpcClient<Unauthenticated> {
        let secrets_provider = self.secrets_provider();
        let config = self.betfair_config(secrets_provider);

        BetfairRpcClient::new_with_config(config).unwrap()
    }

    #[must_use]
    pub fn secrets_provider(&self) -> SecretProvider {
        let identity = reqwest::Identity::from_pem(CERTIFICATE.as_bytes()).unwrap();

        SecretProvider {
            application_key: ApplicationKey::new(APP_KEY.to_owned()),
            identity: Identity::new(identity),
            password: Password::new(PASSWORD.to_owned()),
            username: Username::new(USERNAME.to_owned()),
        }
    }

    #[must_use]
    pub fn betfair_config<'a>(
        &self,
        secrets_provider: SecretProvider,
    ) -> BetfairConfigBuilder<
        CustomUrl<RestBase>,
        CustomUrl<KeepAlive>,
        CustomUrl<BotLogin>,
        CustomUrl<Logout>,
        CustomUrl<InteractiveLogin>,
        CustomUrl<Stream>,
    > {
        let base_uri: url::Url = self.bf_api_mock_server.uri().parse().unwrap();

        BetfairConfigBuilder {
            rest: CustomUrl::new(base_uri.join(REST_URL).unwrap()),
            keep_alive: CustomUrl::new(base_uri.join(KEEP_ALIVE_URL).unwrap()),
            bot_login: CustomUrl::new(base_uri.join(BOT_LOGIN_URL).unwrap()),
            logout: CustomUrl::new(base_uri.join(LOGOUT).unwrap()),
            login: CustomUrl::new(base_uri.join(LOGIN_URL).unwrap()),
            stream: self.mock_settings.stream_url.clone(),
            secrets_provider,
        }
    }

    pub fn mock_success(
        &self,
        http_method: &str,
        path_matcher: PathExactMatcher,
        name: &str,
        with_auth_headers: bool,
        response: serde_json::Value,
    ) -> Mock {
        self.mock_low(
            http_method,
            path_matcher,
            name,
            with_auth_headers,
            response,
            200,
        )
    }

    pub fn mock_error(
        &self,
        http_method: &str,
        path_matcher: PathExactMatcher,
        name: &str,
        with_auth_headers: bool,
        response: serde_json::Value,
    ) -> Mock {
        self.mock_low(
            http_method,
            path_matcher,
            name,
            with_auth_headers,
            response,
            400,
        )
    }

    pub fn mock_low(
        &self,
        http_method: &str,
        path_matcher: PathExactMatcher,
        name: &str,
        with_auth_headers: bool,
        response: serde_json::Value,
        response_code: u16,
    ) -> Mock {
        use wiremock::matchers::{header, method};

        let m = Mock::given(method(http_method)).and(path_matcher);

        let m = if with_auth_headers {
            m.and(header("Accept", "application/json"))
                .and(header("X-Authentication", SESSION_TOKEN))
                .and(header("X-Application", APP_KEY))
        } else {
            m
        };

        m.respond_with(ResponseTemplate::new(response_code).set_body_json(response))
            .named(name)
    }

    pub fn mock_keep_alive(&self) -> Mock {
        let response = json!(
            {
                "token": SESSION_TOKEN,
                "product":"AppKey",
                "status": "SUCCESS",
                "error":""
            }
        );

        self.mock_success("GET", path(KEEP_ALIVE_URL), "Keep alive", true, response)
    }

    pub fn mock_authenticated_rpc<T: BetfairRpcRequest>(&self, response: T::Res) -> Mock
    where
        T::Res: serde::Serialize,
    {
        self.mock_authenticated_rpc_from_json::<T>(serde_json::to_value(&response).unwrap())
    }

    pub fn mock_authenticated_error<T: BetfairRpcRequest>(&self, response: T::Error) -> Mock
    where
        T::Error: serde::Serialize,
    {
        self.mock_error(
            "POST",
            path(rpc_path::<T>()),
            &rpc_path::<T>(),
            true,
            serde_json::to_value(response).unwrap(),
        )
    }

    pub fn mock_authenticated_rpc_from_json<T: BetfairRpcRequest>(
        &self,
        response: serde_json::Value,
    ) -> Mock {
        self.mock_success(
            "POST",
            path(rpc_path::<T>()),
            &rpc_path::<T>(),
            true,
            serde_json::to_value(response).unwrap(),
        )
    }
}

pub const CERTIFICATE: &str = "-----BEGIN CERTIFICATE-----
MIIC3zCCAcegAwIBAgIJALAul9kzR0W/MA0GCSqGSIb3DQEBBQUAMA0xCzAJBgNV
BAYTAmx2MB4XDTIyMDgwMjE5MTE1NloXDTIzMDgwMjE5MTE1NlowDTELMAkGA1UE
BhMCbHYwggEiMA0GCSqGSIb3DQEBAQUAA4IBDwAwggEKAoIBAQC8WWPaghYJcXQp
W/GAoFqKrQIwxy+h8vdZiURVzzqDKt/Mz45x0Zqj8RVSe4S0lLfkRxcgrLz7ZYSc
TKsVcur8P66F8A2AJaC4KDiYj4azkTtYQDs+RDLRJUCz5xf/Nw7m+6Y0K7p/p2m8
bPSm6osefz0orQqpwGogqOwI0FKMkU+BpYjMb+k29xbOec6aHxlaPlHLBPa+n3WC
V96KwmzSMPEN6Fn/G6PZ5PtwmNg769PiXKk02p+hbnx5OCKvi94mn8vVBGgXF6JR
Vq9IQQvfFm6G6tf7q+yxMdR2FBR2s03t1daJ3RLGdHzXWTAaNRS7E93OWx+ZyTkd
kIVM16HTAgMBAAGjQjBAMAkGA1UdEwQCMAAwEQYJYIZIAYb4QgEBBAQDAgeAMAsG
A1UdDwQEAwIFoDATBgNVHSUEDDAKBggrBgEFBQcDAjANBgkqhkiG9w0BAQUFAAOC
AQEAU/uQHjntyIVR4uQCRoSO5VKyQcXXFY5pbx4ny1yrn0Uxb9P6bOxY5ojcs0r6
z8ApT3sUfww7kzhle/G5DRtP0cELq7N2YP+qsFx8UO1GYZ5SLj6xm81vk3c0+hrO
Q3yoS60xKd/7nVsPZ3ch6+9ND0vVUOkefy0aeNix9YgbYjS11rTj7FNiHD25zOJd
VpZtHkvYDpHcnwUCd0UAuu9ntKKMFGwc9GMqzfY5De6nITvlqzH8YM4AjKO26JsU
7uMSyHtGF0vvyzhkwCqcuy7r9lQr9m1jTsJ5pSaVasIOJe+/JBUEJm5E4ppdslnW
1PkfLWOJw34VKkwibWLlwAwTDQ==
-----END CERTIFICATE-----
-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEAvFlj2oIWCXF0KVvxgKBaiq0CMMcvofL3WYlEVc86gyrfzM+O
cdGao/EVUnuEtJS35EcXIKy8+2WEnEyrFXLq/D+uhfANgCWguCg4mI+Gs5E7WEA7
PkQy0SVAs+cX/zcO5vumNCu6f6dpvGz0puqLHn89KK0KqcBqIKjsCNBSjJFPgaWI
zG/pNvcWznnOmh8ZWj5RywT2vp91glfeisJs0jDxDehZ/xuj2eT7cJjYO+vT4lyp
NNqfoW58eTgir4veJp/L1QRoFxeiUVavSEEL3xZuhurX+6vssTHUdhQUdrNN7dXW
id0SxnR811kwGjUUuxPdzlsfmck5HZCFTNeh0wIDAQABAoIBAQCNJFNukCMhanKI
98xu/js7RlCo6urn6mGvJ+0cfJE1b/CL01HEOzUt+2BmEgetJvDy0M8k/i0UGswY
MF/YT+iFpNcMqYoEaK4aspFOyedAMuoMxP1gOMz363mkFt3ls4WoVBYFbGtyc6sJ
t4BSgNpFvUXAcIPYF0ewN8XBCRODH6v7Z6CrbvtjlUXMuU02r5vzMh8a4znIJmZY
40x6oNIss3YDCGe8J6qMWHByMDZbO63gBoBYayTozzCzl1TG0RZ1oTTL4z36wRto
uAhjoRek2kiO5axIgKPR/tYlyKzwLkS5v1W09K+pvsabAU6gQlC8kUPk7/+GOaeI
wGMI9FAZAoGBAOJN8mqJ3zHKvkyFW0uFMU14dl8SVrCZF1VztIooVgnM6bSqNZ3Y
nKE7wk1DuFjqKAi/mgXTr1v8mQtr40t5dBEMdgDpfRf/RrMfQyhEgQ/m1WqBQtPx
Suz+EYMpcH05ynrfSbxCDNYM4OHNJ1QfIvHJ/Q9wt5hT7w+MOH5h5TctAoGBANUQ
cXF4QKU6P+dLUYNjrYP5Wjg4194i0fh/I9NVoUE9Xl22J8l0lybV2phkuODMp1I+
rBi9AON9skjdCnwtH2ZbRCP6a8Zjv7NMLy4b4dQqfoHwTdCJ0FBfgZXhH4i+AXMb
XsKotxKGqCWgFKY8LB3UJ0qakK6h9Ze+/zbnZ9z/AoGBAJwrQkD3SAkqakyQMsJY
9f8KRFWzaBOSciHMKSi2UTmOKTE9zKZTFzPE838yXoMtg9cVsgqXXIpUNKFHIKGy
/L/PI5fZiTQIPBfcWRHuxEne+CP5c86i0xvc8OTcsf4Y5XwJnu7FfeoxFPd+Bcft
fMXyqCoBlREPywelsk606+M5AoGAfXLICJJQJbitRYbQQLcgw/K+DxpQ54bC8DgT
pOvnHR2AAVcuB+xwzrndkhrDzABTiBZEh/BIpKkunr4e3UxID6Eu9qwMZuv2RCBY
KyLZjW1TvTf66Q0rrRb+mnvJcF7HRbnYym5CFFNaj4S4g8QsCYgPdlqZU2kizCz1
4aLQQYsCgYAGKytrtHi2BM4Cnnq8Lwd8wT8/1AASIwg2Va1Gcfp00lamuy14O7uz
yvdFIFrv4ZPdRkf174B1G+FDkH8o3NZ1cf+OuVIKC+jONciIJsYLPTHR0pgWqE4q
FAbbOyAg51Xklqm2Q954WWFmu3lluHCWUGB9eSHshIurTmDd+8o15A==
-----END RSA PRIVATE KEY-----
";
