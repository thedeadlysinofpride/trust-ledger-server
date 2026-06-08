#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use crate::models::_entities::{groups, members};
use loco_rs::prelude::*;
use sea_orm::{ColumnTrait, EntityTrait, QueryFilter, QueryOrder};

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Json<Vec<groups::Model>>> {
    let all_groups = groups::Entity::find()
        .order_by_asc(groups::Column::Id)
        .all(&ctx.db)
        .await?;

    Ok(Json(all_groups))
}

#[debug_handler]
pub async fn show(
    State(ctx): State<AppContext>,
    Path(id): Path<i32>,
) -> Result<Json<groups::Model>> {
    let group = groups::Entity::find_by_id(id)
        .one(&ctx.db)
        .await?
        .ok_or_else(|| Error::NotFound)?;

    Ok(Json(group))
}

#[debug_handler]
pub async fn members_list(
    State(ctx): State<AppContext>,
    Path(group_id): Path<i32>,
) -> Result<Json<Vec<members::Model>>> {
    let members_list = members::Entity::find()
        .filter(members::Column::GroupId.eq(group_id))
        .all(&ctx.db)
        .await?;

    Ok(Json(members_list))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/groups")
        .add("/", get(list))
        .add("/{id}", get(show))
        .add("/{group_id}/members", get(members_list))
}
