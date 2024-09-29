#[derive(Debug, Clone, serde::Deserialize)]
pub struct SecretProvider {
    pub application_key: ApplicationKey,
    pub username: Username,
    pub password: Password,
    pub identity: Identity,
}

#[derive(Debug, Clone, serde::Deserialize)]
pub struct ApplicationKey(pub redact::Secret<String>);

#[derive(Debug, Clone, serde::Deserialize)]
pub struct SessionToken(pub redact::Secret<String>);

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Username(pub redact::Secret<String>);

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Password(pub redact::Secret<String>);

#[derive(Debug, Clone, serde::Deserialize)]
pub struct Identity(
    #[serde(deserialize_with = "serde_utils::deserialize_identity")]
    pub  redact::Secret<reqwest::Identity>,
);

impl ApplicationKey {
    #[must_use]
    pub const fn new(application_key: String) -> Self {
        Self(redact::Secret::new(application_key))
    }
}

impl Username {
    #[must_use]
    pub const fn new(username: String) -> Self {
        Self(redact::Secret::new(username))
    }
}
impl SessionToken {
    #[must_use]
    pub const fn new(username: String) -> Self {
        Self(redact::Secret::new(username))
    }
}

impl Password {
    #[must_use]
    pub const fn new(password: String) -> Self {
        Self(redact::Secret::new(password))
    }
}

impl Identity {
    #[must_use]
    pub const fn new(identity: reqwest::Identity) -> Self {
        Self(redact::Secret::new(identity))
    }
}

mod serde_utils {
    use serde::{Deserialize, Deserializer};

    pub(crate) fn deserialize_identity<'de, D>(
        deserializer: D,
    ) -> Result<redact::Secret<reqwest::Identity>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw_string = String::deserialize(deserializer)?;
        let identity = reqwest::Identity::from_pem(raw_string.as_bytes())
            .inspect_err(|err| {
                tracing::error!(?err, "cannot parse identity");
            })
            .map_err(serde::de::Error::custom)?;
        Ok(redact::Secret::new(identity))
    }
}
#[cfg(test)]
mod tests {
    

    use super::*;

    #[test]
    fn test_deserialize_identity_from_toml() {
        let toml_str = r#"
application_key = "some_application_key"
username = "some_username"
password = "some_password"
identity = '''
-----BEGIN CERTIFICATE-----
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
-----BEGIN PRIVATE KEY-----
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
-----END PRIVATE KEY-----'''
"#;

        let secret_provider: SecretProvider =
            toml::from_str(toml_str).expect("Failed to deserialize SecretProvider");

        // Assertions to verify that fields are correctly deserialized
        assert_eq!(
            secret_provider.application_key.0.expose_secret(),
            "some_application_key"
        );
        assert_eq!(secret_provider.username.0.expose_secret(), "some_username");
        assert_eq!(secret_provider.password.0.expose_secret(), "some_password");
    }
}
