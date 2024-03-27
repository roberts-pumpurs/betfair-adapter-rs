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
    let response = json!(
        {
            "token":"SESSIONTOKEN",
            "product":"AppKey",
            "status":"SUCCESS",
            "error":""
        }
    );
    Mock::given(method("GET"))
        .and(path(KEEP_ALIVE_URL))
        .and(header("Accept", "application/json"))
        .and(header("X-Authentication", SESSION_TOKEN))
        .and(header("X-Application", APP_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(response))
        .expect(1)
        .named("Keep alive")
        .mount(&server.bf_api_mock_server)
        .await;

    // Action
    let client = server.client().await;
    let client = client.authenticate().await.unwrap();
    client.keep_alive().await.unwrap();
}
