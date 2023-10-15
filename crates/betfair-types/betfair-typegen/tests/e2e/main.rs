use betfair_typegen::{gen_v1, BetfairTypeGenerator};
use rstest::rstest;

#[rstest]
fn assert_root_module() {
    let generator = BetfairTypeGenerator;
    let strategy = gen_v1::GenV1GeneratorStrategy::preconfigured();
    let settings = betfair_typegen::settings::SimpleGeneratorSettings::aping_only();
    let output = generator.generate(strategy, settings);
    let output_mod = output.root_mod().to_string();
    assert!(output_mod.contains(quote::quote! { pub mod account_aping; }.to_string().as_str()));
    assert!(output_mod.contains(quote::quote! { pub mod heartbeat_aping; }.to_string().as_str()));
    assert!(output_mod.contains(quote::quote! { pub mod sports_aping; }.to_string().as_str()));
    assert!(!output_mod.contains(quote::quote! { pub mod stream_api; }.to_string().as_str()));
    assert_eq!(output.submodules().len(), 3);
}
