#[test]
fn parse_fixture() {
    use betfair_types::types::sports_aping::list_events;
    use json_rpc_types::Response;
    let data = std::fs::read_to_string("./tests/resources/list_events.json").unwrap();
    serde_json::from_str::<Response<list_events::ReturnType, serde_json::Value>>(&data).unwrap();
}
