#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]
use loco_rs::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct UssdRequest {
    pub session_id: String,
    pub phone_number: String,
    pub input: String,
    pub menu_state: String,
}

#[derive(Serialize)]
pub struct UssdResponse {
    pub response: String,
    pub menu_state: String,
    pub is_terminal: bool,
}

#[debug_handler]
pub async fn handle(Json(req): Json<UssdRequest>) -> Result<Json<UssdResponse>> {
    let input = req.input.trim();

    // Main Menu
    // Handles empty input, "0" or "*713#"
    if input.is_empty() || input == "0" || input == "*713#" {
        return Ok(Json(UssdResponse {
            response: "MOOLRE SUSU\n1. Join Group\n2. Check Balance\n3. Contribute\n4. My Trust Score\n0. Back".to_string(),
            menu_state: "main".to_string(),
            is_terminal: false,
        }));
    }

    match input {
        "1" if req.menu_state == "contribute_confirm" => Ok(Json(UssdResponse {
            response: "Contribution successful! Thank you.\n\n0. Back".to_string(),
            menu_state: "main".to_string(),
            is_terminal: false,
        })),
        "1" => Ok(Json(UssdResponse {
            response: "JOIN GROUP\nEnter group code:\n(MKL for Makola, AOS for Abossey)\n\n0. Back"
                .to_string(),
            menu_state: "join_group".to_string(),
            is_terminal: false,
        })),
        "2" => Ok(Json(UssdResponse {
            response: "YOUR BALANCE\nCurrent Savings: GHS 0.00\n\n0. Back".to_string(),
            menu_state: "main".to_string(),
            is_terminal: false,
        })),
        "3" => Ok(Json(UssdResponse {
            response: "CONTRIBUTE\nEnter amount in GHS:\n\n0. Back".to_string(),
            menu_state: "contribute_amount".to_string(),
            is_terminal: false,
        })),
        "4" => Ok(Json(UssdResponse {
            response: "YOUR TRUST SCORE\nScore: 100%\nRating: EXCELLENT\n\n0. Back".to_string(),
            menu_state: "main".to_string(),
            is_terminal: false,
        })),
        _ if req.menu_state == "contribute_amount" => match input.parse::<f64>() {
            Ok(amount) if amount > 0.0 => Ok(Json(UssdResponse {
                response: format!("Confirm contribution of GHS {:.2}?\n1. Yes\n0. No", amount),
                menu_state: "contribute_confirm".to_string(),
                is_terminal: false,
            })),
            _ => Ok(Json(UssdResponse {
                response: "Invalid amount.\nEnter amount in GHS:\n\n0. Back".to_string(),
                menu_state: "contribute_amount".to_string(),
                is_terminal: false,
            })),
        },
        _ => Ok(Json(UssdResponse {
            response: "Invalid option.\n\n0. Back".to_string(),
            menu_state: "main".to_string(),
            is_terminal: false,
        })),
    }
}

pub fn routes() -> Routes {
    Routes::new().prefix("api/ussd/").add("/", post(handle))
}
