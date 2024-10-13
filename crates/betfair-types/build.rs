//! # Betfair Type Generation Crate
//!
//! This crate provides functionality to generate type bindings for Betfair's API and includes
//! tools for error handling and logging.
//!
//! ## Features
//!
//! - Generates Rust type bindings for Betfair's API
//! - Configurable type generation strategy
//! - Error handling and logging using the `tracing` ecosystem
//! - Spelling error checks for the workspace
//!
//! ## Usage
//!
//! The main entry point of this crate is the `run()` function, which performs the following tasks:
//!
//! 1. Retrieves the output directory from the environment
//! 2. Initializes a `BetfairTypeGenerator` with a specific strategy and settings
//! 3. Generates the type bindings
//! 4. Writes the generated bindings to a file in the specified output directory
//!
//! The `main()` function sets up the tracing subscriber for logging and error handling,
//! checks for spelling errors, and then calls the `run()` function.
//!
//! ## Error Handling
//!
//! This crate uses the `tracing` ecosystem for error handling and logging. Errors are
//! propagated up the call stack and logged with contextual information.
use core::error::Error;
use std::process;

use tracing_error::ErrorLayer;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry};

/// The entry point of the application. Initializes the tracing subscriber
/// and runs the main logic of the program.
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

/// Runs the type generation process for Betfair. It retrieves the output
/// directory, generates the type bindings, and writes them to a file.
///
/// # Returns
/// Returns `Ok(())` on success, or an error wrapped in `Box<dyn Error>` on failure.
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
