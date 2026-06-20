use loco_rs::testing::prelude::*;
use serial_test::serial;
use trust_ledger_server::app::App;
use trust_ledger_server::data::demo_seed;

#[tokio::test]
#[serial]
async fn demo_seed_populates_dashboard_idempotently() {
    request::<App, _, _>(|request, ctx| async move {
        demo_seed::seed(&ctx.db)
            .await
            .expect("first demo seed should succeed");
        demo_seed::seed(&ctx.db)
            .await
            .expect("second demo seed should be idempotent");

        let res = request.get("/api/dashboards/").await;
        assert_eq!(res.status_code(), 200);

        let dashboard = res.json::<serde_json::Value>();
        assert_eq!(dashboard["totalGroups"], 4);
        assert!(dashboard["totalMembers"].as_u64().unwrap() >= 12);
        assert!(dashboard["totalContributions"].as_f64().unwrap() > 20_000.0);
        assert_eq!(dashboard["contributionSeries"].as_array().unwrap().len(), 6);
        assert_eq!(dashboard["groupHealth"].as_array().unwrap().len(), 4);
    })
    .await;
}
