#[test]
fn parse_fixture() {
    use betfair_types::types::sports_aping::list_market_book;
    use json_rpc_types::Response;
    let data = std::fs::read_to_string("./tests/resources/list_market_book.json").unwrap();
    serde_json::from_str::<Response<list_market_book::ReturnType, serde_json::Value>>(&data)
        .unwrap();
}
