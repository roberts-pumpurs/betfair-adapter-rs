use betfair_rpc_server_mock::Server;
use rstest::rstest;

#[rstest]
#[test_log::test(tokio::test)]
async fn keep_alive() {
    let server = Server::new().await;

    // Setup
    server
        .mock_keep_alive()
        .expect(1)
        .mount(&server.bf_api_mock_server)
        .await;

    // Action
    let client = server.client().await;
    let client = client.authenticate().await.unwrap();
    client.keep_alive().await.unwrap();
}
