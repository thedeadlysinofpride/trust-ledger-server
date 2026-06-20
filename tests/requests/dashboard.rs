use loco_rs::testing::prelude::*;
use serde::Deserialize;
use serial_test::serial;
use trust_ledger_server::app::App;
use trust_ledger_server::data::demo_seed;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct DashboardResponse {
    total_groups: u64,
    total_members: u64,
    total_contributions: f64,
    contribution_series: Vec<serde_json::Value>,
    group_health: Vec<serde_json::Value>,
}

#[tokio::test]
#[serial]
async fn demo_seed_populates_dashboard_idempotently() {
    request::<App, _, _>(|request, ctx| async move {
        demo_seed::seed(&ctx.db)
            .await
            .expect("first demo seed should succeed");

        let first_res = request.get("/api/dashboards/").await;
        assert_eq!(first_res.status_code(), 200);
        let first_dashboard = first_res.json::<DashboardResponse>();

        assert_eq!(
            first_dashboard.total_groups, 4,
            "demo seed should create exactly four groups"
        );
        assert!(
            first_dashboard.total_members >= 12,
            "demo seed should create at least twelve members"
        );
        assert!(
            first_dashboard.total_contributions > 20_000.0,
            "demo seed contributions should exceed 20,000"
        );
        assert_eq!(
            first_dashboard.contribution_series.len(),
            6,
            "dashboard should expose six contribution periods"
        );
        assert_eq!(
            first_dashboard.group_health.len(),
            4,
            "dashboard should expose health for every demo group"
        );

        demo_seed::seed(&ctx.db)
            .await
            .expect("second demo seed should be idempotent");

        let second_res = request.get("/api/dashboards/").await;
        assert_eq!(second_res.status_code(), 200);
        let second_dashboard = second_res.json::<DashboardResponse>();

        assert_eq!(
            second_dashboard, first_dashboard,
            "running demo seed twice must not change dashboard totals or series"
        );
    })
    .await;
}
