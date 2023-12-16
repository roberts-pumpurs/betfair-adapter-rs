use betfair_types::keep_alive;

#[test]
fn parse_fixture_fail() {
    let data = std::fs::read_to_string("./tests/resources/keep_alive_fail.json").unwrap();
    let _res = serde_json::from_str::<keep_alive::Response>(&data)
        .unwrap()
        .0
        .unwrap_err();
}

#[test]
fn parse_fixture_success() {
    let data = std::fs::read_to_string("./tests/resources/keep_alive_success.json").unwrap();
    let _res = serde_json::from_str::<keep_alive::Response>(&data)
        .unwrap()
        .0
        .unwrap();
}
