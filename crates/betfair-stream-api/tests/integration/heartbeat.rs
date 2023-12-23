use betfair_adapter::{ApplicationKey, BetfairUrl, SessionToken};
use betfair_stream_api::HeartbeatStrategy;

use crate::utils::{ClientState, StreamAPIBackend, SubSate};

#[rstest::rstest]
#[timeout(std::time::Duration::from_secs(5))]
#[test_log::test(tokio::test)]
async fn successful_heartbeat() {
    let mock = StreamAPIBackend::new().await;
    let url = mock.url.clone();
    let duration = std::time::Duration::from_millis(1800); // 1.8 seconds

    let h1 = tokio::spawn(async move {
        let (client, async_task) = betfair_stream_api::StreamAPIProvider::new(
            std::borrow::Cow::Owned(ApplicationKey::new("app_key".to_string())),
            std::borrow::Cow::Owned(SessionToken::new("app_key".to_string())),
            BetfairUrl::new(std::borrow::Cow::Owned(url.clone())),
            HeartbeatStrategy::Interval(duration),
        )
        .await
        .unwrap();
        async_task.await.unwrap();
        {
            let r = client.read().unwrap();
            assert!(!r.command_sender.is_closed());
            let a = &r.command_sender;
            panic!("Should not reach here {a:#?}");
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
    let conn_state = conn_state.read().await;
    assert_eq!(
        *conn_state,
        ClientState::LoggedIn(SubSate {
            keep_alive_counter: 1
        })
    );
}
