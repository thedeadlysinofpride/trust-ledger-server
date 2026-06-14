#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use crate::models::_entities::{groups, members, transactions};
use chrono::Utc;
use loco_rs::prelude::*;
use reqwest::Client;
use sea_orm::prelude::Decimal;
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};
use serde_json::json;

#[debug_handler]
pub async fn trigger(
    Path(group_id): Path<i32>,
    State(ctx): State<AppContext>,
) -> Result<Json<serde_json::Value>> {
    let group = groups::Entity::find_by_id(group_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    let now = Utc::now().naive_utc();
    if group.next_payout_date > now {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": format!("Payout not available until {}", group.next_payout_date)
        })));
    }

    let member_list = members::Entity::find()
        .filter(members::Column::GroupId.eq(group_id))
        .filter(members::Column::Status.eq("active"))
        .all(&ctx.db)
        .await?;

    if member_list.is_empty() {
        return Ok(Json(serde_json::json!({
            "success": false,
            "message": "No active members in this group"
        })));
    }

    // Get Moolre credentials from config
    let moolre_user = std::env::var("MOOLRE_API_USER").unwrap_or_else(|_| "jhayonline".to_string());
    let moolre_key = std::env::var("MOOLRE_API_KEY").unwrap_or_else(|_| "".to_string());
    let account_number =
        std::env::var("MOOLRE_ACCOUNT_NUMBER").unwrap_or_else(|_| "10878506070909".to_string());

    let client = Client::new();
    let mut failed_members = Vec::new();

    // Process each member's payout via Moolre API
    for member in &member_list {
        let amount = member.total_contributed.unwrap_or(Decimal::from(0));
        let amount_f64: f64 = amount.to_string().parse().unwrap_or(0.0);

        if amount_f64 <= 0.0 {
            tracing::warn!("Member {} has zero balance, skipping payout", member.name);
            continue;
        }

        let external_ref = format!(
            "PAYOUT_{}_{}_{}",
            group_id,
            member.id,
            Utc::now().timestamp()
        );

        let payload = json!({
            "type": 1,
            "channel": "1",  // 1=MTN, 6=Telecel, 7=AT
            "currency": "GHS",
            "amount": amount_f64,
            "receiver": member.phone,
            "sublistid": "",
            "externalref": external_ref,
            "reference": format!("Susu payout for {}", member.name),
            "accountnumber": account_number
        });

        tracing::info!("Sending payout to {}: {:?}", member.name, payload);

        let response = client
            .post("https://api.moolre.com/open/transact/transfer")
            .header("X-API-USER", &moolre_user)
            .header("X-API-KEY", &moolre_key)
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await;

        match response {
            Ok(resp) => {
                let response_body: serde_json::Value = resp.json().await.unwrap_or(json!({}));
                tracing::info!("Moolre response for {}: {:?}", member.name, response_body);

                let txstatus = response_body
                    .get("data")
                    .and_then(|d| d.get("txstatus"))
                    .and_then(|s| s.as_str())
                    .unwrap_or("2");

                if txstatus == "1" {
                    tracing::info!("Payout successful for {}", member.name);
                } else {
                    tracing::error!("Payout failed for {}: {:?}", member.name, response_body);
                    failed_members.push(member.name.clone());
                }
            }
            Err(e) => {
                tracing::error!("Failed to call Moolre API for {}: {}", member.name, e);
                failed_members.push(member.name.clone());
            }
        }
    }

    // Start database transaction for records
    let db_txn = ctx.db.begin().await?;

    // Create transaction records for each member
    for member in &member_list {
        let amount = member.total_contributed.unwrap_or(Decimal::from(0));

        let payout_txn = transactions::ActiveModel {
            member_id: Set(member.id),
            member_name: Set(member.name.clone()),
            group_id: Set(group_id),
            group_name: Set(group.name.clone()),
            amount: Set(amount),
            r#type: Set("payout".to_string()),
            status: Set(if failed_members.contains(&member.name) {
                "failed".to_string()
            } else {
                "completed".to_string()
            }),
            moolre_ref: Set(None),
            date: Set(Some(now)),
            ..Default::default()
        };
        payout_txn.insert(&db_txn).await?;
    }

    let total_payout: i64 = member_list
        .iter()
        .map(|m| m.total_contributed.unwrap_or(Decimal::from(0)))
        .map(|d| d.to_string().parse::<i64>().unwrap_or(0))
        .sum();

    let member_count = member_list.len() as i32;

    // Update group
    let mut group_active: groups::ActiveModel = group.into();
    group_active.total_contributed = Set(Some(Decimal::from(0)));
    group_active.member_count = Set(Some(member_count));
    group_active.next_payout_date = Set(now + chrono::Duration::days(30));
    group_active.update(&db_txn).await?;

    // Reset each member's contribution total
    for member in &member_list {
        let mut member_active: members::ActiveModel = member.clone().into();
        member_active.total_contributed = Set(Some(Decimal::from(0)));
        member_active.update(&db_txn).await?;
    }

    db_txn.commit().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "transactionId": format!("PAYOUT_{}_{}", group_id, Utc::now().timestamp()),
        "totalAmount": total_payout,
        "memberCount": member_list.len(),
        "failedMembers": failed_members
    })))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/groups")
        .add("/{group_id}/payout", post(trigger))
}
