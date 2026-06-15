#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use crate::models::_entities::{members, trust_score_history};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde::Serialize;
use chrono::Utc;

#[derive(Serialize)]
pub struct TrustScoreHistoryResponse {
    pub id: i32,
    pub score: i32,
    pub change: i32,
    pub reason: String,
    pub date: String,
}

#[derive(Serialize)]
pub struct TrustScoreSummary {
    pub current_score: i32,
    pub highest_score: i32,
    pub lowest_score: i32,
    pub average_score: f64,
    pub total_contributions: i64,
    pub on_time_percentage: f64,
    pub history: Vec<TrustScoreHistoryResponse>,
}

#[debug_handler]
pub async fn history(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Json<TrustScoreSummary>> {
    // Get member ID from JWT claims
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    // Get member details
    let member = members::Entity::find_by_id(member_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    let current_score = member.trust_score.unwrap_or(100);
    
    // Get trust score history
    let history_records = trust_score_history::Entity::find()
        .filter(trust_score_history::Column::MemberId.eq(member_id))
        .order_by_desc(trust_score_history::Column::Date)
        .all(&ctx.db)
        .await?;
    
    let mut history_responses = Vec::new();
    let mut highest = current_score;
    let mut lowest = current_score;
    let mut sum = 0i64;
    
    for record in &history_records {
        let score = record.score;
        if score > highest { highest = score; }
        if score < lowest { lowest = score; }
        sum += score as i64;
        
        history_responses.push(TrustScoreHistoryResponse {
            id: record.id,
            score: record.score,
            change: record.change.unwrap_or(0),
            reason: record.reason.clone(),
            date: record.date.to_string(),
        });
    }
    
    let average_score = if history_records.is_empty() {
        current_score as f64
    } else {
        sum as f64 / history_records.len() as f64
    };
    
    // Calculate on-time percentage from member stats
    let total_payments = member.total_payments.unwrap_or(0);
    let on_time_payments = member.on_time_payments.unwrap_or(0);
    let on_time_percentage = if total_payments > 0 {
        (on_time_payments as f64 / total_payments as f64) * 100.0
    } else {
        100.0
    };
    
    let total_contributions = member.total_contributed
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse::<i64>()
        .unwrap_or(0);
    
    Ok(Json(TrustScoreSummary {
        current_score,
        highest_score: highest,
        lowest_score: lowest,
        average_score,
        total_contributions: total_contributions / 100, // Convert from pesewas to GHS
        on_time_percentage,
        history: history_responses,
    }))
}

// Helper function to record trust score change (to be called from other controllers)
pub async fn record_trust_score_change(
    db: &sea_orm::DatabaseConnection,
    member_id: i32,
    new_score: i32,
    change: i32,
    reason: &str,
) -> Result<()> {
    let new_record = trust_score_history::ActiveModel {
        member_id: Set(member_id),
        score: Set(new_score),
        change: Set(Some(change)),
        reason: Set(reason.to_string()),
        date: Set(Utc::now().naive_utc()),
        ..Default::default()
    };
    new_record.insert(db).await?;
    Ok(())
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/member/trust-score")
        .add("/history", get(history))
}
