use std::error::Error;
use std::process;

use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

fn main() {
    let subscriber = Registry::default();
    let filter = EnvFilter::new("debug");

    let output_layer = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_ansi(true)
        .with_file(true)
        .with_writer(std::io::stderr);

    subscriber
        .with(filter)
        .with(ErrorLayer::default())
        .with(output_layer)
        .init();

    if let Err(err) = run() {
        let mut source = err.source();
        while let Some(er) = source {
            source = er.source();
        }
        process::exit(1);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let out_dir = std::env::var("OUT_DIR").map_err(|er| format!("Failed to get OUT_DIR: {er}"))?;

    let generator = betfair_typegen::BetfairTypeGenerator;
    let strategy = betfair_typegen::gen_v1::GenV1GeneratorStrategy::preconfigured();
    let settings = betfair_typegen::settings::SimpleGeneratorSettings::aping_only();
    let output = generator.generate(&strategy, &settings);
    output
        .write_to_file(&out_dir)
        .inspect_err(|err| tracing::error!(?err, "error when generating betfair type bindings"))
        .map_err(|err| format!("Failed to write output: {err}"))?;

    Ok(())
}
