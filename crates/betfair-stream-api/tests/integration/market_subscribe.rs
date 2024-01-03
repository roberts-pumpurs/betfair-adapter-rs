use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_api::HeartbeatStrategy;
use betfair_stream_server_mock::{ClientState, StreamAPIBackend, SubSate};

#[rstest::rstest]
#[timeout(std::time::Duration::from_secs(5))]
#[test_log::test(tokio::test)]
async fn market_subscribtion() {
    let mock = StreamAPIBackend::new().await;
    let url = mock.url.clone();

    let h1 = tokio::spawn(async move {
        let (_client, async_task) = betfair_stream_api::StreamAPIProvider::new(
            std::borrow::Cow::Owned(ApplicationKey::new("app_key".to_string())),
            std::borrow::Cow::Owned(SessionToken::new("app_key".to_string())),
            BetfairUrl::new(std::borrow::Cow::Owned(url.clone())),
            HeartbeatStrategy::None,
        )
        .await
        .unwrap();
        let _ = async_task.await;
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
