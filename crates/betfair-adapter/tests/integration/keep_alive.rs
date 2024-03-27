use betfair_rpc_server_mock::{Server, APP_KEY, KEEP_ALIVE_URL, SESSION_TOKEN};
use rstest::rstest;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

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
