use std::time::Duration;

use betfair_adapter::{
    ApplicationKey, Identity, Password, SecretProvider, UnauthenticatedBetfairRpcProvider, Username,
};
use betfair_stream_api::types::request::order_subscription_message::OrderFilter;
use betfair_stream_api::{HeartbeatStrategy, OrderSubscriber};
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;

#[derive(Debug, serde::Deserialize)]
struct Config {
    betfair_username: Username,
    betfair_application_key: ApplicationKey,
    betfair_password: Password,
    betfair_identity: Identity,
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
        identity: config.betfair_identity,
    };

    // connect to stream
    let mut stream = {
        use betfair_stream_api::BetfairProviderExt;
        UnauthenticatedBetfairRpcProvider::new(secret_provider.clone())?
            .connect_to_stream_with_hb(HeartbeatStrategy::Interval(Duration::from_secs(5)))
            .await
            .run_with_default_runtime()
            .enable_cache()
    };

    // start processing stream
    {
        use betfair_stream_api::StreamExt;
        let mut os = OrderSubscriber::new(&stream, OrderFilter::default());

        tokio::spawn(async move {
            // sleep for a bit to allow the stream to connect
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            os.resubscribe().await.unwrap();
        });

        while let Some(value) = stream.next().await {
            tracing::info!(?value, "received value from stream");
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
