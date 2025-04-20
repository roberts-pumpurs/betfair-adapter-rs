use std::sync::Arc;

use betfair_rpc_server_mock::Server;
use betfair_types::types::sports_aping::{
    Competition, CompetitionId, CountryCode, Event, EventId, EventType, EventTypeId,
    MarketCatalogue, MarketDescription, MarketFilter, MarketId, MarketProjection, MarketType,
    PriceLadderDescription, PriceLadderType, RunnerCatalog, SelectionId, TimeRange,
    list_market_catalogue,
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
                "rules": "<br><br>MARKET INFORMATION</b><br><br>For further information please see <a href=http://content.betfair.com/aboutus/content.asp?sWhichKey=Rules%20and%20Regulations#undefined.do style=color:0163ad; text-decoration: underline; target=_blank>Rules & Regs</a>.<br><br>Who will win this match? This market includes overtime. At the start of play all unmatched bets will be cancelled and the market <b>turned in-play</b>. Please note that this market will not be actively managed, therefore it is the responsibility of all users to manage their in-play positions. Dead Heat rules apply.<br><br>Customers should be aware that:<b><br><br><li>Transmissions described as \u{201c}live\u{201d} by some broadcasters may actually be delayed and that all in-play matches are not necessarily televised.</li><br><li>The extent of any such delay may vary, depending on the set-up through which they are receiving pictures or data.</li><br></b><br>",
                "rulesHasDate": false,
                "priceLadderDescription": {
                    "type": "CLASSIC"
                }
            },
            "totalMatched": 0.0,
            "runners": [
                {
                    "selectionId": 12_062_411,
                    "runnerName": "Atomeromu Szekszard Women",
                    "handicap": 0.0,
                    "sortPriority": 1
                },
                {
                    "selectionId": 50_310_375,
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
    let market_id = MarketId(Arc::new("1.210878100".to_owned()));
    let result = client
        .send_request(
            list_market_catalogue::Parameters::builder()
                .max_results(2)
                .filter(
                    MarketFilter::builder()
                        .event_type_ids(vec![EventTypeId(Arc::new("31908334".to_owned()))])
                        .market_start_time(TimeRange {
                            from: "2022-11-15T17:00:00.000Z".parse().ok(),
                            to: "2022-11-15T17:00:00.000Z".parse().ok(),
                        })
                        .market_type_codes(vec![MarketType(Arc::new("MATCH_ODDS".to_owned()))])
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
        market_id: MarketId(Arc::new("1.206502771".to_owned())),
        total_matched: dec!(0).into(),
        market_start_time: "2022-11-15T17:00:00.000Z".parse().ok(),
        event: Some(Event {
            country_code: Some(CountryCode(Arc::new("GB".to_owned()))),
            id: Some(EventId(Arc::new("31908334".to_owned()))),
            name: Some(Arc::new("Atomeromu Szekszard Women v Olympiakos Piraeus BC ".to_owned())),
            open_date: Some("2022-11-15T17:00:00.000Z".parse().unwrap()),
            timezone: Some(Arc::new("GMT".to_owned())),
            venue: None,
        }),
        event_type: Some(EventType {
            id: Some(EventTypeId(Arc::new("1123123".to_owned()))),
            name: Some(Arc::new("Basketball".to_owned())),
        }),
        competition: Some(Competition {
            id: Some(CompetitionId(Arc::new("8347200".to_owned()))),
            name: Some(Arc::new("Euroleague Women".to_owned())),
        }),
        runners: Some(vec![
            RunnerCatalog {
                selection_id: SelectionId(12_062_411),
                runner_name: Arc::new("Atomeromu Szekszard Women".to_owned()),
                handicap: dec!(0),
                sort_priority: 1,
                metadata: None,
            },
            RunnerCatalog {
                selection_id: SelectionId(50_310_375),
                runner_name: Arc::new("Olympiakos Piraeus BC".to_owned()),
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
            betting_type: Arc::new("ODDS".to_owned()),
            turn_in_play_enabled: true,
            market_type: MarketType(Arc::new("MATCH_ODDS".to_owned())),
            regulator: Some(Arc::new("MALTA LOTTERIES AND GAMBLING AUTHORITY".to_owned())),
            market_base_rate: Some(dec!(2.0)),
            discount_allowed: Some(false),
            wallet: Some(Arc::new("UK wallet".to_owned()),),
            rules: Some(
                Arc::new("<br><br>MARKET INFORMATION</b><br><br>For further information please see <a href=http://content.betfair.com/aboutus/content.asp?sWhichKey=Rules%20and%20Regulations#undefined.do style=color:0163ad; text-decoration: underline; target=_blank>Rules & Regs</a>.<br><br>Who will win this match? This market includes overtime. At the start of play all unmatched bets will be cancelled and the market <b>turned in-play</b>. Please note that this market will not be actively managed, therefore it is the responsibility of all users to manage their in-play positions. Dead Heat rules apply.<br><br>Customers should be aware that:<b><br><br><li>Transmissions described as \u{201c}live\u{201d} by some broadcasters may actually be delayed and that all in-play matches are not necessarily televised.</li><br><li>The extent of any such delay may vary, depending on the set-up through which they are receiving pictures or data.</li><br></b><br>".to_owned()),
            ),
            rules_has_date: Some(false),
            clarifications: None,
            each_way_divisor: None,
            line_range_info: None,
            race_type: None,
            price_ladder_description: Some(PriceLadderDescription { r_type: PriceLadderType::Classic }),
        }),
        market_name: Arc::new("Moneyline".to_owned()),
    };
    assert_eq!(result, vec![market]);
}
