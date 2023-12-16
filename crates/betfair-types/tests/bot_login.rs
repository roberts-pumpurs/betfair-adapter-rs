#[test]
fn parse_fixture_1() {
    use betfair_types::bot_login::BotLoginResponse;
    let data = std::fs::read_to_string("./tests/resources/bot_login_fail.json").unwrap();
    serde_json::from_str::<BotLoginResponse>(&data).unwrap();
}

#[test]
fn parse_fixture_2() {
    use betfair_types::bot_login::BotLoginResponse;
    let data = std::fs::read_to_string("./tests/resources/bot_login_success.json").unwrap();
    serde_json::from_str::<BotLoginResponse>(&data).unwrap();
}
