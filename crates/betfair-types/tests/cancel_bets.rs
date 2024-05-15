use betfair_types::types::sports_aping::CancelExecutionReport;
use serde_json::json;

#[test]
fn test_deserialize() {
    let data = b"{\"customerRef\":\"0oxfjBrq8K2TZg2Ytqjo1\",\"errorCode\":\"BET_ACTION_ERROR\",\"marketId\":\"1.210878100\",\"status\":\"FAILURE\"}";
    let result: Result<CancelExecutionReport, _> = serde_json::from_slice(data);
    result.unwrap();
}

#[test]
fn test_deserialize2() {
    let data = json!({
        "customerRef": "0oxfjBrq8K2TZg2Ytqjo1",
        "status": "FAILURE",
        "errorCode": "BET_ACTION_ERROR",
        "marketId": "1.210878100",
        "instructionReports": [
            {
                "status": "FAILURE",
                "errorCode": "BET_TAKEN_OR_LAPSED",
                "sizeCancelled": "0.0",
                "instruction": {
                    "betId": "298537625817",
                }
            }
        ]
    });
    let _result = serde_json::from_value::<CancelExecutionReport>(data).unwrap();
}

#[test]
fn parse_fixture() {
    use betfair_types::types::sports_aping::cancel_orders;
    use json_rpc_types::Response;
    let data = std::fs::read_to_string("./tests/resources/cancel_orders.json").unwrap();
    serde_json::from_str::<Response<cancel_orders::ReturnType, serde_json::Value>>(&data).unwrap();
}
