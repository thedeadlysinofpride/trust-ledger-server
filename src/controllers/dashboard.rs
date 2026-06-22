#![allow(clippy::missing_errors_doc)]

use std::collections::HashMap;

use crate::models::_entities::{groups, members, transactions};
use chrono::{Datelike, NaiveDate, Utc};
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, QueryOrder};
use serde::Serialize;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TrustBreakdown {
    group_name: String,
    score: i32,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ContributionSeriesPoint {
    label: String,
    contributions: f64,
    payouts: f64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupHealth {
    group_id: i32,
    group_name: String,
    collection_rate: f64,
    trust_score: i32,
    next_payout_date: chrono::NaiveDateTime,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct NextPayout {
    group_name: String,
    amount: f64,
    date: chrono::NaiveDateTime,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardResponse {
    total_groups: u64,
    total_members: u64,
    total_contributions: i64,
    average_trust_score: i32,
    trust_breakdown: Vec<TrustBreakdown>,
    contribution_series: Vec<ContributionSeriesPoint>,
    group_health: Vec<GroupHealth>,
    next_payout: Option<NextPayout>,
}

#[derive(Default)]
struct GroupTransactionTotals {
    collected: f64,
    expected: f64,
}

#[derive(Default)]
struct MonthlyTotals {
    contributions: f64,
    payouts: f64,
}

pub async fn stats(State(ctx): State<AppContext>) -> Result<Json<impl Serialize>> {
    // Three bulk reads keep the endpoint predictable and avoid per-group queries.
    let all_groups = groups::Entity::find()
        .order_by_asc(groups::Column::Id)
        .all(&ctx.db)
        .await?;
    let all_members = members::Entity::find().all(&ctx.db).await?;
    let all_transactions = transactions::Entity::find().all(&ctx.db).await?;

    let total_groups = all_groups
        .iter()
        .filter(|group| group.is_active == Some(true))
        .count() as u64;
    let total_members = all_members.len() as u64;
    let total_contributions = all_groups
        .iter()
        .map(|group| {
            group
                .total_contributed
                .unwrap_or(Decimal::ZERO)
                .to_string()
                .parse::<i64>()
                .unwrap_or(0)
        })
        .sum();

    let average_trust_score = average_score(all_members.iter().map(|member| member.trust_score));

    let mut members_by_group: HashMap<i32, Vec<&members::Model>> = HashMap::new();
    for member in &all_members {
        members_by_group
            .entry(member.group_id)
            .or_default()
            .push(member);
    }

    let current_month = Utc::now().date_naive();
    let month_keys: Vec<(i32, u32)> = (0..6)
        .rev()
        .map(|months_ago| month_key(current_month, months_ago))
        .collect();
    let mut monthly_totals: HashMap<(i32, u32), MonthlyTotals> = month_keys
        .iter()
        .copied()
        .map(|key| (key, MonthlyTotals::default()))
        .collect();
    let mut group_transaction_totals: HashMap<i32, GroupTransactionTotals> = HashMap::new();

    for transaction in &all_transactions {
        let amount = decimal_to_f64(Some(transaction.amount));
        if transaction.r#type == "contribution" {
            let totals = group_transaction_totals
                .entry(transaction.group_id)
                .or_default();
            totals.expected += amount;
            if transaction.status == "completed" {
                totals.collected += amount;
            }
        }

        if transaction.status != "completed" {
            continue;
        }
        let Some(date) = transaction.date else {
            continue;
        };
        let Some(totals) = monthly_totals.get_mut(&(date.year(), date.month())) else {
            continue;
        };
        match transaction.r#type.as_str() {
            "contribution" => totals.contributions += amount,
            "payout" => totals.payouts += amount,
            _ => {}
        }
    }

    let contribution_series = month_keys
        .into_iter()
        .map(|key| {
            let totals = monthly_totals.remove(&key).unwrap_or_default();
            ContributionSeriesPoint {
                label: NaiveDate::from_ymd_opt(key.0, key.1, 1)
                    .expect("generated year and month are valid")
                    .format("%b")
                    .to_string(),
                contributions: totals.contributions,
                payouts: totals.payouts,
            }
        })
        .collect();

    let trust_breakdown = all_groups
        .iter()
        .map(|group| TrustBreakdown {
            group_name: group.name.clone(),
            score: group_score(members_by_group.get(&group.id)),
        })
        .collect();

    let group_health = all_groups
        .iter()
        .map(|group| {
            let transaction_totals = group_transaction_totals.get(&group.id);
            let collection_rate = transaction_totals.map_or(0.0, |totals| {
                if totals.expected <= 0.0 {
                    0.0
                } else {
                    (totals.collected / totals.expected * 100.0).clamp(0.0, 100.0)
                }
            });
            GroupHealth {
                group_id: group.id,
                group_name: group.name.clone(),
                collection_rate,
                trust_score: group_score(members_by_group.get(&group.id)),
                next_payout_date: group.next_payout_date,
            }
        })
        .collect();

    let now = Utc::now().naive_utc();
    let next_payout = all_groups
        .iter()
        .filter(|group| group.is_active == Some(true) && group.next_payout_date >= now)
        .min_by_key(|group| group.next_payout_date)
        .map(|group| NextPayout {
            group_name: group.name.clone(),
            amount: decimal_to_f64(group.total_contributed),
            date: group.next_payout_date,
        });

    Ok(Json(DashboardResponse {
        total_groups,
        total_members,
        total_contributions,
        average_trust_score,
        trust_breakdown,
        contribution_series,
        group_health,
        next_payout,
    }))
}

fn decimal_to_f64(value: Option<Decimal>) -> f64 {
    value
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse()
        .unwrap_or(0.0)
}

fn average_score(scores: impl Iterator<Item = Option<i32>>) -> i32 {
    let (sum, count) = scores.fold((0_i64, 0_i64), |(sum, count), score| {
        (sum + i64::from(score.unwrap_or(0)), count + 1)
    });
    if count == 0 {
        0
    } else {
        (sum / count) as i32
    }
}

fn group_score(members: Option<&Vec<&members::Model>>) -> i32 {
    members.map_or(0, |members| {
        average_score(members.iter().map(|member| member.trust_score))
    })
}

fn month_key(current: NaiveDate, months_ago: u32) -> (i32, u32) {
    let month_index = current.year() * 12 + current.month0() as i32 - months_ago as i32;
    (
        month_index.div_euclid(12),
        month_index.rem_euclid(12) as u32 + 1,
    )
}

pub fn routes() -> Routes {
    Routes::new().prefix("api/dashboards/").add("/", get(stats))
}
