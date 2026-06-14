#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use crate::models::_entities::{members, groups, contributions};
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, QueryFilter, ColumnTrait, QuerySelect, PaginatorTrait, QueryOrder};
use serde::Serialize;

#[derive(Serialize)]
pub struct MemberDashboardResponse {
    pub total_savings: i64,
    pub active_groups: i32,
    pub trust_score: i32,
    pub total_contributions: i64,
    pub recent_contributions: Vec<RecentContribution>,
    pub upcoming_payouts: Vec<UpcomingPayout>,
}

#[derive(Serialize)]
pub struct RecentContribution {
    pub id: i32,
    pub group_name: String,
    pub amount: i64,
    pub date: String,
    pub status: String,
}

#[derive(Serialize)]
pub struct UpcomingPayout {
    pub group_id: i32,
    pub group_name: String,
    pub payout_date: String,
    pub my_turn: Option<i32>,
    pub total_members: i32,
}

#[debug_handler]
pub async fn dashboard(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Json<MemberDashboardResponse>> {
    // Get member ID from JWT claims
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    tracing::info!("Member dashboard requested for member_id: {}", member_id);
    
    // Get member details
    let member = members::Entity::find_by_id(member_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    tracing::info!("Member found: {}", member.id);
    
    // Get all contributions by this member (join with groups table)
    let contributions_list = contributions::Entity::find()
        .filter(contributions::Column::MemberId.eq(member_id))
        .find_also_related(groups::Entity)
        .order_by_desc(contributions::Column::Date)
        .limit(10)
        .all(&ctx.db)
        .await?;
    
    tracing::info!("Contributions found: {}", contributions_list.len());
    
    let recent_contributions: Vec<RecentContribution> = contributions_list
        .into_iter()
        .filter_map(|(contribution, group)| {
            group.map(|g| {
                let amount_str = contribution.amount.to_string();
                let amount_int = amount_str.parse::<i64>().unwrap_or(0);
                let date_str = contribution.date.map(|d| d.to_string()).unwrap_or_default();
                
                RecentContribution {
                    id: contribution.id,
                    group_name: g.name,
                    amount: amount_int,
                    date: date_str,
                    status: contribution.status,
                }
            })
        })
        .collect();
    
    // Get all groups this member belongs to (only those with non-NULL group_id)
    let member_groups = members::Entity::find()
        .filter(members::Column::Id.eq(member_id))
        .filter(members::Column::GroupId.is_not_null())
        .find_also_related(groups::Entity)
        .all(&ctx.db)
        .await?;
    
    tracing::info!("Member groups found: {}", member_groups.len());
    
    let active_groups_count = member_groups.len() as i32;
    
    // Calculate total savings (sum of all contributions)
    let total_savings: i64 = contributions::Entity::find()
        .filter(contributions::Column::MemberId.eq(member_id))
        .filter(contributions::Column::Status.eq("completed"))
        .select_only()
        .column_as(contributions::Column::Amount.sum(), "total")
        .into_tuple::<Option<i64>>()
        .one(&ctx.db)
        .await?
        .flatten()
        .unwrap_or(0);
    
    tracing::info!("Total savings: {}", total_savings);
    
    // Calculate total contributions count
    let total_contributions_count = contributions::Entity::find()
        .filter(contributions::Column::MemberId.eq(member_id))
        .filter(contributions::Column::Status.eq("completed"))
        .count(&ctx.db)
        .await?;
    
    tracing::info!("Total contributions count: {}", total_contributions_count);
    
    // Get upcoming payouts for groups where member is active
    let mut upcoming_payouts = Vec::new();
    
    for (_member_record, group_opt) in &member_groups {
        if let Some(group) = group_opt {
            // Check if payout date is in the future
            let today = chrono::Utc::now().naive_utc();
            if group.next_payout_date > today {
                let my_turn = get_member_turn_position(&ctx.db, member_id, group.id).await?;
                
                upcoming_payouts.push(UpcomingPayout {
                    group_id: group.id,
                    group_name: group.name.clone(),
                    payout_date: group.next_payout_date.to_string(),
                    my_turn,
                    total_members: group.member_count.unwrap_or(0),
                });
            }
        }
    }

    let trust_score = member.trust_score.unwrap_or(100);
    tracing::info!("Trust score: {}", trust_score);

    let response = MemberDashboardResponse {
        total_savings,
        active_groups: active_groups_count,
        trust_score,
        total_contributions: total_contributions_count as i64,
        recent_contributions,
        upcoming_payouts,
    };
    
    Ok(Json(response))
}

// Helper function to get member's turn position in a rotating group
async fn get_member_turn_position(db: &sea_orm::DatabaseConnection, member_id: i32, group_id: i32) -> Result<Option<i32>> {
    let all_members = members::Entity::find()
        .filter(members::Column::GroupId.eq(group_id))
        .order_by_asc(members::Column::JoinedAt)
        .all(db)
        .await?;
    
    for (index, member) in all_members.iter().enumerate() {
        if member.id == member_id {
            return Ok(Some((index + 1) as i32));
        }
    }
    
    Ok(None)
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/member/dashboard")
        .add("/", get(dashboard))
}
