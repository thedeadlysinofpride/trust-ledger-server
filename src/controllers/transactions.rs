#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use crate::models::_entities::transactions;
use loco_rs::prelude::*;
use sea_orm::{EntityTrait, QueryOrder};

#[debug_handler]
pub async fn list(State(ctx): State<AppContext>) -> Result<Json<Vec<transactions::Model>>> {
    let all_transactions = transactions::Entity::find()
        .order_by_desc(transactions::Column::Date)
        .all(&ctx.db)
        .await?;

    Ok(Json(all_transactions))
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/transactions/")
        .add("/", get(list))
}
