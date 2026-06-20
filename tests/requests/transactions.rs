use loco_rs::testing::prelude::*;
use serial_test::serial;
use trust_ledger_server::app::App;
use trust_ledger_server::data::demo_seed;

#[tokio::test]
#[serial]
async fn demo_seed_populates_transaction_history() {
    request::<App, _, _>(|request, ctx| async move {
        demo_seed::seed(&ctx.db)
            .await
            .expect("demo seed should succeed");

        let res = request.get("/api/transactions/").await;
        assert_eq!(res.status_code(), 200);

        let transactions = res.json::<Vec<serde_json::Value>>();
        assert!(transactions.len() >= 18);
        assert!(transactions.windows(2).all(|pair| {
            pair[0]["date"]
                .as_str()
                .expect("transaction date should be present")
                >= pair[1]["date"]
                    .as_str()
                    .expect("transaction date should be present")
        }));
        assert!(transactions
            .iter()
            .any(|transaction| transaction["type"] == "payout"));
        assert!(transactions
            .iter()
            .any(|transaction| transaction["status"] == "pending"));
    })
    .await;
}
