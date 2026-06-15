#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use crate::models::_entities::{savings_goals, savings_deposits};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

#[derive(Debug, Deserialize)]
pub struct CreateSavingsGoalParams {
    pub name: String,
    pub target_amount: f64,
    pub duration_days: i32,
    pub frequency: String,
    pub mode: String,
}

#[derive(Debug, Deserialize)]
pub struct DepositParams {
    pub goal_id: i32,
    pub amount: f64,
}

#[derive(Serialize)]
pub struct SavingsGoalResponse {
    pub id: i32,
    pub name: String,
    pub target_amount: i64,
    pub current_amount: i64,
    pub start_date: String,
    pub end_date: String,
    pub frequency: String,
    pub mode: String,
    pub status: String,
    pub progress_percent: f64,
    pub days_remaining: i64,
    pub suggested_deposit: i64,
}

#[derive(Serialize)]
pub struct CreateGoalResponse {
    pub success: bool,
    pub goal_id: i32,
    pub message: String,
    pub suggested_deposit: i64,
}

#[derive(Serialize)]
pub struct DepositResponse {
    pub success: bool,
    pub deposit_id: i32,
    pub goal_id: i32,
    pub new_current_amount: i64,
    pub message: String,
}

#[derive(Serialize)]
pub struct WithdrawResponse {
    pub success: bool,
    pub amount: i64,
    pub message: String,
}

fn calculate_suggested_deposit(target: i64, duration_days: i32, frequency: &str) -> i64 {
    let intervals = match frequency {
        "daily" => duration_days as f64,
        "weekly" => duration_days as f64 / 7.0,
        "monthly" => duration_days as f64 / 30.0,
        _ => duration_days as f64,
    };
    (target as f64 / intervals).ceil() as i64
}

fn days_remaining(end_date: chrono::NaiveDateTime) -> i64 {
    let now = Utc::now().naive_utc();
    let remaining = end_date - now;
    remaining.num_days()
}

#[debug_handler]
pub async fn create(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<CreateSavingsGoalParams>,
) -> Result<Json<CreateGoalResponse>> {
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    if !["daily", "weekly", "monthly"].contains(&params.frequency.as_str()) {
        return Ok(Json(CreateGoalResponse {
            success: false,
            goal_id: 0,
            message: "Frequency must be daily, weekly, or monthly".to_string(),
            suggested_deposit: 0,
        }));
    }
    
    if !["strict", "loose"].contains(&params.mode.as_str()) {
        return Ok(Json(CreateGoalResponse {
            success: false,
            goal_id: 0,
            message: "Mode must be strict or loose".to_string(),
            suggested_deposit: 0,
        }));
    }
    
    let target_amount_i64 = params.target_amount as i64;
    let start_date = Utc::now().naive_utc();
    let end_date = start_date + Duration::days(params.duration_days as i64);
    let suggested_deposit = calculate_suggested_deposit(target_amount_i64, params.duration_days, &params.frequency);
    
    let new_goal = savings_goals::ActiveModel {
        member_id: Set(member_id),
        name: Set(params.name),
        target_amount: Set(Decimal::from_i128_with_scale(target_amount_i64 as i128, 2)),
        current_amount: Set(Some(Decimal::ZERO)),
        start_date: Set(start_date),
        end_date: Set(end_date),
        frequency: Set(params.frequency),
        mode: Set(params.mode),
        status: Set(Some("active".to_string())),
        ..Default::default()
    };
    
    let goal = new_goal.insert(&ctx.db).await?;
    
    Ok(Json(CreateGoalResponse {
        success: true,
        goal_id: goal.id,
        message: format!("Savings goal '{}' created successfully", goal.name),
        suggested_deposit,
    }))
}

#[debug_handler]
pub async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Json<Vec<SavingsGoalResponse>>> {
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    let goals = savings_goals::Entity::find()
        .filter(savings_goals::Column::MemberId.eq(member_id))
        .order_by_desc(savings_goals::Column::CreatedAt)
        .all(&ctx.db)
        .await?;
    
    let mut responses = Vec::new();
    for goal in goals {
        let target_amount: i64 = goal.target_amount.to_string().parse().unwrap_or(0);
        let current_amount: i64 = goal.current_amount
            .unwrap_or(Decimal::ZERO)
            .to_string()
            .parse()
            .unwrap_or(0);
        
        let progress_percent = if target_amount > 0 {
            (current_amount as f64 / target_amount as f64) * 100.0
        } else {
            0.0
        };
        
        let days_rem = days_remaining(goal.end_date);
        let suggested = calculate_suggested_deposit(target_amount - current_amount, days_rem as i32, &goal.frequency);
        
        responses.push(SavingsGoalResponse {
            id: goal.id,
            name: goal.name,
            target_amount,
            current_amount,
            start_date: goal.start_date.to_string(),
            end_date: goal.end_date.to_string(),
            frequency: goal.frequency,
            mode: goal.mode,
            status: goal.status.unwrap_or_else(|| "active".to_string()),
            progress_percent,
            days_remaining: days_rem,
            suggested_deposit: suggested,
        });
    }
    
    Ok(Json(responses))
}

#[debug_handler]
pub async fn deposit(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<DepositParams>,
) -> Result<Json<DepositResponse>> {
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    let goal = savings_goals::Entity::find_by_id(params.goal_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    if goal.member_id != member_id {
        return Ok(Json(DepositResponse {
            success: false,
            deposit_id: 0,
            goal_id: params.goal_id,
            new_current_amount: 0,
            message: "You don't own this savings goal".to_string(),
        }));
    }
    
    let amount_decimal = Decimal::from_i128_with_scale(params.amount as i128, 2);
    let current = goal.current_amount.unwrap_or(Decimal::ZERO);
    let new_current = current + amount_decimal;
    
    let new_deposit = savings_deposits::ActiveModel {
        goal_id: Set(params.goal_id),
        amount: Set(amount_decimal),
        date: Set(Some(Utc::now().naive_utc())),
        moolre_ref: Set(None),
        ..Default::default()
    };
    let deposit = new_deposit.insert(&ctx.db).await?;
    
    let mut goal_active: savings_goals::ActiveModel = goal.into();
    // Extract the target value from ActiveValue
    let target_value = match &goal_active.target_amount {
        ActiveValue::Set(val) => val.clone(),
        ActiveValue::Unchanged(val) => val.clone(),
        _ => Decimal::ZERO,
    };
    goal_active.current_amount = Set(Some(new_current));
    
    if new_current >= target_value {
        goal_active.status = Set(Some("completed".to_string()));
    }
    
    goal_active.update(&ctx.db).await?;
    
    let new_current_i64 = new_current.to_string().parse().unwrap_or(0);
    
    Ok(Json(DepositResponse {
        success: true,
        deposit_id: deposit.id,
        goal_id: params.goal_id,
        new_current_amount: new_current_i64,
        message: format!("Deposited GHS {:.2} successfully", params.amount),
    }))
}

#[debug_handler]
pub async fn withdraw(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(goal_id): Path<i32>,
) -> Result<Json<WithdrawResponse>> {
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    let goal = savings_goals::Entity::find_by_id(goal_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    if goal.member_id != member_id {
        return Ok(Json(WithdrawResponse {
            success: false,
            amount: 0,
            message: "You don't own this savings goal".to_string(),
        }));
    }
    
    let current_amount = goal.current_amount.unwrap_or(Decimal::ZERO);
    let current_i64 = current_amount.to_string().parse().unwrap_or(0);
    let goal_name = goal.name.clone();
    
    let now = Utc::now().naive_utc();
    if goal.mode == "strict" && goal.status != Some("completed".to_string()) && now < goal.end_date {
        return Ok(Json(WithdrawResponse {
            success: false,
            amount: 0,
            message: format!("Cannot withdraw from strict savings until {}. This goal is locked.", goal.end_date),
        }));
    }
    
    let mut goal_active: savings_goals::ActiveModel = goal.into();
    goal_active.current_amount = Set(Some(Decimal::ZERO));
    goal_active.status = Set(Some("withdrawn".to_string()));
    goal_active.update(&ctx.db).await?;
    
    Ok(Json(WithdrawResponse {
        success: true,
        amount: current_i64,
        message: format!("Withdrawn GHS {:.2} from savings goal '{}'", current_i64 as f64 / 100.0, goal_name),
    }))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/member/savings")
        .add("/", post(create))
        .add("/", get(list))
        .add("/deposit", post(deposit))
        .add("/withdraw/{goal_id}", post(withdraw))
}
