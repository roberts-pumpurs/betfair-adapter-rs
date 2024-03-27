use betfair_rpc_server_mock::{rpc_path, Server, APP_KEY, SESSION_TOKEN};
use betfair_types::price::Price;
use betfair_types::size::Size;
use betfair_types::types::sports_aping::{
    list_market_book, ExchangePrices, MarketBook, MarketId, PriceData, PriceProjection, PriceSize,
    Runner, RunnerStatus, SelectionId,
};
use pretty_assertions::assert_eq;
use rstest::rstest;
use rust_decimal_macros::dec;
use serde_json::json;
use wiremock::matchers::{header, method, path};
use wiremock::{Mock, ResponseTemplate};

#[rstest]
#[test_log::test(tokio::test)]
async fn single_market_book() {
    let server = Server::new().await;

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
    server
        .mock_authenticated_rpc_from_json::<list_market_book::Parameters>(response)
        .expect(1)
        .mount(&server.bf_api_mock_server)
        .await;

    // Action
    let client = server.client().await;
    let client = client.authenticate().await.unwrap();

    let result = client
        .send_request(
            list_market_book::Parameters::builder()
                .market_ids(vec![MarketId("1.206502771".to_string())])
                .price_projection(
                    PriceProjection::builder()
                        .virtualise(true)
                        .price_data(vec![
                            PriceData::SpAvailable,
                            PriceData::SpTraded,
                            PriceData::ExBestOffers,
                        ])
                        .build(),
                )
                .build(),
        )
        .await
        .unwrap();

    // Assert
    assert_eq!(result.len(), 1);
    let result = result.first().unwrap();
    let expected = MarketBook {
        market_id: MarketId("1.206502771".to_string()),
        is_market_data_delayed: false,
        status: Some(betfair_types::types::sports_aping::MarketStatus::Open),
        bet_delay: Some(0),
        bsp_reconciled: Some(false),
        complete: Some(true),
        inplay: Some(false),
        number_of_winners: Some(1),
        number_of_runners: Some(2),
        number_of_active_runners: Some(2),
        last_match_time: None,
        total_matched: Some(dec!(0)),
        total_available: Some(dec!(153.6)),
        cross_matching: Some(true),
        runners_voidable: Some(false),
        version: Some(4910),
        runners: Some(vec![
            betfair_types::types::sports_aping::Runner {
                selection_id: betfair_types::types::sports_aping::SelectionId(12062411),
                handicap: dec!(0),
                status: betfair_types::types::sports_aping::RunnerStatus::Active,
                adjustment_factor: None,
                last_price_traded: None,
                total_matched: Some(dec!(0)),
                removal_date: None,
                sp: None,
                ex: Some(betfair_types::types::sports_aping::ExchangePrices {
                    available_to_back: Some(vec![
                        betfair_types::types::sports_aping::PriceSize {
                            price: Price::new(dec!(1.19)).unwrap(),
                            size: Size::new(dec!(57.52)),
                        },
                        betfair_types::types::sports_aping::PriceSize {
                            price: Price::new(dec!(1.02)).unwrap(),
                            size: Size::new(dec!(6.48)),
                        },
                        betfair_types::types::sports_aping::PriceSize {
                            price: Price::new(dec!(1.01)).unwrap(),
                            size: Size::new(dec!(21.14)),
                        },
                    ]),
                    available_to_lay: Some(vec![PriceSize {
                        price: Price::new(dec!(1.43)).unwrap(),
                        size: Size::new(dec!(31.11)),
                    }]),
                    traded_volume: Some(vec![]),
                }),
                orders: None,
                matches: None,
                matches_by_strategy: None,
            },
            Runner {
                selection_id: SelectionId(50310375),
                handicap: dec!(0),
                status: RunnerStatus::Active,
                adjustment_factor: None,
                last_price_traded: None,
                total_matched: Some(dec!(0)),
                removal_date: None,
                sp: None,
                ex: Some(ExchangePrices {
                    available_to_back: Some(vec![
                        PriceSize {
                            price: Price::new(dec!(3.75)).unwrap(),
                            size: Size::new(dec!(5.21)),
                        },
                        PriceSize {
                            price: Price::new(dec!(1.02)).unwrap(),
                            size: Size::new(dec!(6.65)),
                        },
                        PriceSize {
                            price: Price::new(dec!(1.01)).unwrap(),
                            size: Size::new(dec!(21.14)),
                        },
                    ]),
                    available_to_lay: Some(vec![PriceSize {
                        price: Price::new(dec!(4.8)).unwrap(),
                        size: Size::new(dec!(4.37)),
                    }]),
                    traded_volume: Some(vec![]),
                }),
                orders: None,
                matches: None,
                matches_by_strategy: None,
            },
        ]),
        key_line_description: None,
    };
    assert_eq!(result, &expected);
}
