use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};

use crate::utils::{ClientState, StreamAPIBackend};

#[rstest::rstest]
#[timeout(std::time::Duration::from_secs(3))]
#[test_log::test(tokio::test)]
async fn successful_handshake() {
    let mock = StreamAPIBackend::new().await;
    let url = mock.url.clone();

    tokio::spawn(async move {
        let (_client, _async_task) = betfair_stream_api::StreamAPIProvider::new(
            std::borrow::Cow::Owned(ApplicationKey::new("app_key".to_string())),
            std::borrow::Cow::Owned(SessionToken::new("app_key".to_string())),
            BetfairUrl::new(std::borrow::Cow::Owned(url.clone())),
        )
        .await
        .unwrap();
    });

    let connection = mock.process_next().await;
    let conn_state = connection.state.clone();

    tokio::spawn(async move {
        connection.process().await;
    });

    // Sleep for 1 second to allow the connection to be established
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    let conn_state = conn_state.read().await;
    assert_eq!(
        *conn_state,
        ClientState::LoggedIn(crate::utils::SubSate::WaitingForSub)
    );
}
