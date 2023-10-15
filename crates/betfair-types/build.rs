fn main() {
    let out_dir = std::env::var("OUT_DIR").unwrap();

    let generator = betfair_typegen::BetfairTypeGenerator;
    let strategy = betfair_typegen::gen_v1::GenV1GeneratorStrategy::preconfigured();
    let settings = betfair_typegen::settings::SimpleGeneratorSettings::aping_only();
    let output = generator.generate(strategy, settings);
    output.write_to_file(out_dir).unwrap();
}
