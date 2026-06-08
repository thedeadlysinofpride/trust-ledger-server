#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use crate::models::_entities::{groups, members};
use loco_rs::prelude::*;
use sea_orm::PaginatorTrait;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter};

pub async fn stats(State(ctx): State<AppContext>) -> Result<Json<serde_json::Value>> {
    let total_groups = groups::Entity::find()
        .filter(groups::Column::IsActive.eq(true))
        .count(&ctx.db)
        .await?;
    let total_members = members::Entity::find().count(&ctx.db).await?;
    let all_groups = groups::Entity::find().all(&ctx.db).await?;

    let total_contributions: i64 = all_groups
        .iter()
        .map(|g| {
            g.total_contributed
                .unwrap_or(Decimal::ZERO)
                .to_string()
                .parse::<i64>()
                .unwrap_or(0)
        })
        .sum();

    let (trust_sum, trust_count) = members::Entity::find()
        .all(&ctx.db)
        .await?
        .iter()
        .fold((0i64, 0i64), |(sum, count), m| {
            (sum + m.trust_score.unwrap_or(0) as i64, count + 1)
        });

    let average_trust_score = if trust_count == 0 {
        0
    } else {
        (trust_sum / trust_count) as i32
    };

    let mut trust_breakdown = Vec::new();

    for group in &all_groups {
        let members_in_group = members::Entity::find()
            .filter(members::Column::GroupId.eq(group.id))
            .all(&ctx.db)
            .await?;

        let avg_score = if members_in_group.is_empty() {
            0
        } else {
            let sum: i32 = members_in_group
                .iter()
                .map(|m| m.trust_score.unwrap_or(0))
                .sum();
            sum / members_in_group.len() as i32
        };

        trust_breakdown.push(serde_json::json!({
            "groupName": group.name,
            "score": avg_score
        }));
    }

    Ok(Json(serde_json::json!({
        "totalGroups": total_groups,
        "totalMembers": total_members,
        "totalContributions": total_contributions,
        "averageTrustScore": average_trust_score,
        "trustBreakdown": trust_breakdown
    })))
}

pub fn routes() -> Routes {
    Routes::new().prefix("api/dashboards/").add("/", get(stats))
}
