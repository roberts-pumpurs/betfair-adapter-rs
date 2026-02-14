use std::env;
use std::path::PathBuf;

use betfair_stream_api::{StreamAPIClientCodec, cache};
use betfair_stream_types::response::ResponseMessage;
use futures::StreamExt as _;
use tokio::fs::File;
use tokio_util::codec::Framed;

#[tokio::test]
async fn test_can_build_prod_cache_from_stream_data() {
    let fixture = PathBuf::new()
        .join(env::current_dir().unwrap())
        .join("fixtures")
        .join("29788105");
    dbg!(&fixture);

    let mut cached_state = cache::tracker::StreamState::new();

    let file = File::open(fixture).await.unwrap();
    let mut framed = Framed::new(file, StreamAPIClientCodec).take(500);
    while let Some(frame) = framed.next().await {
        let (_raw, frame) = frame.unwrap();
        match frame {
            ResponseMessage::Connection(_) => {}
            ResponseMessage::MarketChange(msg) => {
                cached_state.market_change_update(msg);
            }
            ResponseMessage::OrderChange(msg) => {
                cached_state.order_change_update(msg);
            }
            ResponseMessage::Status(_) => {}
        }
    }
    dbg!(cached_state);
}
