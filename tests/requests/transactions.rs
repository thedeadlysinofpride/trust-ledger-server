use loco_rs::testing::prelude::*;
use serial_test::serial;
use trust_ledger_server::app::App;
use trust_ledger_server::data::demo_seed;
use trust_ledger_server::models::_entities::transactions::Model as Transaction;

#[tokio::test]
#[serial]
async fn demo_seed_populates_transaction_history() {
    request::<App, _, _>(|request, ctx| async move {
        demo_seed::seed(&ctx.db)
            .await
            .expect("demo seed should succeed");

        let first_res = request.get("/api/transactions").await;
        assert_eq!(first_res.status_code(), 200);
        let first_transactions = first_res.json::<Vec<Transaction>>();

        assert!(
            first_transactions.len() >= 18,
            "demo seed should expose at least eighteen transactions"
        );
        assert!(
            first_transactions.windows(2).all(|pair| {
                pair[0].date.expect("transaction date should be present")
                    >= pair[1].date.expect("transaction date should be present")
            }),
            "transactions should be ordered by descending date"
        );
        assert!(
            first_transactions
                .iter()
                .any(|transaction| transaction.r#type == "payout"),
            "demo seed should include a payout transaction"
        );
        assert!(
            first_transactions
                .iter()
                .any(|transaction| transaction.status == "pending"),
            "demo seed should include a pending transaction"
        );

        demo_seed::seed(&ctx.db)
            .await
            .expect("second demo seed should be idempotent");

        let second_res = request.get("/api/transactions").await;
        assert_eq!(second_res.status_code(), 200);
        let second_transactions = second_res.json::<Vec<Transaction>>();

        assert_eq!(
            second_transactions, first_transactions,
            "running demo seed twice must not duplicate or alter transactions"
        );
    })
    .await;
}
