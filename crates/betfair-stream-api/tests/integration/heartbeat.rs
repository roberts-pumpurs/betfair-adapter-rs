use betfair_stream_api::{BetfairProviderExt, HeartbeatStrategy};
use betfair_stream_server_mock::{ClientState, StreamAPIBackend, SubSate};
use tokio_stream::StreamExt;

#[rstest::rstest]
#[timeout(std::time::Duration::from_secs(5))]
#[test_log::test(tokio::test)]
async fn successful_heartbeat() {
    let mock = StreamAPIBackend::new().await;

    let h1 = tokio::spawn({
        let duration = std::time::Duration::from_millis(800); // 0.8 seconds
        let url = mock.url.clone().into();
        async move {
            let bf_mock = betfair_rpc_server_mock::Server::new_with_stream_url(url).await;
            let client = bf_mock.client().await;
            let mut stream_api_abi = client
                .connect_to_stream_with_hb(HeartbeatStrategy::Interval(duration))
                .await;

            let mut stream = stream_api_abi.run_with_default_runtime();
            while let Some(value) = stream.next().await {
                tracing::info!(?value, "received vaue from stream");
            }
        }
    });

    let connection = mock.process_next().await;
    let conn_state = connection.state.clone();

    let h2 = tokio::spawn(async move {
        connection.process().await;
    });

    // Sleep for 1 second to allow the connection to be established
    tokio::time::sleep(std::time::Duration::from_secs(2)).await;

    assert!(!h1.is_finished());
    assert!(!h2.is_finished());
    let conn_state = conn_state.lock().await;
    assert_eq!(
        *conn_state,
        ClientState::LoggedIn(SubSate {
            heartbeat_counter: 2
        })
    );
}
