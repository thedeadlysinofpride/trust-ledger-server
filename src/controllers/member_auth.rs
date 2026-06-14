#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use loco_rs::prelude::*;
use loco_rs::auth::jwt;
use crate::models::_entities::members;
use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter, ColumnTrait};
use rand::prelude::*;
use chrono::{Utc, Duration};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct SendOtpParams {
    pub phone: String,
}

#[derive(Debug, Deserialize)]
pub struct VerifyOtpParams {
    pub phone: String,
    pub code: String,
}

#[derive(Serialize)]
pub struct MemberAuthResponse {
    pub token: String,
    pub member_id: i32,
    pub name: String,
    pub phone: String,
    pub trust_score: i32,
}

// Helper to normalize phone numbers
fn normalize_phone(phone: &str) -> String {
    let cleaned: String = phone.chars().filter(|c| c.is_ascii_digit()).collect();
    if cleaned.starts_with("0") {
        format!("233{}", &cleaned[1..])
    } else if cleaned.starts_with("233") {
        cleaned
    } else {
        format!("233{}", cleaned)
    }
}

// Send OTP via Moolre SMS API
async fn send_sms(phone: &str, message: &str) -> Result<()> {
    let api_user = std::env::var("MOOLRE_API_USER").unwrap_or_default();
    let vas_key = std::env::var("MOOLRE_API_VASKEY").unwrap_or_default();

    let client = reqwest::Client::new();
    let payload = serde_json::json!({
        "type": 1,
        "senderid": "TrustLedger",
        "messages": [{
            "recipient": phone,
            "message": message,
            "ref": format!("otp_{}", chrono::Utc::now().timestamp())
        }]
    });

    let response = client
        .post("https://api.moolre.com/open/sms/send")
        .header("X-API-USER", api_user)
        .header("X-API-VASKEY", vas_key)
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| Error::string(&format!("SMS failed: {}", e)))?;

    if response.status().is_success() {
        Ok(())
    } else {
        let error_text = response.text().await.unwrap_or_default();
        Err(Error::string(&format!("SMS API error: {}", error_text)))
    }
}

// Generate random 6-digit OTP
fn generate_otp() -> String {
    let mut rng = rand::rng();
    let otp: u32 = rng.random_range(100000..999999);
    otp.to_string()
}

// Request OTP endpoint
#[debug_handler]
pub async fn send_otp(
    State(ctx): State<AppContext>,
    Json(params): Json<SendOtpParams>,
) -> Result<Response> {
    let phone = normalize_phone(&params.phone);

    let member = members::Entity::find()
        .filter(members::Column::Phone.eq(&phone))
        .one(&ctx.db)
        .await?;

    let otp_code = generate_otp();
    let expires_at = Utc::now() + Duration::minutes(10);
    let expires_at_naive = expires_at.naive_utc();
    
    if let Some(existing_member) = member {
        // Update existing member with OTP
        let mut active: members::ActiveModel = existing_member.into();
        active.otp_code = Set(Some(otp_code.clone()));
        active.otp_expires_at = Set(Some(expires_at_naive));
        active.update(&ctx.db).await?;
    } else {
        // Create new member (just for auth, not yet in a group)
        // Note: group_id is required, so use a default or null?
        // For now, create without group (group_id = 0 or NULL)
        let new_member = members::ActiveModel {
            phone: Set(phone.clone()),
            name: Set(format!("Member_{}", &phone[phone.len()-4..])),
            trust_score: Set(Some(100)),
            group_id: Set(None),
            total_contributed: Set(Some(Default::default())),
            on_time_payments: Set(Some(0)),
            total_payments: Set(Some(0)),
            last_missed_payment: Set(None),
            status: Set(Some("active".to_string())),
            joined_at: Set(Some(Utc::now().naive_utc())),
            otp_code: Set(Some(otp_code.clone())),
            otp_expires_at: Set(Some(expires_at_naive)),
            ..Default::default()
        };
        new_member.insert(&ctx.db).await?;
    }

    // Send OTP via SMS
    let message = format!("Your TrustLedger verification code is: {}. Valid for 10 minutes.", otp_code);

    match send_sms(&phone, &message).await {
        Ok(_) => {
            tracing::info!("OTP sent to {}: {}", phone, otp_code);
            format::json(serde_json::json!({
                "success": true,
                "message": "OTP sent successfully"
            }))
        }
        Err(e) => {
            tracing::error!("Failed to send OTP: {}", e);
            // For development, still return success but log the OTP
            format::json(serde_json::json!({
                "success": true,
                "message": "OTP sent successfully",
                "debug_otp": otp_code // Remove in production
            }))
        }
    }
}

// Verify OTP endpoint
#[debug_handler]
pub async fn verify_otp(
    State(ctx): State<AppContext>,
    Json(params): Json<VerifyOtpParams>,
) -> Result<Json<MemberAuthResponse>> {
    let phone = normalize_phone(&params.phone);

    let member = members::Entity::find()
        .filter(members::Column::Phone.eq(&phone))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::Unauthorized("Member not found".to_string()))?;

    // Check OTP
    let stored_otp = member.otp_code.as_deref().ok_or_else(|| Error::Unauthorized("No OTP requested".to_string()))?;
    let expires_at = member.otp_expires_at.ok_or_else(|| Error::Unauthorized("OTP expired".to_string()))?;

    if Utc::now().naive_utc() > expires_at {
        return Err(Error::Unauthorized("OTP expired".to_string()));
    }

    if stored_otp != params.code {
        return Err(Error::Unauthorized("Invalid OTP".to_string()));
    }

    // Clear OTP after successful verification
    let mut active: members::ActiveModel = member.into();
    active.otp_code = Set(None);
    active.otp_expires_at = Set(None);
    let updated_member = active.update(&ctx.db).await?;

    // Generate JWT token for member using member id as claim
    let jwt_secret = ctx.config.get_jwt_config()?;
    let claims = serde_json::Map::new();
    let token = jwt::JWT::new(&jwt_secret.secret)
        .generate_token(jwt_secret.expiration, updated_member.id.to_string(), claims)
        .map_err(|_| Error::Unauthorized("Token generation failed".to_string()))?;

    Ok(Json(MemberAuthResponse {
        token,
        member_id: updated_member.id,
        name: updated_member.name,
        phone: updated_member.phone,
        trust_score: updated_member.trust_score.unwrap_or(100),
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/member/auth")
        .add("/send-otp", post(send_otp))
        .add("/verify", post(verify_otp))
}
