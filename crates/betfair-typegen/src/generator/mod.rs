use betfair_xml_parser::Interface;
use output::GeneratedOutput;
use settings::GeneratorSettings;

use crate::GeneratorStrategy;

/// Contains the output structure for the types generator
pub mod output;

/// Contains the settings for the types generator
pub mod settings;

/// # The Betfair type generator
/// This is the main entry point for the crate
pub struct BetfairTypeGenerator;

impl BetfairTypeGenerator {
    /// # Generate the types for the Betfair API
    /// Provide the strategy to use to generate the types and the Betfair API interface
    pub fn generate(
        &self,
        strategy: impl GeneratorStrategy,
        settings: impl GeneratorSettings,
    ) -> GeneratedOutput {
        const SERVICES: [(&str, &str); 4] = [
            ("account_aping", include_str!("../../assets/AccountAPING.xml")),
            ("heartbeat_aping", include_str!("../../assets/HeartbeatAPING.xml")),
            ("sports_aping", include_str!("../../assets/SportsAPING.xml")),
            ("stream_api", include_str!("../../assets/ESASwaggerSchema.json")),
        ];

        fn parse_aping(
            strategy: &impl GeneratorStrategy,
            output: &mut GeneratedOutput,
            idx: usize,
        ) {
            let (module_name, interface): (&str, Interface) =
                (SERVICES[idx].0, SERVICES[idx].1.into());
            let submodule_output = strategy.generate_submodule(interface);
            output.submodules_mut().push((module_name.to_string(), submodule_output));
        }

        let mut output = GeneratedOutput::new();
        *output.root_mut() = strategy.generate_mod(&settings);

        if settings.account_aping() {
            parse_aping(&strategy, &mut output, 0);
        }
        if settings.heartbeat_aping() {
            parse_aping(&strategy, &mut output, 1);
        }
        if settings.sports_aping() {
            parse_aping(&strategy, &mut output, 2);
        }
        if settings.stream_api() {
            unimplemented!("Stream API is not yet implemented")
        }

        output
    }
}
