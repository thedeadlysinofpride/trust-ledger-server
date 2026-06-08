#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use sea_orm::QueryOrder;

use crate::models::_entities::groups;

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Json<Vec<groups::Model>>> {
    let all_groups = groups::Entity::find()
        .order_by_asc(groups::Column::Id)
        .all(&ctx.db)
        .await?;

    Ok(Json(all_groups))
}

pub fn routes() -> Routes {
    Routes::new().prefix("api/groups/").add("/", get(list))
}
