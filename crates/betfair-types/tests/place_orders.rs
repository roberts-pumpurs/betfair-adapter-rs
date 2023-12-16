#[test]
fn parse_fixture() {
    use betfair_types::types::sports_aping::place_orders;
    use json_rpc_types::Response;
    let data = std::fs::read_to_string("./tests/resources/place_orders.json").unwrap();
    serde_json::from_str::<Response<place_orders::ReturnType, serde_json::Value>>(&data).unwrap();
}
