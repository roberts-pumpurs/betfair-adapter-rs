use betfair_adapter::betfair_types::types::sports_aping::{
    MarketFilter, MarketProjection, MarketSort, list_market_catalogue,
};
use betfair_adapter::{
    ApplicationKey, BetfairRpcClient, Identity, Password, SecretProvider, Username,
};
use betfair_stream_api::types::request::market_subscription_message::LadderLevel;
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
    let bf_provider = BetfairRpcClient::new(secret_provider.clone())?
        .authenticate()
        .await?;
    // get market id
    let market_book = bf_provider
        .send_request(list_market_catalogue::Parameters {
            filter: MarketFilter::builder().in_play_only(true).build(),
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
    let mut stream = {
        use betfair_stream_api::BetfairProviderExt;

        BetfairRpcClient::new(secret_provider.clone())?
            .connect_to_stream()
            .run_with_default_runtime()
            .enable_cache()
    };

    // start processing stream
    {
        use betfair_stream_api::types::request::market_subscription_message::{
            Fields, MarketFilter,
        };
        let mut ms = MarketSubscriber::new(
            &stream,
            MarketFilter::default(),
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

        tokio::spawn(async move {
            // sleep for a bit to allow the stream to connect
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            ms.subscribe_to_market(market_id).unwrap();
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
