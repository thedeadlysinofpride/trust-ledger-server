#![allow(clippy::missing_errors_doc)]

use std::collections::HashMap;

use crate::models::_entities::{groups, members, transactions};
use chrono::{Datelike, Duration, NaiveDate, NaiveDateTime, Utc};
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};
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
    total_contributions: f64,
    average_trust_score: i32,
    trust_breakdown: Vec<TrustBreakdown>,
    contribution_series: Vec<ContributionSeriesPoint>,
    group_health: Vec<GroupHealth>,
    next_payout: Option<NextPayout>,
}

#[derive(Default)]
struct MonthlyTotals {
    contributions: Decimal,
    payouts: Decimal,
}

pub async fn stats(State(ctx): State<AppContext>) -> Result<Json<impl Serialize>> {
    // Three bulk reads keep the endpoint predictable and avoid per-group queries.
    let all_groups = groups::Entity::find()
        .order_by_asc(groups::Column::Id)
        .all(&ctx.db)
        .await?;
    let all_members = members::Entity::find().all(&ctx.db).await?;
    let now = Utc::now().naive_utc();
    let current_month = now.date();
    let month_keys: Vec<(i32, u32)> = (0..6)
        .rev()
        .map(|months_ago| month_key(current_month, months_ago))
        .collect();
    let transaction_start = dashboard_transaction_start(now);
    let all_transactions = transactions::Entity::find()
        .filter(transactions::Column::Date.gte(transaction_start))
        .filter(transactions::Column::Status.eq("completed"))
        .filter(transactions::Column::Type.is_in(["contribution", "payout"]))
        .all(&ctx.db)
        .await?;

    let total_groups = all_groups
        .iter()
        .filter(|group| group.is_active == Some(true))
        .count() as u64;
    let total_members = all_members.len() as u64;
    let total_contributions_decimal: Decimal = all_groups
        .iter()
        .map(|group| group.total_contributed.unwrap_or(Decimal::ZERO))
        .sum();
    let total_contributions = decimal_to_response_number(total_contributions_decimal)?;

    let average_trust_score = average_score(all_members.iter().map(|member| member.trust_score));

    let mut members_by_group: HashMap<i32, Vec<&members::Model>> = HashMap::new();
    for member in &all_members {
        members_by_group
            .entry(member.group_id)
            .or_default()
            .push(member);
    }

    let mut monthly_totals: HashMap<(i32, u32), MonthlyTotals> = month_keys
        .iter()
        .copied()
        .map(|key| (key, MonthlyTotals::default()))
        .collect();
    let mut transactions_by_group: HashMap<i32, Vec<&transactions::Model>> = HashMap::new();

    for transaction in &all_transactions {
        transactions_by_group
            .entry(transaction.group_id)
            .or_default()
            .push(transaction);

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
            "contribution" => totals.contributions += transaction.amount,
            "payout" => totals.payouts += transaction.amount,
            _ => {}
        }
    }

    let contribution_series = month_keys
        .into_iter()
        .map(|key| -> Result<ContributionSeriesPoint> {
            let totals = monthly_totals.remove(&key).unwrap_or_default();
            Ok(ContributionSeriesPoint {
                label: NaiveDate::from_ymd_opt(key.0, key.1, 1)
                    .expect("generated year and month are valid")
                    .format("%b")
                    .to_string(),
                contributions: decimal_to_response_number(totals.contributions)?,
                payouts: decimal_to_response_number(totals.payouts)?,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let trust_breakdown = all_groups
        .iter()
        .map(|group| TrustBreakdown {
            group_name: group.name.clone(),
            score: group_score(members_by_group.get(&group.id)),
        })
        .collect();

    let group_health = all_groups
        .iter()
        .map(|group| -> Result<GroupHealth> {
            let active_member_count = members_by_group.get(&group.id).map_or(0, |members| {
                members
                    .iter()
                    .filter(|member| member.status.as_deref() == Some("active"))
                    .count()
            });
            let group_transactions = transactions_by_group
                .get(&group.id)
                .map(Vec::as_slice)
                .unwrap_or(&[]);
            let collection_rate = cycle_collection_rate(
                &group.frequency,
                group.contribution_amount.unwrap_or(Decimal::ZERO),
                active_member_count,
                group.id,
                group_transactions,
                now,
            )?;
            Ok(GroupHealth {
                group_id: group.id,
                group_name: group.name.clone(),
                collection_rate,
                trust_score: group_score(members_by_group.get(&group.id)),
                next_payout_date: group.next_payout_date,
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let next_payout = all_groups
        .iter()
        .filter(|group| group.is_active == Some(true) && group.next_payout_date >= now)
        .min_by_key(|group| group.next_payout_date)
        .map(|group| -> Result<NextPayout> {
            Ok(NextPayout {
                group_name: group.name.clone(),
                amount: decimal_to_response_number(
                    group.total_contributed.unwrap_or(Decimal::ZERO),
                )?,
                date: group.next_payout_date,
            })
        })
        .transpose()?;

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

fn decimal_to_response_number(value: Decimal) -> Result<f64> {
    let converted = value.to_string().parse().map_err(|error| {
        Error::Message(format!("cannot convert dashboard amount {value}: {error}"))
    })?;
    if !f64::is_finite(converted) {
        return Err(Error::Message(format!(
            "dashboard amount {value} is not a finite response number"
        )));
    }
    Ok(converted)
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

fn cycle_collection_rate(
    frequency: &str,
    contribution_amount: Decimal,
    active_member_count: usize,
    group_id: i32,
    transactions: &[&transactions::Model],
    now: NaiveDateTime,
) -> Result<f64> {
    let expected = contribution_amount * Decimal::from(active_member_count as u64);
    if expected <= Decimal::ZERO {
        return Ok(0.0);
    }

    let (cycle_start, cycle_end) = cycle_bounds(frequency, now);
    let collected: Decimal = transactions
        .iter()
        .filter(|transaction| {
            transaction.group_id == group_id
                && transaction.r#type == "contribution"
                && transaction.status == "completed"
                && transaction
                    .date
                    .is_some_and(|date| date >= cycle_start && date < cycle_end)
        })
        .map(|transaction| transaction.amount)
        .sum();

    decimal_to_response_number(
        (collected / expected * Decimal::from(100)).clamp(Decimal::ZERO, Decimal::from(100)),
    )
}

fn cycle_bounds(frequency: &str, now: NaiveDateTime) -> (NaiveDateTime, NaiveDateTime) {
    let current_date = now.date();
    let start_date = if frequency == "weekly" {
        current_date - Duration::days(i64::from(current_date.weekday().num_days_from_monday()))
    } else {
        NaiveDate::from_ymd_opt(current_date.year(), current_date.month(), 1)
            .expect("the first day of the current month is valid")
    };
    let end_date = if frequency == "weekly" {
        start_date + Duration::days(7)
    } else {
        let (year, month) = if start_date.month() == 12 {
            (start_date.year() + 1, 1)
        } else {
            (start_date.year(), start_date.month() + 1)
        };
        NaiveDate::from_ymd_opt(year, month, 1).expect("the first day of the next month is valid")
    };
    (
        start_date.and_hms_opt(0, 0, 0).expect("midnight is valid"),
        end_date.and_hms_opt(0, 0, 0).expect("midnight is valid"),
    )
}

fn month_key(current: NaiveDate, months_ago: u32) -> (i32, u32) {
    let month_index = current.year() * 12 + current.month0() as i32 - months_ago as i32;
    (
        month_index.div_euclid(12),
        month_index.rem_euclid(12) as u32 + 1,
    )
}

fn dashboard_transaction_start(now: NaiveDateTime) -> NaiveDateTime {
    let (year, month) = month_key(now.date(), 5);
    NaiveDate::from_ymd_opt(year, month, 1)
        .expect("the oldest dashboard month is valid")
        .and_hms_opt(0, 0, 0)
        .expect("midnight is valid")
}

pub fn routes() -> Routes {
    Routes::new().prefix("api/dashboards/").add("/", get(stats))
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;
    use std::str::FromStr;

    fn transaction(
        id: i32,
        group_id: i32,
        amount: Decimal,
        kind: &str,
        status: &str,
        date: chrono::NaiveDateTime,
    ) -> transactions::Model {
        transactions::Model {
            id,
            member_id: id,
            member_name: format!("Member {id}"),
            group_id,
            group_name: format!("Group {group_id}"),
            amount,
            r#type: kind.to_owned(),
            status: status.to_owned(),
            moolre_ref: None,
            date: Some(date),
        }
    }

    fn at(year: i32, month: u32, day: u32) -> chrono::NaiveDateTime {
        NaiveDate::from_ymd_opt(year, month, day)
            .unwrap()
            .and_hms_opt(12, 0, 0)
            .unwrap()
    }

    #[test]
    fn monthly_collection_rate_excludes_historical_pending_and_unrelated_values() {
        let now = at(2026, 6, 22);
        let transactions = [
            transaction(
                1,
                7,
                100.into(),
                "contribution",
                "completed",
                at(2026, 6, 5),
            ),
            transaction(
                2,
                7,
                500.into(),
                "contribution",
                "completed",
                at(2026, 5, 31),
            ),
            transaction(3, 7, 100.into(), "contribution", "pending", at(2026, 6, 6)),
            transaction(4, 7, 900.into(), "payout", "completed", at(2026, 6, 7)),
            transaction(
                5,
                8,
                900.into(),
                "contribution",
                "completed",
                at(2026, 6, 8),
            ),
        ];

        let transaction_refs = transactions.iter().collect::<Vec<_>>();
        assert_eq!(
            cycle_collection_rate("monthly", 50.into(), 4, 7, &transaction_refs, now).unwrap(),
            50.0,
        );
    }

    #[test]
    fn weekly_collection_rate_uses_the_current_monday_to_monday_window() {
        let now = at(2026, 6, 24);
        let transactions = [
            transaction(
                1,
                7,
                75.into(),
                "contribution",
                "completed",
                at(2026, 6, 22),
            ),
            transaction(
                2,
                7,
                75.into(),
                "contribution",
                "completed",
                at(2026, 6, 21),
            ),
        ];

        let transaction_refs = transactions.iter().collect::<Vec<_>>();
        assert_eq!(
            cycle_collection_rate("weekly", 75.into(), 2, 7, &transaction_refs, now).unwrap(),
            50.0,
        );
    }

    #[test]
    fn collection_rate_is_zero_safe_for_missing_members_or_amount() {
        let now = at(2026, 6, 22);
        let transactions = [transaction(
            1,
            7,
            100.into(),
            "contribution",
            "completed",
            now,
        )];

        let transaction_refs = transactions.iter().collect::<Vec<_>>();
        assert_eq!(
            cycle_collection_rate("monthly", 100.into(), 0, 7, &transaction_refs, now).unwrap(),
            0.0,
        );
        assert_eq!(
            cycle_collection_rate("monthly", Decimal::ZERO, 2, 7, &transaction_refs, now).unwrap(),
            0.0,
        );
    }

    #[test]
    fn collection_rate_preserves_fractional_decimal_amounts() {
        let now = at(2026, 6, 22);
        let transactions = [transaction(
            1,
            7,
            Decimal::from_str("0.10").unwrap(),
            "contribution",
            "completed",
            now,
        )];
        let transaction_refs = transactions.iter().collect::<Vec<_>>();

        let rate = cycle_collection_rate(
            "monthly",
            Decimal::from_str("0.10").unwrap(),
            3,
            7,
            &transaction_refs,
            now,
        )
        .unwrap();
        assert!((rate - 33.333_333_333_333_336).abs() < 1e-12);
    }

    #[test]
    fn response_money_conversion_keeps_fractional_high_value_amounts() {
        let amount = Decimal::from_str("123456789012345.67").unwrap();
        let converted = decimal_to_response_number(amount).unwrap();

        assert!((converted - 123_456_789_012_345.67).abs() < 0.02);
    }

    #[test]
    fn transaction_window_starts_at_the_first_of_the_oldest_series_month() {
        assert_eq!(
            dashboard_transaction_start(at(2026, 6, 22)),
            NaiveDate::from_ymd_opt(2026, 1, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
        assert_eq!(
            dashboard_transaction_start(at(2026, 2, 10)),
            NaiveDate::from_ymd_opt(2025, 9, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap()
        );
    }
}
