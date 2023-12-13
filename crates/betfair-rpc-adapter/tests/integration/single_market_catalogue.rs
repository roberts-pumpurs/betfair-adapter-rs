use betfair_adapter::traits::IBetfairAdapter;
use betfair_adapter::types::competition::Competition;
use betfair_adapter::types::event::Event;
use betfair_adapter::types::event_type::EventType;
use betfair_adapter::types::market_catalogue::MarketCatalogue;
use betfair_adapter::types::market_catalogue_runner::MarketCatalogueRunner;
use betfair_adapter::types::market_id::MarketId;
use betfair_adapter::types::selection_id::SelectionId;
use rstest::rstest;
use rust_decimal_macros::dec;
use serde_json::json;
use test_log::test;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::utils::{server, Server, Transform, APP_KEY, REST_URL, SESSION_TOKEN};

#[rstest]
#[test(tokio::test)]
async fn single_market_catalogue(#[future] server: Server) {
    let server = server.await;

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

    Mock::given(method("POST"))
        .and(path(format!("{REST_URL:}/listMarketCatalogue/")))
        .and(header("Accept", "application/json"))
        .and(header("X-Authentication", SESSION_TOKEN))
        .and(header("X-Application", APP_KEY))
        .respond_with(ResponseTemplate::new(200).set_body_json(response.clone()))
        .expect(1)
        .named("single market book call")
        .mount(&server.bf_api_mock_server)
        .await;

    // Action
    let client = server.client().await;
    let result = client.single_market_catalogue(MarketId("1.23456789".to_string())).await.unwrap();

    // Assert
    let expected = MarketCatalogue {
        market_id: MarketId("1.206502771".to_string()),
        total_matched: dec!(0.0).into(),
        market_start_time: "2022-11-15T17:00:00.000Z".transform(),
        event: Some(Event {
            country_code: Some("GB".to_string()),
            id: "31908334".to_string(),
            name: "Atomeromu Szekszard Women v Olympiakos Piraeus BC ".to_string(),
            open_date: "2022-11-15T17:00:00.000Z".transform(),
        }),
        event_type: Some(EventType { id: "1123123".to_string(), name: "Basketball".to_string() }),
        competition: Some(Competition {
            id: "8347200".to_string(),
            name: "Euroleague Women".to_string(),
        }),
        runners: vec![
            MarketCatalogueRunner {
                name: "Atomeromu Szekszard Women".to_string(),
                selection_id: SelectionId::new(12062411),
            },
            MarketCatalogueRunner {
                name: "Olympiakos Piraeus BC".to_string(),
                selection_id: SelectionId::new(50310375),
            },
        ],
    };
    assert_eq!(result, expected);
}
