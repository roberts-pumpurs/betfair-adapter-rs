#[test]
fn parse_fixture() {
    use betfair_types::types::sports_aping::replace_orders;
    use json_rpc_types::Response;
    let data = std::fs::read_to_string("./tests/resources/replace_orders.json").unwrap();
    serde_json::from_str::<Response<replace_orders::ReturnType, serde_json::Value>>(&data).unwrap();
}
