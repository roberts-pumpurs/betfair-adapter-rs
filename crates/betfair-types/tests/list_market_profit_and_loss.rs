#[test]
fn parse_fixture() {
    use betfair_types::types::sports_aping::list_market_profit_and_loss;
    use json_rpc_types::Response;
    let data =
        std::fs::read_to_string("./tests/resources/list_market_profit_and_loss.json").unwrap();
    serde_json::from_str::<Response<list_market_profit_and_loss::ReturnType, serde_json::Value>>(
        &data,
    )
    .unwrap();
}
