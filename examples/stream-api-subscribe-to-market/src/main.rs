use betfair_adapter::{
    ApplicationKey, BetfairConfigBuilder, Identity, Password, SecretProvider,
    UnauthenticatedBetfairRpcProvider, Username,
};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, serde::Deserialize)]
struct Config {
    betfair_username: Username,
    betfair_application_key: ApplicationKey,
    betfair_password: Password,
    betfair_certificate: Identity,
}

#[tokio::main]
async fn main() -> eyre::Result<()> {
    color_eyre::install()?;
    init_tracing();

    // load config
    let config = std::fs::read_to_string("example_config.toml")?;
    let config = toml::from_str::<Config>(config.as_str())?;
    let secret_provider = SecretProvider {
        application_key: config.betfair_application_key,
        username: config.betfair_username,
        password: config.betfair_password,
        identity: config.betfair_certificate,
    };

    // login to betfair
    let bf_provider = UnauthenticatedBetfairRpcProvider::new(secret_provider)?
        .authenticate()
        .await?;

    // connect to stream
    let mut stream = {
        use betfair_stream_api::BetfairProviderExt;

        bf_provider
            .connect_to_stream()
            .await
            .run_with_default_runtime()
    };

    // start processing stream
    {
        use betfair_stream_api::futures::StreamExt;

        while let Some(value) = stream.next().await {
            tracing::info!(?value, "received vaue from stream");
        }
    }
    Ok(())
}

fn init_tracing() {
    let subscriber = tracing_subscriber::Registry::default();
    let level_filter_layer = tracing_subscriber::EnvFilter::try_from_default_env()
        .unwrap_or(tracing_subscriber::EnvFilter::new("INFO"));
    let output_layer = tracing_subscriber::fmt::layer()
        .with_line_number(true)
        .with_ansi(true)
        .with_file(true)
        .with_writer(std::io::stderr);
    subscriber
        .with(level_filter_layer)
        .with(output_layer)
        .init();
}
