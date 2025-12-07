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
                "sizeCancelled": 0.0, // Changed from string to double as per https://betfair-developer-docs.atlassian.net/wiki/spaces/1smk3cen4v3lu3yomq5qye0ni/pages/2687465/Betting+Type+Definitions#CancelInstructionReport
                "instruction": {
                    "betId": "298537625817",
                }
            }
        ]
    });
    let _result = serde_json::from_value::<CancelExecutionReport>(data).unwrap();
}

#[test]
fn test_deserialize_missing_size_cancelled() {
    let data = json!({
        "customerRef": "0oxfjBrq8K2TZg2Ytqjo1",
        "status": "FAILURE",
        "errorCode": "BET_ACTION_ERROR",
        "marketId": "1.210878100",
        "instructionReports": [
            {
                "status": "FAILURE",
                "errorCode": "INVALID_BET_ID",
                // Betfair forum notes `sizeCancelled` can be missing even though docs say mandatory:
                // https://forum.developer.betfair.com/forum/sports-exchange-api/exchange-api/2473-cancelinstructionreport-sizecancelled-required-but-absent
                // Deliberately omit sizeCancelled to match Betfair behaviour.
                "instruction": {
                    "betId": "758934758934"
                }
            }
        ]
    });
    serde_json::from_value::<CancelExecutionReport>(data).unwrap();
}

#[test]
fn parse_fixture() {
    use betfair_types::types::sports_aping::cancel_orders;
    use json_rpc_types::Response;
    let data = std::fs::read_to_string("./tests/resources/cancel_orders.json").unwrap();
    serde_json::from_str::<Response<cancel_orders::ReturnType, serde_json::Value>>(&data).unwrap();
}
