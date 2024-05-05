#[derive(Debug, Clone)]
pub struct SecretProvider {
    pub application_key: ApplicationKey,
    pub username: Username,
    pub password: Password,
    pub identity: Identity,
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
