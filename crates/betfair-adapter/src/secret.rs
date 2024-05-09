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
    pub fn new(application_key: String) -> Self {
        Self(redact::Secret::new(application_key))
    }
}

impl Username {
    pub fn new(username: String) -> Self {
        Self(redact::Secret::new(username))
    }
}
impl SessionToken {
    pub fn new(username: String) -> Self {
        Self(redact::Secret::new(username))
    }
}

impl Password {
    pub fn new(password: String) -> Self {
        Self(redact::Secret::new(password))
    }
}

impl Identity {
    pub fn new(identity: reqwest::Identity) -> Self {
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
        let identity =
            reqwest::Identity::from_pem(raw_string.as_bytes()).map_err(serde::de::Error::custom)?;
        Ok(redact::Secret::new(identity))
    }
}
