#[derive(Debug)]
pub struct SecretProvider;

impl SecretProvider {
    pub fn application_key(&self) -> &ApplicationKey {
        todo!()
    }
    pub fn username(&self) -> &Username {
        todo!()
    }
    pub fn password(&self) -> &Password {
        todo!()
    }
    pub fn identity(&self) -> &Identity {
        todo!()
    }
}

#[derive(Debug)]
pub struct ApplicationKey(pub(crate) redact::Secret<String>);

#[derive(Debug)]
pub struct Username(pub(crate) redact::Secret<String>);

#[derive(Debug)]
pub struct Password(pub(crate) redact::Secret<String>);

#[derive(Debug)]
pub struct Identity(pub(crate) redact::Secret<reqwest::Identity>);

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
