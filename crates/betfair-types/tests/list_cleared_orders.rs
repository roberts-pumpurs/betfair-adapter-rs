#[test]
fn parse_fixture() {
    use betfair_types::types::sports_aping::list_cleared_orders;
    use json_rpc_types::Response;
    let data = std::fs::read_to_string("./tests/resources/list_cleared_orders.json").unwrap();
    serde_json::from_str::<Response<list_cleared_orders::ReturnType, serde_json::Value>>(&data)
        .unwrap();
}
