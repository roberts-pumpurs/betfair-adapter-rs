use betfair_adapter::betfair_types::types::sports_aping::MarketId;
use betfair_adapter::{ApplicationKey, SessionToken};
use betfair_stream_api::{HeartbeatStrategy, MarketSubscriber};
use betfair_stream_server_mock::{ClientState, StreamAPIBackend, SubSate};
use futures_util::StreamExt;

#[rstest::rstest]
#[timeout(std::time::Duration::from_secs(5))]
#[test_log::test(tokio::test)]
async fn market_subscription() {
    let mock = StreamAPIBackend::new().await;
    let url = mock.url.clone();

    let h1 = tokio::spawn(async move {
        let (stream_listener, async_task, _output) = betfair_stream_api::StreamListener::new(
            ApplicationKey::new("app_key".to_string()),
            SessionToken::new("token".to_string()),
            url.into(),
            HeartbeatStrategy::None,
        )
        .await
        .unwrap();
        let async_task_handle = tokio::spawn(async_task);

        let mut ms = MarketSubscriber::new(
            stream_listener,
            betfair_stream_types::request::market_subscription_message::MarketFilter::default(),
            vec![betfair_stream_types::request::market_subscription_message::Fields::ExBestOffers],
            Some(
                betfair_stream_types::request::market_subscription_message::LadderLevel::new(3)
                    .unwrap(),
            ),
        );

        let market_id = MarketId("1.23456789".to_string());
        let mut receiver = ms.subscribe_to_market(market_id).await;

        let msg = receiver.next().await.unwrap();

        async_task_handle.await.unwrap();
    });

    let connection = mock.process_next().await;
    let conn_state = connection.state.clone();

    let h2 = tokio::spawn(async move {
        connection.process().await;
    });

    // Sleep for 1 second to allow the connection to be established
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    assert!(!h1.is_finished());
    assert!(!h2.is_finished());
    let conn_state = conn_state.lock().await;
    assert_eq!(
        *conn_state,
        ClientState::LoggedIn(SubSate {
            keep_alive_counter: 0
        })
    );
}
