#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use crate::models::_entities::{groups, members, transactions};
use chrono::Utc;
use loco_rs::prelude::*;
use sea_orm::prelude::Decimal;
use sea_orm::{ActiveValue::Set, ColumnTrait, EntityTrait, QueryFilter, TransactionTrait};

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

    let txn = ctx.db.begin().await?;

    for member in &member_list {
        let amount = member.total_contributed.unwrap_or(Decimal::from(0));

        let payout_txn = transactions::ActiveModel {
            member_id: Set(member.id),
            member_name: Set(member.name.clone()),
            group_id: Set(group_id),
            group_name: Set(group.name.clone()),
            amount: Set(amount),
            r#type: Set("payout".to_string()),
            status: Set("completed".to_string()),
            moolre_ref: Set(None),
            date: Set(Some(now)),
            ..Default::default()
        };
        payout_txn.insert(&txn).await?;
    }

    let total_payout: i64 = member_list
        .iter()
        .map(|m| m.total_contributed.unwrap_or(Decimal::from(0)))
        .map(|d| d.to_string().parse::<i64>().unwrap_or(0))
        .sum();

    let member_count = member_list.len() as i32;

    let mut group_active: groups::ActiveModel = group.into();
    group_active.total_contributed = Set(Some(Decimal::from(0)));
    group_active.member_count = Set(Some(member_count));
    group_active.next_payout_date = Set(now + chrono::Duration::days(30));
    group_active.update(&txn).await?;

    for member in &member_list {
        let mut member_active: members::ActiveModel = member.clone().into();
        member_active.total_contributed = Set(Some(Decimal::from(0)));
        member_active.update(&txn).await?;
    }

    txn.commit().await?;

    Ok(Json(serde_json::json!({
        "success": true,
        "transactionId": format!("PAYOUT_{}_{}", group_id, Utc::now().timestamp()),
        "totalAmount": total_payout,
        "memberCount": member_list.len()
    })))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/groups")
        .add("/{group_id}/payout", post(trigger))
}
