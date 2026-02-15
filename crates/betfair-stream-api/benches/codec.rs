use std::sync::Arc;

use betfair_stream_api::StreamAPIClientCodec;
use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::request::heartbeat_message::HeartbeatMessage;
use betfair_stream_types::request::market_subscription_message::{
    Fields, MarketDataFilter, MarketFilter, MarketSubscriptionMessage,
};
use betfair_types::types::sports_aping::{EventTypeId, MarketId};
use bytes::BytesMut;
use std::hint::black_box;

use criterion::{BatchSize, Criterion, criterion_group, criterion_main};
use tokio_util::codec::Encoder;

fn codec_encode_market_subscription(c: &mut Criterion) {
    let msg = RequestMessage::MarketSubscription(MarketSubscriptionMessage {
        id: Some(1),
        segmentation_enabled: Some(true),
        clk: None,
        heartbeat_ms: Some(500),
        initial_clk: None,
        market_filter: Some(Box::new(
            MarketFilter::builder()
                .market_ids(vec![
                    MarketId::new("1.206502771"),
                    MarketId::new("1.206502772"),
                ])
                .event_type_ids(vec![EventTypeId(Arc::new("7".to_owned()))])
                .turn_in_play_enabled(true)
                .build(),
        )),
        conflate_ms: Some(0),
        market_data_filter: Some(Box::new(
            MarketDataFilter::builder()
                .fields(vec![
                    Fields::ExBestOffers,
                    Fields::ExTradedVol,
                    Fields::ExLtp,
                    Fields::ExMarketDef,
                ])
                .build(),
        )),
    });

    c.bench_function("codec_encode_market_subscription", |b| {
        b.iter_batched(
            || {
                (
                    StreamAPIClientCodec,
                    BytesMut::with_capacity(512),
                    msg.clone(),
                )
            },
            |(mut codec, mut buf, msg)| {
                codec.encode(msg, &mut buf).unwrap();
                black_box(buf);
            },
            BatchSize::SmallInput,
        );
    });
}

fn codec_encode_heartbeat(c: &mut Criterion) {
    let msg = RequestMessage::Heartbeat(HeartbeatMessage { id: Some(1) });

    c.bench_function("codec_encode_heartbeat", |b| {
        b.iter_batched(
            || {
                (
                    StreamAPIClientCodec,
                    BytesMut::with_capacity(64),
                    msg.clone(),
                )
            },
            |(mut codec, mut buf, msg)| {
                codec.encode(msg, &mut buf).unwrap();
                black_box(buf);
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(
    benches,
    codec_encode_market_subscription,
    codec_encode_heartbeat
);
criterion_main!(benches);
