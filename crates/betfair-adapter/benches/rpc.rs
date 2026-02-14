use betfair_rpc_server_mock::Server;
use betfair_types::types::sports_aping::MarketId;
use betfair_types::types::sports_aping::list_market_book;
use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};
use serde_json::json;

fn make_runtime() -> tokio::runtime::Runtime {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
}

fn rpc_send_request_list_market_book(c: &mut Criterion) {
    let rt = make_runtime();

    // Setup: start mock server, authenticate client, mount mock response
    let (client, _server) = rt.block_on(async {
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
                                { "price": 1.19, "size": 57.52 },
                                { "price": 1.02, "size": 6.48 },
                                { "price": 1.01, "size": 21.14 }
                            ],
                            "availableToLay": [
                                { "price": 1.43, "size": 31.11 }
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
                                { "price": 3.75, "size": 5.21 }
                            ],
                            "availableToLay": [
                                { "price": 4.8, "size": 4.37 }
                            ],
                            "tradedVolume": []
                        }
                    }
                ]
            }
        ]);
        server
            .mock_authenticated_rpc_from_json::<list_market_book::Parameters>(response)
            .mount(&server.bf_api_mock_server)
            .await;

        let unauth = server.client().await;
        let (client, _keep_alive) = unauth.authenticate().await.unwrap();
        (client, server)
    });

    c.bench_function("rpc_send_request_list_market_book", |b| {
        b.iter(|| {
            rt.block_on(async {
                let result = client
                    .send_request(
                        list_market_book::Parameters::builder()
                            .market_ids(vec![MarketId::new("1.206502771")])
                            .build(),
                    )
                    .await
                    .unwrap();
                black_box(result);
            });
        });
    });
}

fn rpc_bot_login(c: &mut Criterion) {
    let rt = make_runtime();

    let server = rt.block_on(async { Server::new().await });

    c.bench_function("rpc_bot_login", |b| {
        b.iter_batched(
            || rt.block_on(async { server.client().await }),
            |unauth_client| {
                rt.block_on(async {
                    let result = unauth_client.authenticate().await.unwrap();
                    black_box(result);
                });
            },
            BatchSize::SmallInput,
        );
    });
}

criterion_group!(benches, rpc_send_request_list_market_book, rpc_bot_login);
criterion_main!(benches);
