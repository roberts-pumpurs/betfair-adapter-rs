use rstest::rstest;
use serde_json::json;
use test_log::test;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

use super::server_cron;
use crate::utils::{Server, APP_KEY, KEEP_ALIVE_URL, SESSION_TOKEN};

#[rstest]
#[test(tokio::test)]
async fn keep_alive(#[future] server_cron: Server) {
    let server_cron = server_cron.await;

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
        .expect(1) // IMPORTANT!
        .named("Keep alive")
        .mount(&server_cron.bf_api_mock_server)
        .await;

    // Action
    let client = server_cron.client().await;
    tokio::time::sleep(std::time::Duration::from_millis(800)).await;

    // Assert
    // The mock assertion will auto-trigger on test-end
    drop(client);
}
