#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde_json::Value;

#[debug_handler]
pub async fn moolre_callback(
    State(_ctx): State<AppContext>,
    Json(payload): Json<Value>,
) -> Result<Response> {
    tracing::info!("Moolre callback: {:?}", payload);

    // TODO: Update contribution/payout status in database
    // For now, just log and return success

    format::empty_json()
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/webhooks/moolre")
        .add("/", post(moolre_callback))
}
