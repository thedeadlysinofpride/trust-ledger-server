#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use crate::models::_entities::{transactions, members};
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, QueryOrder, PaginatorTrait, QuerySelect};
use serde::{Deserialize, Serialize};

#[derive(Serialize)]
pub struct MemberTransactionResponse {
    pub id: i32,
    pub amount: i64,
    pub transaction_type: String,
    pub status: String,
    pub group_name: String,
    pub date: String,
    pub reference: Option<String>,
}

#[derive(Serialize)]
pub struct MemberTransactionsSummary {
    pub total_contributions: i64,
    pub total_payouts: i64,
    pub net_savings: i64,
    pub transaction_count: i32,
    pub transactions: Vec<MemberTransactionResponse>,
}

#[derive(Serialize)]
pub struct PaginatedTransactionsResponse {
    pub data: Vec<MemberTransactionResponse>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}

// Query parameters for pagination and filtering
#[derive(Debug, Deserialize)]
pub struct TransactionQueryParams {
    pub page: Option<i32>,
    pub per_page: Option<i32>,
    pub transaction_type: Option<String>,
    pub status: Option<String>,
}

#[debug_handler]
pub async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Query(params): Query<TransactionQueryParams>,
) -> Result<Json<PaginatedTransactionsResponse>> {
    // Get member ID from JWT claims
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    // Get member details to verify existence
    let _member = members::Entity::find_by_id(member_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    // Build query
    let mut query = transactions::Entity::find()
        .filter(transactions::Column::MemberId.eq(member_id));
    
    // Apply type filter if provided
    if let Some(transaction_type) = &params.transaction_type {
        if transaction_type == "contribution" || transaction_type == "payout" {
            query = query.filter(transactions::Column::Type.eq(transaction_type));
        }
    }
    
    // Apply status filter if provided
    if let Some(status) = &params.status {
        if status == "completed" || status == "pending" || status == "failed" {
            query = query.filter(transactions::Column::Status.eq(status));
        }
    }
    
    // Get total count before pagination
    let total = query.clone().count(&ctx.db).await?;
    
    // Apply pagination
    let page = params.page.unwrap_or(1);
    let per_page = params.per_page.unwrap_or(10);
    let offset = (page - 1) * per_page;
    
    let transaction_list = query
        .order_by_desc(transactions::Column::Date)
        .limit(per_page as u64)
        .offset(offset as u64)
        .all(&ctx.db)
        .await?;
    
    let total_pages = (total as f64 / per_page as f64).ceil() as i32;
    
    let mut responses = Vec::new();
    for txn in transaction_list {
        responses.push(MemberTransactionResponse {
            id: txn.id,
            amount: txn.amount.to_string().parse().unwrap_or(0),
            transaction_type: txn.r#type.clone(),
            status: txn.status.clone(),
            group_name: txn.group_name.clone(),
            date: txn.date.map(|d| d.to_string()).unwrap_or_default(),
            reference: txn.moolre_ref.clone(),
        });
    }
    
    Ok(Json(PaginatedTransactionsResponse {
        data: responses,
        total: total as i64,
        page,
        per_page,
        total_pages,
    }))
}

#[debug_handler]
pub async fn summary(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Json<MemberTransactionsSummary>> {
    // Get member ID from JWT claims
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    // Get all member transactions
    let all_transactions = transactions::Entity::find()
        .filter(transactions::Column::MemberId.eq(member_id))
        .all(&ctx.db)
        .await?;
    
    let mut total_contributions: i64 = 0;
    let mut total_payouts: i64 = 0;
    let mut transaction_responses = Vec::new();
    
    for txn in all_transactions {
        let amount: i64 = txn.amount.to_string().parse().unwrap_or(0);
        
        if txn.r#type == "contribution" && txn.status == "completed" {
            total_contributions += amount;
        } else if txn.r#type == "payout" && txn.status == "completed" {
            total_payouts += amount;
        }
        
        transaction_responses.push(MemberTransactionResponse {
            id: txn.id,
            amount,
            transaction_type: txn.r#type.clone(),
            status: txn.status.clone(),
            group_name: txn.group_name.clone(),
            date: txn.date.map(|d| d.to_string()).unwrap_or_default(),
            reference: txn.moolre_ref.clone(),
        });
    }
    
    let net_savings = total_contributions - total_payouts;
    
    Ok(Json(MemberTransactionsSummary {
        total_contributions,
        total_payouts,
        net_savings,
        transaction_count: transaction_responses.len() as i32,
        transactions: transaction_responses,
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/member/transactions")
        .add("/", get(list))
        .add("/summary", get(summary))
}
