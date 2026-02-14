use std::sync::Arc;

use betfair_stream_types::request::RequestMessage;
use betfair_stream_types::request::authentication_message::AuthenticationMessage;
use betfair_stream_types::request::heartbeat_message::HeartbeatMessage;
use betfair_stream_types::request::market_subscription_message::{
    Fields, MarketDataFilter, MarketFilter, MarketSubscriptionMessage,
};
use betfair_stream_types::request::order_subscription_message::{
    OrderFilter, OrderSubscriptionMessage,
};
use betfair_types::types::sports_aping::{EventTypeId, MarketId};
use criterion::{Criterion, black_box, criterion_group, criterion_main};

fn ser_market_subscription(c: &mut Criterion) {
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
                .ladder_levels(
                    betfair_stream_types::request::market_subscription_message::LadderLevel::new(3)
                        .unwrap(),
                )
                .build(),
        )),
    });

    c.bench_function("ser_market_subscription", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(black_box(&msg)).unwrap());
        });
    });
}

fn ser_order_subscription(c: &mut Criterion) {
    let msg = RequestMessage::OrderSubscription(OrderSubscriptionMessage {
        id: Some(2),
        segmentation_enabled: Some(true),
        order_filter: Some(Box::new(
            OrderFilter::builder()
                .include_overall_position(Some(true))
                .account_ids(None)
                .customer_strategy_refs(None)
                .partition_matched_by_strategy_ref(Some(false))
                .build(),
        )),
        clk: None,
        heartbeat_ms: Some(500),
        initial_clk: None,
        conflate_ms: Some(0),
    });

    c.bench_function("ser_order_subscription", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(black_box(&msg)).unwrap());
        });
    });
}

fn ser_authentication(c: &mut Criterion) {
    let msg = RequestMessage::Authentication(AuthenticationMessage {
        id: Some(1),
        session: "xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx".to_owned(),
        app_key: "qa{n}pCPTV]EYTLGVO".to_owned(),
    });

    c.bench_function("ser_authentication", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(black_box(&msg)).unwrap());
        });
    });
}

fn ser_heartbeat(c: &mut Criterion) {
    let msg = RequestMessage::Heartbeat(HeartbeatMessage { id: Some(1) });

    c.bench_function("ser_heartbeat", |b| {
        b.iter(|| {
            black_box(serde_json::to_string(black_box(&msg)).unwrap());
        });
    });
}

criterion_group!(
    benches,
    ser_market_subscription,
    ser_order_subscription,
    ser_authentication,
    ser_heartbeat,
);
criterion_main!(benches);
