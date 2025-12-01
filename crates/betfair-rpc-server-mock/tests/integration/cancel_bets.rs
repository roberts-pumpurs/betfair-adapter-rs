use betfair_rpc_server_mock::Server;
use betfair_types::customer_ref::CustomerRef;
use betfair_types::num;
use betfair_types::size::Size;
use betfair_types::types::sports_aping::{
    BetId, CancelInstruction, CancelInstructionReport, ExecutionReportErrorCode,
    ExecutionReportStatus, InstructionReportErrorCode, InstructionReportStatus, MarketId,
    cancel_orders,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use serde_json::json;

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
                "sizeCancelled": 0.0, // Changed from string to double as per https://betfair-developer-docs.atlassian.net/wiki/spaces/1smk3cen4v3lu3yomq5qye0ni/pages/2687465/Betting+Type+Definitions#CancelInstructionReport
                "instruction": {
                    "betId": "298537625817",
                }
        },
            {
                "status": "FAILURE",
                "errorCode": "INVALID_BET_ID",
                // Missing `sizeCancelled` field.
                "instruction": {
                    "betId": "758934758934",
                }
            }
        ]
    });
    server
        .mock_authenticated_rpc_from_json::<cancel_orders::Parameters>(response)
        .expect(1)
        .mount(&server.bf_api_mock_server)
        .await;

    // Action
    let client = server.client().await;
    let (client, _) = client.authenticate().await.unwrap();
    let market_id = MarketId::new("1.210878100");
    let result = client
        .send_request(cancel_orders::Parameters {
            market_id: Some(market_id),
            instructions: Some(vec![
                CancelInstruction {
                    bet_id: BetId::new("298537625817"),
                    size_reduction: None,
                },
                CancelInstruction {
                    bet_id: BetId::new("758934758934"),
                    size_reduction: None,
                },
            ]),
            customer_ref: None,
        })
        .await
        .unwrap();

    // Assert
    let expected = cancel_orders::ReturnType {
        customer_ref: Some(CustomerRef::new("0oxfjBrq8K2TZg2Ytqjo1".to_owned()).unwrap()),
        error_code: Some(ExecutionReportErrorCode::BetActionError),
        instruction_reports: Some(vec![
            CancelInstructionReport {
                status: InstructionReportStatus::Failure,
                instruction: Some(CancelInstruction {
                    bet_id: BetId::new("298537625817"),
                    size_reduction: None,
                }),
                cancelled_date: None,
                error_code: Some(InstructionReportErrorCode::BetTakenOrLapsed),
                size_cancelled: Option::Some(Size::from(num!(0.0))),
            },
            CancelInstructionReport {
                status: InstructionReportStatus::Failure,
                instruction: Some(CancelInstruction {
                    bet_id: BetId::new("758934758934"),
                    size_reduction: None,
                }),
                cancelled_date: None,
                error_code: Some(InstructionReportErrorCode::InvalidBetId),
                size_cancelled: None,
            },
        ]),
        market_id: Some(MarketId::new("1.210878100")),
        status: ExecutionReportStatus::Failure,
    };
    assert_eq!(result, expected);
}
