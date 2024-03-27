use betfair_rpc_server_mock::{rpc_path, Server, APP_KEY, SESSION_TOKEN};
use betfair_types::customer_ref::CustomerRef;
use betfair_types::size::Size;
use betfair_types::types::sports_aping::{
    cancel_orders, BetId, CancelInstruction, CancelInstructionReport, ExecutionReportErrorCode,
    ExecutionReportStatus, InstructionReportErrorCode, InstructionReportStatus, MarketId,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

#[rstest]
#[test_log::test(tokio::test)]
async fn cancel_bets_unsuccessful() {
    let server = Server::new().await;

    // Setup
    let response = json!({
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

    Mock::given(method("POST"))
        .and(path(rpc_path::<cancel_orders::Parameters>()))
        .and(header("Accept", "application/json"))
        .and(header("X-Authentication", SESSION_TOKEN))
        .and(header("X-Application", APP_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(response.clone()))
        .expect(1)
        .named("single cancel order call")
        .mount(&server.bf_api_mock_server)
        .await;

    // Action
    let client = server.client().await;
    let client = client.authenticate().await.unwrap();
    let market_id = MarketId("1.210878100".to_string());
    let result = client
        .send_request(cancel_orders::Parameters {
            market_id: Some(market_id),
            instructions: Some(vec![CancelInstruction {
                bet_id: BetId("298537625817".to_string()),
                size_reduction: None,
            }]),
            customer_ref: None,
        })
        .await
        .unwrap();

    // Assert
    let expected = cancel_orders::ReturnType {
        customer_ref: Some(CustomerRef::new("0oxfjBrq8K2TZg2Ytqjo1".to_string()).unwrap()),
        error_code: Some(ExecutionReportErrorCode::BetActionError),
        instruction_reports: Some(vec![CancelInstructionReport {
            status: InstructionReportStatus::Failure,
            instruction: Some(CancelInstruction {
                bet_id: BetId("298537625817".to_string()),
                size_reduction: None,
            }),
            cancelled_date: None,
            error_code: Some(InstructionReportErrorCode::BetTakenOrLapsed),
            size_cancelled: Size::from(dec!(0.0)),
        }]),
        market_id: Some(MarketId("1.210878100".to_string())),
        status: ExecutionReportStatus::Failure,
    };
    assert_eq!(result, expected);
}
