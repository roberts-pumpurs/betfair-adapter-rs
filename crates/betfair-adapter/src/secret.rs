use std::borrow::Cow;

#[derive(Debug, Clone)]
pub struct SecretProvider<'a> {
    pub application_key: Cow<'a, ApplicationKey>,
    pub username: Cow<'a, Username>,
    pub password: Cow<'a, Password>,
    pub identity: Cow<'a, Identity>,
}

#[derive(Debug, Clone)]
pub struct ApplicationKey(pub redact::Secret<String>);

#[derive(Debug, Clone)]
pub struct SessionToken(pub redact::Secret<String>);

#[derive(Debug, Clone)]
pub struct Username(pub redact::Secret<String>);

#[derive(Debug, Clone)]
pub struct Password(pub redact::Secret<String>);

#[derive(Debug, Clone)]
pub struct Identity(pub redact::Secret<reqwest::Identity>);

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
