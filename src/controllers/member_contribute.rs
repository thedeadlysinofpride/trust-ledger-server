#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use crate::models::_entities::{members, groups, contributions};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue::Set, EntityTrait};
use serde::{Deserialize, Serialize};
use reqwest::Client;
use chrono::Utc;
use num_traits::cast::FromPrimitive;

#[derive(Debug, Deserialize)]
pub struct ContributeParams {
    pub group_id: i32,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct ContributeResponse {
    pub success: bool,
    pub contribution_id: i32,
    pub amount: f64,
    pub message: String,
    pub moolre_status: Option<String>,
}

#[debug_handler]
pub async fn contribute(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<ContributeParams>,
) -> Result<Json<ContributeResponse>> {
    // Get member ID from JWT claims
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    // Get member details
    let member = members::Entity::find_by_id(member_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    // Verify member belongs to this group
    if member.group_id != Some(params.group_id) {
        return Ok(Json(ContributeResponse {
            success: false,
            contribution_id: 0,
            amount: params.amount,
            message: "You are not a member of this group".to_string(),
            moolre_status: None,
        }));
    }
    
    // Get group details (clone for later use)
    let group = groups::Entity::find_by_id(params.group_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    let group_name = group.name.clone();
    
    // Create contribution record (pending status)
    let external_ref = format!("CONTRIB_{}_{}_{}", member_id, params.group_id, Utc::now().timestamp());
    
    // Convert f64 to Decimal
    let amount_decimal = Decimal::from_f64(params.amount).unwrap_or(Decimal::ZERO);
    
    let new_contribution = contributions::ActiveModel {
        member_id: Set(member_id),
        group_id: Set(params.group_id),
        amount: Set(amount_decimal),
        status: Set("pending".to_string()),
        moolre_ref: Set(Some(external_ref.clone())),
        date: Set(Some(Utc::now().naive_utc())),
        ..Default::default()
    };
    
    let contribution = new_contribution.insert(&ctx.db).await?;
    let contribution_id = contribution.id;
    
    // Call Moolre Collections API
    let moolre_user = std::env::var("MOOLRE_API_USER").unwrap_or_default();
    let moolre_pubkey = std::env::var("MOOLRE_API_PUBKEY").unwrap_or_default();
    let account_number = std::env::var("MOOLRE_ACCOUNT_NUMBER").unwrap_or_default();

    let client = Client::new();
    let payload = serde_json::json!({
        "type": 1,
        "channel": "1",
        "currency": "GHS",
        "amount": params.amount,
        "payer": member.phone,
        "externalref": external_ref,
        "reference": format!("Contribution to {}", group_name),
        "accountnumber": account_number
    });

    let moolre_response = client
        .post("https://api.moolre.com/open/transact/payment")
        .header("X-API-USER", &moolre_user)
        .header("X-API-PUBKEY", &moolre_pubkey)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await;
    
    let moolre_status = match moolre_response {
        Ok(resp) => {
            let resp_body: serde_json::Value = resp.json().await.unwrap_or_default();
            let status = resp_body.get("status").and_then(|s| s.as_str()).unwrap_or("unknown");
            
            if status == "1" {
                // Update contribution status to completed
                let mut contrib_active: contributions::ActiveModel = contribution.into();
                contrib_active.status = Set("completed".to_string());
                contrib_active.update(&ctx.db).await?;
                
                // Update group's total contributed
                let mut group_active: groups::ActiveModel = group.clone().into();
                // Use take() to get the value, then unwrap_or to get Decimal
                let current_total = group_active.total_contributed.take().unwrap_or(Some(Decimal::ZERO)).unwrap_or(Decimal::ZERO);
                let new_total = current_total + amount_decimal;
                group_active.total_contributed = Set(Some(new_total));
                group_active.update(&ctx.db).await?;
                
                Some("success".to_string())
            } else {
                Some(format!("failed: {:?}", resp_body))
            }
        }
        Err(e) => Some(format!("error: {}", e)),
    };
    
    Ok(Json(ContributeResponse {
        success: moolre_status == Some("success".to_string()),
        contribution_id,
        amount: params.amount,
        message: if moolre_status == Some("success".to_string()) {
            format!("Successfully contributed GHS {} to {}", params.amount, group_name)
        } else {
            format!("Contribution initiated but pending confirmation")
        },
        moolre_status,
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/member/contribute")
        .add("/", post(contribute))
}

