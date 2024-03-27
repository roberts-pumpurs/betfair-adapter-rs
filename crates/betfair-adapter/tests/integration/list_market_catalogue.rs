use betfair_rpc_server_mock::Server;
use betfair_types::types::sports_aping::{
    list_market_catalogue, Competition, CompetitionId, CountryCode, Event, EventId, EventType,
    EventTypeId, MarketCatalogue, MarketDescription, MarketFilter, MarketId, MarketProjection,
    MarketType, PriceLadderDescription, PriceLadderType, RunnerCatalog, SelectionId, TimeRange,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use rust_decimal_macros::dec;
use serde_json::json;

#[rstest]
#[test_log::test(tokio::test)]
async fn list_market_catalogue() {
    let server = Server::new().await;

    // Setup
    let response = json!([
        {
            "marketId": "1.206502771",
            "marketName": "Moneyline",
            "marketStartTime": "2022-11-15T17:00:00.000Z",
            "description": {
                "persistenceEnabled": true,
                "bspMarket": false,
                "marketTime": "2022-11-15T17:00:00.000Z",
                "suspendTime": "2022-11-15T17:00:00.000Z",
                "bettingType": "ODDS",
                "turnInPlayEnabled": true,
                "marketType": "MATCH_ODDS",
                "regulator": "MALTA LOTTERIES AND GAMBLING AUTHORITY",
                "marketBaseRate": 2.0,
                "discountAllowed": false,
                "wallet": "UK wallet",
                "rules": "<br><br>MARKET INFORMATION</b><br><br>For further information please see <a href=http://content.betfair.com/aboutus/content.asp?sWhichKey=Rules%20and%20Regulations#undefined.do style=color:0163ad; text-decoration: underline; target=_blank>Rules & Regs</a>.<br><br>Who will win this match? This market includes overtime. At the start of play all unmatched bets will be cancelled and the market <b>turned in-play</b>. Please note that this market will not be actively managed, therefore it is the responsibility of all users to manage their in-play positions. Dead Heat rules apply.<br><br>Customers should be aware that:<b><br><br><li>Transmissions described as “live” by some broadcasters may actually be delayed and that all in-play matches are not necessarily televised.</li><br><li>The extent of any such delay may vary, depending on the set-up through which they are receiving pictures or data.</li><br></b><br>",
                "rulesHasDate": false,
                "priceLadderDescription": {
                    "type": "CLASSIC"
                }
            },
            "totalMatched": 0.0,
            "runners": [
                {
                    "selectionId": 12062411,
                    "runnerName": "Atomeromu Szekszard Women",
                    "handicap": 0.0,
                    "sortPriority": 1
                },
                {
                    "selectionId": 50310375,
                    "runnerName": "Olympiakos Piraeus BC",
                    "handicap": 0.0,
                    "sortPriority": 2
                }
            ],
            "competition": {
                "id": "8347200",
                "name": "Euroleague Women"
            },
            "eventType": {
                "id": "1123123",
                "name": "Basketball"
            },
            "event": {
                "id": "31908334",
                "name": "Atomeromu Szekszard Women v Olympiakos Piraeus BC ",
                "countryCode": "GB",
                "timezone": "GMT",
                "openDate": "2022-11-15T17:00:00.000Z"
            }
        }
    ]);
    server
        .mock_authenticated_rpc_from_json::<list_market_catalogue::Parameters>(response)
        .expect(1)
        .mount(&server.bf_api_mock_server)
        .await;

    // Action
    let client = server.client().await;
    let client = client.authenticate().await.unwrap();
    let market_id = MarketId("1.210878100".to_string());
    let result = client
        .send_request(
            list_market_catalogue::Parameters::builder()
                .max_results(2)
                .filter(
                    MarketFilter::builder()
                        .event_type_ids(vec![EventTypeId("31908334".to_string())])
                        .market_start_time(TimeRange {
                            from: "2022-11-15T17:00:00.000Z".parse().ok(),
                            to: "2022-11-15T17:00:00.000Z".parse().ok(),
                        })
                        .market_type_codes(vec![MarketType("MATCH_ODDS".to_string())])
                        .market_ids(vec![market_id])
                        .build(),
                )
                .market_projection(vec![
                    MarketProjection::Competition,
                    MarketProjection::Event,
                    MarketProjection::MarketStartTime,
                    MarketProjection::MarketDescription,
                    MarketProjection::RunnerDescription,
                    MarketProjection::EventType,
                ])
                .build(),
        )
        .await
        .unwrap();

    // Assert
    let market = MarketCatalogue {
        market_id: MarketId("1.206502771".to_string()),
        total_matched: dec!(0).into(),
        market_start_time: "2022-11-15T17:00:00.000Z".parse().ok(),
        event: Some(Event {
            country_code: Some(CountryCode("GB".to_string())),
            id: Some(EventId("31908334".to_string())),
            name: Some("Atomeromu Szekszard Women v Olympiakos Piraeus BC ".to_string()),
            open_date: Some("2022-11-15T17:00:00.000Z".parse().unwrap()),
            timezone: Some("GMT".to_string()),
            venue: None,
        }),
        event_type: Some(EventType {
            id: Some(EventTypeId("1123123".to_string())),
            name: Some("Basketball".to_string()),
        }),
        competition: Some(Competition {
            id: Some(CompetitionId("8347200".to_string())),
            name: Some("Euroleague Women".to_string()),
        }),
        runners: Some(vec![
            RunnerCatalog {
                selection_id: SelectionId(12062411),
                runner_name: "Atomeromu Szekszard Women".to_string(),
                handicap: dec!(0),
                sort_priority: 1,
                metadata: None,
            },
            RunnerCatalog {
                selection_id: SelectionId(50310375),
                runner_name: "Olympiakos Piraeus BC".to_string(),
                handicap: dec!(0),
                sort_priority: 2,
                metadata: None,
            },
        ]),
        description: Some(MarketDescription {
            persistence_enabled: Some(true),
            bsp_market: false,
            market_time: "2022-11-15T17:00:00.000Z".parse().unwrap(),
            suspend_time: "2022-11-15T17:00:00.000Z".parse().unwrap(),
            settle_time: None,
            betting_type: "ODDS".to_string(),
            turn_in_play_enabled: true,
            market_type: MarketType("MATCH_ODDS".to_string()),
            regulator: Some("MALTA LOTTERIES AND GAMBLING AUTHORITY".to_string()),
            market_base_rate: Some(dec!(2.0)),
            discount_allowed: Some(false),
            wallet: Some("UK wallet".to_string(),),
            rules: Some(
                "<br><br>MARKET INFORMATION</b><br><br>For further information please see <a href=http://content.betfair.com/aboutus/content.asp?sWhichKey=Rules%20and%20Regulations#undefined.do style=color:0163ad; text-decoration: underline; target=_blank>Rules & Regs</a>.<br><br>Who will win this match? This market includes overtime. At the start of play all unmatched bets will be cancelled and the market <b>turned in-play</b>. Please note that this market will not be actively managed, therefore it is the responsibility of all users to manage their in-play positions. Dead Heat rules apply.<br><br>Customers should be aware that:<b><br><br><li>Transmissions described as “live” by some broadcasters may actually be delayed and that all in-play matches are not necessarily televised.</li><br><li>The extent of any such delay may vary, depending on the set-up through which they are receiving pictures or data.</li><br></b><br>"
                    .to_string(),
            ),
            rules_has_date: Some(false),
            clarifications: None,
            each_way_divisor: None,
            line_range_info: None,
            race_type: None,
            price_ladder_description: Some(PriceLadderDescription { r_type: PriceLadderType::Classic }),
        }),
        market_name: "Moneyline".to_string(),
    };
    assert_eq!(result, vec![market]);
}
