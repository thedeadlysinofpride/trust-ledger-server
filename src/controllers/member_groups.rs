#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use crate::models::_entities::{members, groups, contributions};
use loco_rs::prelude::*;
use sea_orm::{ActiveValue::Set, EntityTrait, QueryFilter, ColumnTrait, QueryOrder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct JoinGroupParams {
    pub code: String,
}

#[derive(Serialize)]
pub struct JoinGroupResponse {
    pub success: bool,
    pub group_id: i32,
    pub group_name: String,
    pub message: String,
}

#[derive(Serialize)]
pub struct MemberGroupInfo {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub description: String,
    pub member_count: i32,
    pub total_contributed: i64,
    pub contribution_amount: i64,
    pub frequency: String,
    pub next_payout_date: String,
    pub my_turn: Option<i32>,
}

#[derive(Serialize)]
pub struct MemberGroupDetailResponse {
    pub id: i32,
    pub name: String,
    pub code: String,
    pub description: String,
    pub member_count: i32,
    pub total_contributed: i64,
    pub contribution_amount: i64,
    pub frequency: String,
    pub next_payout_date: String,
    pub my_turn: Option<i32>,
    pub members: Vec<GroupMemberDetail>,
    pub my_contributions: Vec<MemberContribution>,
    pub total_my_contributions: i64,
}

#[derive(Serialize)]
pub struct GroupMemberDetail {
    pub id: i32,
    pub name: String,
    pub phone: String,
    pub trust_score: i32,
    pub turn_position: i32,
    pub total_contributed: i64,
}

#[derive(Serialize)]
pub struct MemberContribution {
    pub id: i32,
    pub amount: i64,
    pub date: String,
    pub status: String,
}

#[debug_handler]
pub async fn join(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Json(params): Json<JoinGroupParams>,
) -> Result<Json<JoinGroupResponse>> {
    // Get member ID from JWT claims
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    // Find the group by code
    let group = groups::Entity::find()
        .filter(groups::Column::Code.eq(&params.code))
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    // Clone the group name before using it
    let group_name = group.name.clone();
    
    // Check if member already belongs to this group
    let existing_member = members::Entity::find()
        .filter(members::Column::Id.eq(member_id))
        .filter(members::Column::GroupId.eq(group.id))
        .one(&ctx.db)
        .await?;
    
    if existing_member.is_some() {
        return Ok(Json(JoinGroupResponse {
            success: false,
            group_id: group.id,
            group_name,
            message: "You are already a member of this group".to_string(),
        }));
    }

    // Update the member with the group ID
    let mut member_active: members::ActiveModel = members::Entity::find_by_id(member_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?
        .into();

    member_active.group_id = Set(Some(group.id));
    member_active.update(&ctx.db).await?;

    // Increment group member count
    let mut group_active: groups::ActiveModel = group.clone().into();
    let current_count = group_active.member_count.take().flatten().unwrap_or(0);
    group_active.member_count = Set(Some(current_count + 1));
    group_active.update(&ctx.db).await?;

    Ok(Json(JoinGroupResponse {
        success: true,
        group_id: group.id,
        group_name: group_name.clone(),
        message: format!("Successfully joined {}", group_name),
    }))
}

#[debug_handler]
pub async fn list(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
) -> Result<Json<Vec<MemberGroupInfo>>> {
    // Get member ID from JWT claims
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;

    // Get the member with their group
    let member = members::Entity::find_by_id(member_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    // If member has no group, return empty list
    if member.group_id.is_none() {
        return Ok(Json(vec![]));
    }
    
    let group_id = member.group_id.unwrap();
    
    let group = groups::Entity::find_by_id(group_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    // Calculate member's turn position
    let my_turn = get_member_turn_position(&ctx.db, member_id, group_id).await?;
    
    // Convert Decimal to i64 safely
    let total_contributed: i64 = group.total_contributed
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse()
        .unwrap_or(0);
    
    // contribution_amount is required (not Option), so it's safe to use directly
    let contribution_amount: i64 = group.contribution_amount
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse()
        .unwrap_or(0);
    
    let result = vec![MemberGroupInfo {
        id: group.id,
        name: group.name.clone(),
        code: group.code.clone(),
        description: group.description.clone().unwrap_or_default(),
        member_count: group.member_count.unwrap_or(0),
        total_contributed,
        contribution_amount,
        frequency: group.frequency.clone(),
        next_payout_date: group.next_payout_date.to_string(),
        my_turn,
    }];
    
    Ok(Json(result))
}

#[debug_handler]
pub async fn show(
    auth: auth::JWT,
    State(ctx): State<AppContext>,
    Path(group_id): Path<i32>,
) -> Result<Json<MemberGroupDetailResponse>> {
    // Get member ID from JWT claims
    let member_id: i32 = auth.claims.pid.parse().map_err(|_| Error::Unauthorized("Invalid member ID".to_string()))?;
    
    // Verify member belongs to this group
    let member = members::Entity::find_by_id(member_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    if member.group_id != Some(group_id) {
        return Err(Error::Unauthorized("You are not a member of this group".to_string()));
    }
    
    // Get group details
    let group = groups::Entity::find_by_id(group_id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;
    
    // Get all members in this group with their turn positions
    let group_members = members::Entity::find()
        .filter(members::Column::GroupId.eq(group_id))
        .order_by_asc(members::Column::JoinedAt)
        .all(&ctx.db)
        .await?;
    
    let mut members_with_turns = Vec::new();
    for (index, member) in group_members.iter().enumerate() {
        members_with_turns.push(GroupMemberDetail {
            id: member.id,
            name: member.name.clone(),
            phone: member.phone.clone(),
            trust_score: member.trust_score.unwrap_or(100),
            turn_position: (index + 1) as i32,
            total_contributed: member.total_contributed
                .unwrap_or(Decimal::ZERO)
                .to_string()
                .parse()
                .unwrap_or(0),
        });
    }
    
    // Get current member's turn position
    let my_turn = get_member_turn_position(&ctx.db, member_id, group_id).await?;
    
    // Get member's contributions to this group
    let member_contributions = contributions::Entity::find()
        .filter(contributions::Column::MemberId.eq(member_id))
        .filter(contributions::Column::GroupId.eq(group_id))
        .order_by_desc(contributions::Column::Date)
        .all(&ctx.db)
        .await?;
    
    let my_contributions: Vec<MemberContribution> = member_contributions
        .into_iter()
        .map(|c| MemberContribution {
            id: c.id,
            amount: c.amount.to_string().parse().unwrap_or(0),
            date: c.date.map(|d| d.to_string()).unwrap_or_default(),
            status: c.status,
        })
        .collect();
    
    let total_my_contributions: i64 = my_contributions.iter().map(|c| c.amount).sum();
    
    // Convert group fields
    let total_contributed: i64 = group.total_contributed
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse()
        .unwrap_or(0);
    
    let contribution_amount: i64 = group.contribution_amount
        .unwrap_or(Decimal::ZERO)
        .to_string()
        .parse()
        .unwrap_or(0);
    
    Ok(Json(MemberGroupDetailResponse {
        id: group.id,
        name: group.name,
        code: group.code,
        description: group.description.unwrap_or_default(),
        member_count: group.member_count.unwrap_or(0),
        total_contributed,
        contribution_amount,
        frequency: group.frequency,
        next_payout_date: group.next_payout_date.to_string(),
        my_turn,
        members: members_with_turns,
        my_contributions,
        total_my_contributions,
    }))
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
        .prefix("api/member/groups")
        .add("/join", post(join))
        .add("/", get(list))
        .add("/{group_id}", get(show))
}
