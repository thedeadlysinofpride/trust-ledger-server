use loco_rs::testing::prelude::*;
use serde::Deserialize;
use serial_test::serial;
use trust_ledger_server::app::App;
use trust_ledger_server::data::demo_seed;

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct ContributionSeriesPoint {
    label: String,
    contributions: f64,
    payouts: f64,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct GroupHealth {
    group_id: i32,
    group_name: String,
    collection_rate: f64,
    trust_score: i32,
    next_payout_date: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct NextPayout {
    group_name: String,
    amount: f64,
    date: String,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct TrustBreakdown {
    group_name: String,
    score: i32,
}

#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct DashboardResponse {
    total_groups: u64,
    total_members: u64,
    total_contributions: f64,
    average_trust_score: i32,
    trust_breakdown: Vec<TrustBreakdown>,
    contribution_series: Vec<ContributionSeriesPoint>,
    group_health: Vec<GroupHealth>,
    next_payout: Option<NextPayout>,
}

#[tokio::test]
#[serial]
async fn demo_seed_populates_dashboard_idempotently() {
    request::<App, _, _>(|request, ctx| async move {
        demo_seed::seed(&ctx.db)
            .await
            .expect("first demo seed should succeed");

        let first_res = request.get("/api/dashboards").await;
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
        assert!(
            first_dashboard.group_health.iter().all(|group| {
                (0.0..=100.0).contains(&group.collection_rate)
                    && (0..=100).contains(&group.trust_score)
            }),
            "group health rates and scores should be bounded percentages"
        );
        assert!(
            first_dashboard.next_payout.is_some(),
            "demo seed should expose the nearest future payout"
        );

        demo_seed::seed(&ctx.db)
            .await
            .expect("second demo seed should be idempotent");

        let second_res = request.get("/api/dashboards").await;
        assert_eq!(second_res.status_code(), 200);
        let second_dashboard = second_res.json::<DashboardResponse>();

        assert_eq!(
            second_dashboard, first_dashboard,
            "running demo seed twice must not change dashboard totals or series"
        );
    })
    .await;
}
