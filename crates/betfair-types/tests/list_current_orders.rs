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

#[test]
fn parse_fixture_missing_matched_date() {
    use betfair_types::types::sports_aping::list_current_orders;
    use json_rpc_types::Response;

    // Betfair forum notes `matchedDate` is sometimes missing despite being documented as mandatory:
    // https://forum.developer.betfair.com/forum/sports-exchange-api/exchange-api/2473-cancelinstructionreport-sizecancelled-required-but-absent
    let data =
        std::fs::read_to_string("./tests/resources/list_current_orders_missing_matched_date.json")
            .unwrap();

    let response: Response<list_current_orders::ReturnType, serde_json::Value> =
        serde_json::from_str(&data).unwrap();

    let orders = response
        .payload
        .expect("response should be successful")
        .current_orders;

    assert_eq!(orders.len(), 1);
    assert!(orders[0].matched_date.is_none());
}
