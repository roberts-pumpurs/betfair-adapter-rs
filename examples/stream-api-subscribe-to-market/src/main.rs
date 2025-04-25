use betfair_adapter::betfair_types::types::sports_aping::{
    self, MarketProjection, MarketSort, list_market_catalogue,
};
use betfair_adapter::{
    ApplicationKey, BetfairRpcClient, Identity, Password, SecretProvider, Username,
};
use betfair_stream_api::cache::market_subscriber::MarketSubscriber;
use betfair_stream_api::types::request::market_subscription_message::LadderLevel;
use betfair_stream_api::types::request::market_subscription_message::{self, Fields};
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

    // login to betfair
    let bf_unauth = BetfairRpcClient::new(secret_provider.clone())?;
    let (bf_client, _) = bf_unauth.clone().authenticate().await?;
    // get market id
    let market_book = bf_client
        .send_request(list_market_catalogue::Parameters {
            filter: sports_aping::MarketFilter::builder()
                .in_play_only(true)
                .build(),
            market_projection: Some(vec![
                MarketProjection::Event,
                MarketProjection::Competition,
                MarketProjection::EventType,
                MarketProjection::MarketDescription,
            ]),
            sort: Some(MarketSort::MaximumTraded),
            max_results: 1,
            locale: None,
        })
        .await?;
    let market_id = market_book[0].market_id.clone();

    // connect to stream
    let stream = BetfairStreamBuilder::<Cache>::new(bf_unauth.clone());
    let (mut stream, _task) = stream.start::<10>();

    // start processing stream
    let mut ms = MarketSubscriber::new(
        &stream,
        market_subscription_message::MarketFilter::default(),
        vec![
            Fields::ExAllOffers,
            Fields::ExBestOffers,
            Fields::ExBestOffersDisp,
            Fields::ExLtp,
            Fields::ExMarketDef,
            Fields::ExTraded,
            Fields::ExTradedVol,
            Fields::SpProjected,
            Fields::SpTraded,
        ],
        Some(LadderLevel::new(1).unwrap()),
    );

    let mut subscribed = false;
    while let Some(message) = stream.sink.recv().await {
        tracing::info!(?message, "received value from stream");
        if let CachedMessage::Status(status_message) = message {
            match status_message {
                StatusMessage::Success(_message) => {
                    if !subscribed {
                        ms.subscribe_to_market(market_id.clone()).await?;
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
