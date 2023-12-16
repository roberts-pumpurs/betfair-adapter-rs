#[test]
fn parse_fixture_desc() {
    use betfair_types::types::sports_aping::list_current_orders;
    use json_rpc_types::Response;
    let data =
        std::fs::read_to_string("./tests/resources/list_current_orders_description.json").unwrap();
    serde_json::from_str::<Response<list_current_orders::ReturnType, serde_json::Value>>(&data)
        .unwrap();
}

#[test]
fn parse_fixture() {
    use betfair_types::types::sports_aping::list_current_orders;
    use json_rpc_types::Response;
    let data = std::fs::read_to_string("./tests/resources/list_current_orders.json").unwrap();
    serde_json::from_str::<Response<list_current_orders::ReturnType, serde_json::Value>>(&data)
        .unwrap();
}
