#[test]
fn parse_fixture() {
    use betfair_types::types::sports_aping::list_competitions;
    use json_rpc_types::Response;
    let data = std::fs::read_to_string("./tests/resources/list_competitions.json").unwrap();
    serde_json::from_str::<Response<list_competitions::ReturnType, serde_json::Value>>(&data)
        .unwrap();
}
