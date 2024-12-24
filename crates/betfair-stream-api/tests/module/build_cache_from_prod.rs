use std::env;
use std::path::PathBuf;

use betfair_stream_api::{cache, tokio_util, StreamAPIClientCodec};
use betfair_stream_types::response::ResponseMessage;
use futures::StreamExt;
use tokio::fs::File;
use tokio_util::codec::Framed;

#[tokio::test]
async fn test_can_build_prod_cache_from_straem_data() {
    let fixture = PathBuf::new()
        .join(env::current_dir().unwrap())
        .join("fixtures")
        .join("29788105");
    dbg!(&fixture);

    let mut cached_state = cache::tracker::StreamState::new();

    let file = File::open(fixture).await.unwrap();
    let mut framed = Framed::new(file, StreamAPIClientCodec).take(500);
    while let Some(frame) = framed.next().await {
        let frame = frame.unwrap();
        let change = match frame {
            ResponseMessage::MarketChange(mc) => cache::tracker::IncomingMessage::Market(mc),
            ResponseMessage::OrderChange(oc) => cache::tracker::IncomingMessage::Order(oc),
            ResponseMessage::Connection(_) | ResponseMessage::Status(_) => unreachable!(),
        };
        cached_state.calculate_updates(change);
    }
    dbg!(cached_state);
}
