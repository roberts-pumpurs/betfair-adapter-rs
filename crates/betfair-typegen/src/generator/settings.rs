/// Trait that defines the settings for the generator.
pub trait GeneratorSettings {
    /// Whether to generate the types for the AccountAPING service
    fn account_aping(&self) -> bool;
    /// Whether to generate the types for the HeartbeatAPING service
    fn heartbeat_aping(&self) -> bool;
    /// Whether to generate the types for the SportsAPING service
    fn sports_aping(&self) -> bool;
    /// Whether to generate the types for the Stream API
    fn stream_api(&self) -> bool;
}

/// Simple implementation of the GeneratorSettings trait.
pub struct SimpleGeneratorSettings {
    account_aping: bool,
    heartbeat_aping: bool,
    sports_aping: bool,
    stream_api: bool,
}

impl SimpleGeneratorSettings {
    /// Create a new instance of the SimpleGeneratorSettings struct.
    pub fn new(
        account_aping: bool,
        heartbeat_aping: bool,
        sports_aping: bool,
        stream_api: bool,
    ) -> Self {
        Self {
            account_aping,
            heartbeat_aping,
            sports_aping,
            stream_api,
        }
    }

    /// Create a new instance of the SimpleGeneratorSettings struct with only the AccountAPING
    /// service enabled.
    pub fn aping_only() -> Self {
        Self::new(true, true, true, false)
    }

    /// Create a new instance of the SimpleGeneratorSettings struct with all services enabled.
    pub fn all() -> Self {
        Self::new(true, true, true, true)
    }
}

impl GeneratorSettings for SimpleGeneratorSettings {
    fn account_aping(&self) -> bool {
        self.account_aping
    }

    fn heartbeat_aping(&self) -> bool {
        self.heartbeat_aping
    }

    fn sports_aping(&self) -> bool {
        self.sports_aping
    }

    fn stream_api(&self) -> bool {
        self.stream_api
    }
}
