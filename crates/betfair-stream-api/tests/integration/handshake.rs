use betfair_adapter::jurisdiction::CustomUrl;
use betfair_adapter::{ApplicationKey, SessionToken};
use betfair_stream_api::{BetfairProviderExt, HeartbeatStrategy};
use betfair_stream_server_mock::{ClientState, StreamAPIBackend, SubSate};

#[rstest::rstest]
#[timeout(std::time::Duration::from_secs(5))]
#[test_log::test(tokio::test)]
async fn successful_handshake() {
    let mock = StreamAPIBackend::new().await;
    let url = mock.url.clone().into();

    let client_task = tokio::spawn(async move {
        let bf_mock = betfair_rpc_server_mock::Server::new_with_stream_url(url).await;
        let client = bf_mock.client().await;
        let authenticated = client.authenticate().await.unwrap();
        let (_client, async_task, _output) = authenticated.connect_to_stream().await.unwrap();
        async_task.await
    });

    let connection = mock.process_next().await;
    let conn_state = connection.state.clone();

    let mock_server = tokio::spawn(async move {
        connection.process().await;
    });

    // Sleep for 1 second to allow the connection to be established
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    assert!(
        !client_task.is_finished(),
        "the client should not be finished"
    );
    assert!(
        !mock_server.is_finished(),
        "the server should not be finished"
    );
    let conn_state = conn_state.lock().await;
    assert_eq!(
        *conn_state,
        ClientState::LoggedIn(SubSate {
            keep_alive_counter: 0
        })
    );
}
