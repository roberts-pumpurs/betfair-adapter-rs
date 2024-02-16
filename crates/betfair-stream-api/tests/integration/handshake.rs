use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_api::HeartbeatStrategy;
use betfair_stream_server_mock::{ClientState, StreamAPIBackend, SubSate};

#[rstest::rstest]
#[timeout(std::time::Duration::from_secs(5))]
#[test_log::test(tokio::test)]
async fn successful_handshake() {
    let mock = StreamAPIBackend::new().await;
    let url = mock.url.clone();

    let client_task = tokio::spawn(async move {
        let (_client, async_task, _output) = betfair_stream_api::StreamListener::new(
            ApplicationKey::new("app_key".to_string()),
            SessionToken::new("session_token".to_string()),
            url.into(),
            HeartbeatStrategy::None,
        )
        .await
        .unwrap();
        let _ = async_task.await;
    });

    let connection = mock.process_next().await;
    let conn_state = connection.state.clone();

    let mock_server = tokio::spawn(async move {
        connection.process().await;
    });

    // Sleep for 1 second to allow the connection to be established
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    assert!(!client_task.is_finished(), "the client should not be finished");
    assert!(!mock_server.is_finished(), "the server should not be finished");
    let conn_state = conn_state.lock().await;
    assert_eq!(
        *conn_state,
        ClientState::LoggedIn(SubSate {
            keep_alive_counter: 0
        })
    );
}
