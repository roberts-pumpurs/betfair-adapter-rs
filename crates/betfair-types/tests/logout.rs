use betfair_types::logout;

#[test]
fn parse_fixture_success() {
    let data = std::fs::read_to_string("./tests/resources/logout_success.json").unwrap();
    let _res = serde_json::from_str::<logout::Response>(&data)
        .unwrap()
        .0
        .unwrap();
}

#[test]
fn parse_fixture_fail() {
    let data = std::fs::read_to_string("./tests/resources/logout_fail.json").unwrap();
    let _res = serde_json::from_str::<logout::Response>(&data)
        .unwrap()
        .0
        .unwrap_err();
}
