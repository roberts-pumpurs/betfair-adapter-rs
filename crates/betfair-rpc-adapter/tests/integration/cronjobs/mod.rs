use rstest::fixture;

use crate::utils::{MockSettings, Server};

mod health_check;
mod keep_alive;

#[fixture]
pub async fn server_cron() -> Server {
    let settings = MockSettings {
        keep_alive_period: std::time::Duration::from_millis(500),
        health_check_period: std::time::Duration::from_millis(500),
    };

    Server::new_with_settings(settings).await
}
