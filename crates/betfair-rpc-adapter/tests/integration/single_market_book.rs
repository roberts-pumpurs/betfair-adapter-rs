use betfair_adapter::traits::IBetfairAdapter;
use betfair_adapter::types::market_book::MarketBook;
use betfair_adapter::types::market_book_runner::MarketBookRunner;
use betfair_adapter::types::market_id::MarketId;
use betfair_adapter::types::price::{Informative, Price};
use betfair_adapter::types::price_size::PriceSize;
use betfair_adapter::types::selection_id::SelectionId;
use rstest::rstest;
use rust_decimal_macros::dec;
use serde_json::json;
use test_log::test;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

use crate::utils::{server, Server, APP_KEY, REST_URL, SESSION_TOKEN};

#[rstest]
#[test(tokio::test)]
async fn single_market_book(#[future] server: Server) {
    let server = server.await;

    let response = json!([
        {
            "marketId": "1.206502771",
            "isMarketDataDelayed": false,
            "status": "OPEN",
            "betDelay": 0,
            "bspReconciled": false,
            "complete": true,
            "inplay": false,
            "numberOfWinners": 1,
            "numberOfRunners": 2,
            "numberOfActiveRunners": 2,
            "totalMatched": 0.0,
            "totalAvailable": 153.6,
            "crossMatching": true,
            "runnersVoidable": false,
            "version": 4910,
            "runners": [
                {
                    "selectionId": 12062411,
                    "handicap": 0.0,
                    "status": "ACTIVE",
                    "totalMatched": 0.0,
                    "ex": {
                        "availableToBack": [
                            {
                                "price": 1.19,
                                "size": 57.52
                            },
                            {
                                "price": 1.02,
                                "size": 6.48
                            },
                            {
                                "price": 1.01,
                                "size": 21.14
                            }
                        ],
                        "availableToLay": [
                            {
                                "price": 1.43,
                                "size": 31.11
                            }
                        ],
                        "tradedVolume": []
                    }
                },
                {
                    "selectionId": 50310375,
                    "handicap": 0.0,
                    "status": "ACTIVE",
                    "totalMatched": 0.0,
                    "ex": {
                        "availableToBack": [
                            {
                                "price": 3.75,
                                "size": 5.21
                            },
                            {
                                "price": 1.02,
                                "size": 6.65
                            },
                            {
                                "price": 1.01,
                                "size": 21.14
                            }
                        ],
                        "availableToLay": [
                            {
                                "price": 4.8,
                                "size": 4.37
                            }
                        ],
                        "tradedVolume": []
                    }
                }
            ]
        }
    ]);

    Mock::given(method("POST"))
        .and(path(format!("{REST_URL:}/listMarketBook/")))
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
    let result = client.single_market_book(MarketId("1.23456789".to_string())).await.unwrap();

    // Assert
    let expected = MarketBook {
        market_id: MarketId("1.206502771".to_string()),
        bet_delay: 0,
        is_market_data_delayed: false,
        in_play: false,
        complete: true,
        total_matched: dec!(0.0).into(),
        total_available: dec!(153.6).into(),
        runners: vec![
            MarketBookRunner {
                selection_id: SelectionId::new(12062411),
                available_to_back: vec![
                    PriceSize { price: Price::<Informative>::new_f64(1.19), size: 57.52.into() },
                    PriceSize { price: Price::<Informative>::new_f64(1.02), size: 6.48.into() },
                    PriceSize { price: Price::<Informative>::new_f64(1.01), size: 21.14.into() },
                ],
                available_to_lay: vec![PriceSize {
                    price: Price::<Informative>::new_f64(1.43),
                    size: 31.11.into(),
                }],
            },
            MarketBookRunner {
                available_to_back: vec![
                    PriceSize { price: Price::<Informative>::new_f64(3.75), size: 5.21.into() },
                    PriceSize { price: Price::<Informative>::new_f64(1.02), size: 6.65.into() },
                    PriceSize { price: Price::<Informative>::new_f64(1.01), size: 21.14.into() },
                ],
                available_to_lay: vec![PriceSize {
                    price: Price::<Informative>::new_f64(4.8),
                    size: 4.37.into(),
                }],
                selection_id: SelectionId::new(50310375),
            },
        ],
    };
    assert_eq!(result, expected);
}
