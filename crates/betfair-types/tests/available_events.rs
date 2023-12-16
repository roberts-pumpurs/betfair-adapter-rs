#[test]
fn parse_fixture() {
    let data = std::fs::read_to_string("./tests/resources/availableevents.json").unwrap();
    serde_json::from_str::<betfair_types::types::sports_aping::list_events::ReturnType>(&data)
        .unwrap();
}
