use std::path::PathBuf;

use clap::Parser;
use xshell::{Shell, cmd};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
enum Args {
    Deny,
    Test {
        #[clap(short, long, default_value_t = false)]
        coverage: bool,
        #[clap(last = true)]
        args: Vec<String>,
    },
    Check,
    Fmt,
    Doc,
    UnusedDeps,
    Typos {
        #[clap(short, long, default_value_t = false)]
        write: bool,
    },
    Bench {
        /// Criterion --save-baseline name
        #[clap(long)]
        save_baseline: Option<String>,
    },
    SubscribeToMarket,
    /// Generate a Betfair certificate for non-interactive bot usage
    /// Reference:
    /// https://docs.developer.betfair.com/display/1smk3cen4v3lu3yomq5qye0ni/Non-Interactive+(bot)+login
    GenBetfairCertificate {
        /// The country name (2 letter identifier) for the certificate
        #[clap(short, long, default_value = "GB")]
        country_name: String,
        /// The state or province name for the certificate (a city, neighborhood, or other locality
        /// name)
        #[clap(short, long, default_value = "London")]
        state_or_province_name: String,
        /// The organizational unit name for the certificate (a department, team, or other group
        /// name)
        #[clap(short, long, default_value = "IT Department")]
        organizational_unit_name: String,
        /// The common name for the certificate (a domain name, email address, or other identifier)
        #[clap(long, default_value = "Awesome Trading App in Rust")]
        common_name: String,
    },
}

fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    let sh = Shell::new()?;
    let args = Args::parse();

    sh.change_dir(project_root());
    match args {
        Args::Deny => {
            println!("cargo deny");
            cmd!(sh, "cargo install --locked cargo-deny").run()?;
            cmd!(sh, "cargo deny check").run()?;
        }
        Args::Test { args, coverage } => {
            println!("cargo test");
            cmd!(sh, "cargo install --locked cargo-nextest").run()?;

            if coverage {
                cmd!(sh, "cargo install --locked grcov").run()?;
                for (key, val) in [
                    ("CARGO_INCREMENTAL", "0"),
                    ("RUSTFLAGS", "-Cinstrument-coverage"),
                    ("LLVM_PROFILE_FILE", "target/coverage/%p-%m.profraw"),
                ] {
                    sh.set_var(key, val);
                }
            }
            cmd!(
                sh,
                "cargo nextest run --workspace --tests --all-targets --no-fail-fast {args...}"
            )
            .run()?;

            if coverage {
                cmd!(sh, "mkdir -p target/coverage").run()?;
                cmd!(sh, "grcov . --binary-path ./target/debug/deps/ -s . -t html,cobertura --branch --ignore-not-existing --ignore '../*' --ignore \"/*\" -o target/coverage/").run()?;

                if std::option_env!("CI").is_some() {
                    return Ok(());
                }

                // Open the generated file
                #[cfg(target_os = "macos")]
                cmd!(sh, "open target/coverage/html/index.html").run()?;

                #[cfg(target_os = "linux")]
                cmd!(sh, "xdg-open target/coverage/html/index.html").run()?;
            }
        }
        Args::Check => {
            println!("cargo check");
            cmd!(sh, "cargo clippy --workspace --locked -- -D warnings").run()?;
            cmd!(sh, "cargo fmt --all --check").run()?;
        }
        Args::Fmt => {
            println!("cargo fix");
            cmd!(sh, "cargo fmt --all").run()?;
            cmd!(
                sh,
                "cargo clippy --fix --allow-dirty --allow-staged --workspace --all-features"
            )
            .run()?;
        }
        Args::Doc => {
            println!("cargo doc");
            cmd!(sh, "cargo doc --workspace --no-deps --all-features").run()?;
            if std::option_env!("CI").is_some() {
                return Ok(());
            }

            // Open the generated file
            #[cfg(target_os = "macos")]
            cmd!(sh, "open target/doc/minerva/index.html").run()?;

            #[cfg(target_os = "linux")]
            cmd!(sh, "xdg-open target/doc/minerva/index.html").run()?;
        }
        Args::UnusedDeps => {
            println!("unused deps");
            cmd!(sh, "cargo install --locked cargo-machete").run()?;
            cmd!(sh, "cargo-machete").run()?;
        }
        Args::Typos { write } => {
            println!("cargo spellcheck");
            cmd!(sh, "cargo install typos-cli").run()?;
            if write {
                cmd!(sh, "typos -w").run()?;
            } else {
                cmd!(sh, "typos").run()?;
            }
        }
        Args::Bench { save_baseline } => {
            println!("cargo bench");
            let benches = [
                ("betfair-types", "numeric"),
                ("betfair-stream-types", "serialize"),
                ("betfair-stream-api", "deserialize"),
                ("betfair-stream-api", "cache_update"),
                ("betfair-stream-api", "process_message"),
                ("betfair-stream-api", "codec"),
                ("betfair-adapter", "rpc"),
            ];
            for (crate_name, bench_name) in benches {
                if let Some(ref baseline) = save_baseline {
                    cmd!(
                        sh,
                        "cargo bench -p {crate_name} --bench {bench_name} -- --save-baseline {baseline}"
                    )
                    .run()?;
                } else {
                    cmd!(sh, "cargo bench -p {crate_name} --bench {bench_name}").run()?;
                }
            }
        }
        Args::SubscribeToMarket => {
            println!("stream-api-subscribe to market");
            cmd!(sh, "cargo run --bin stream-api-subscribe-to-market").run()?;
        }
        Args::GenBetfairCertificate {
            country_name,
            state_or_province_name,
            organizational_unit_name,
            common_name,
        } => {
            println!("gen betfair certificate");
            let (cert, key_pair) = betfair_cert_gen::rcgen_cert(
                country_name.as_str(),
                state_or_province_name.as_str(),
                organizational_unit_name.as_str(),
                common_name.as_str(),
            )?;

            let pem_serialized = cert.pem();
            println!("{pem_serialized}");
            println!("{}", key_pair.serialize_pem());

            std::fs::create_dir_all("certs/")?;
            std::fs::write("certs/cert.pem", pem_serialized.as_bytes())?;
            std::fs::write("certs/key.pem", key_pair.serialize_pem().as_bytes())?;
            std::fs::write("certs/identity.pem", {
                let mut identity = pem_serialized.clone();
                identity.push_str(&key_pair.serialize_pem());
                identity
            })?;
        }
    }

    Ok(())
}

/// Returns the path to the root directory
fn project_root() -> PathBuf {
    let dir = env!("CARGO_MANIFEST_DIR").to_owned();
    PathBuf::from(dir).parent().unwrap().to_owned()
}
