use std::sync::Arc;

use betfair_stream_api::{BetfairProviderExt, ExternalUpdates, MetadataUpdates, PostAuthMessages};
use betfair_stream_server_mock::{ClientState, StreamAPIBackend, SubSate};
use futures::StreamExt;
use futures_concurrency::stream::IntoStream;
use tokio::sync::Mutex;

#[rstest::rstest]
#[timeout(std::time::Duration::from_secs(5))]
#[test_log::test(tokio::test)]
async fn successful_handshake() {
    let mock = StreamAPIBackend::new().await;
    let url = mock.url.clone().into();

    let messages = Arc::new(Mutex::new(Vec::new()));
    let client_task = tokio::spawn({
        let mut messages = Arc::clone(&messages);
        async move {
            let bf_mock = betfair_rpc_server_mock::Server::new_with_stream_url(url).await;
            let client = bf_mock.client().await;
            let authenticated = client.authenticate().await.unwrap();
            let mut stream_api_abi = authenticated.connect_to_stream().await;
            let mut stream = stream_api_abi.run_with_default_runtime();
            while let Some(value) = stream.next().await {
                tracing::info!(?value, "received vaue from stream");
                let mut w = messages.as_ref().lock().await;
                w.push(value);
                drop(w);
            }
        }
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
            heartbeat_counter: 0
        })
    );
    let messages = messages.lock().await;
    assert!(matches!(
        messages.get(0).unwrap().clone(),
        ExternalUpdates::Metadata(MetadataUpdates::TcpConnected)
    ));
    assert!(matches!(
        messages.get(1).unwrap().clone(),
        ExternalUpdates::Layer(PostAuthMessages::ConnectionMessage(..))
    ));
    assert!(matches!(
        messages.get(2).unwrap().clone(),
        ExternalUpdates::Metadata(MetadataUpdates::Authenticated { .. })
    ));
}
