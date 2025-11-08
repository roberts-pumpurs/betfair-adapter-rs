use betfair_types::types::sports_aping::MarketCatalogue;
use serde_json::json;

#[test]
fn parse_fixture_error() {
    use betfair_types::types::sports_aping::list_market_catalogue;
    use json_rpc_types::Response;
    let data = std::fs::read_to_string("./tests/resources/list_market_catalogue.json").unwrap();
    serde_json::from_str::<Response<list_market_catalogue::ReturnType, serde_json::Value>>(&data)
        .unwrap();
}
#[test]
fn parse_fixture_success() {
    use betfair_types::types::sports_aping::list_market_catalogue;
    use json_rpc_types::Response;
    let data =
        std::fs::read_to_string("./tests/resources/list_market_catalogue_no_ero.json").unwrap();
    serde_json::from_str::<Response<list_market_catalogue::ReturnType, serde_json::Value>>(&data)
        .unwrap();
}

#[test]
fn test_deserialize() {
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
                    "sortPriority": 2,
                    "metadata": {
                        "foo": "bar",
                        "baz": null
                    }
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
    let result = serde_json::from_value::<Vec<MarketCatalogue>>(response).unwrap();
    assert_eq!(result.len(), 1);

    // Check that we correctly deserialize metadata containing null values.
    let runners = result[0].runners.as_ref().unwrap();
    assert_eq!(runners.len(), 2);
    assert!(runners[0].metadata.is_none());
    assert!(runners[1].metadata.is_some());
    assert_eq!(runners[1].metadata.as_ref().unwrap().len(), 1);
}
