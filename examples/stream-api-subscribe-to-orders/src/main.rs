use std::time::Duration;

use betfair_adapter::{
    ApplicationKey, BetfairRpcClient, Identity, Password, SecretProvider, Username,
};
use betfair_stream_api::cache::order_subscriber::OrderSubscriber;
use betfair_stream_api::types::request::order_subscription_message::OrderFilter;
use betfair_stream_api::types::response::status_message::StatusMessage;
use betfair_stream_api::{BetfairStreamBuilder, Cache, CachedMessage};
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
    let bf_client = BetfairRpcClient::new(secret_provider.clone())?
        .authenticate()
        .await?;

    // connect to stream
    let stream =
        BetfairStreamBuilder::<Cache>::new(bf_client).with_heartbeat(Duration::from_secs(5));
    let (mut stream, _task) = stream.start().await;

    // start processing stream
    let os = OrderSubscriber::new(&stream, OrderFilter::default());

    let mut subscribed = false;
    while let Some(message) = stream.sink.recv().await {
        tracing::info!(?message, "received value from stream");
        if let CachedMessage::Status(status_message) = message {
            match status_message {
                StatusMessage::Success(_message) => {
                    if !subscribed {
                        os.resubscribe().await?;
                        subscribed = true;
                    }
                }
                StatusMessage::Failure(err) => {
                    tracing::error!(?err, "error during auth");
                }
            }
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
