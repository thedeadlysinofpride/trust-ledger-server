#![allow(clippy::missing_errors_doc)]
#![allow(clippy::unnecessary_struct_initialization)]
#![allow(clippy::unused_async)]

use loco_rs::prelude::*;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;

#[derive(Debug, Deserialize)]
pub struct MoolreUssdRequest {
    pub sessionid: String,
    pub new: bool,
    pub msisdn: String,
    pub network: i32,
    pub message: String,
    pub extension: String,
    pub data: String,
}

#[derive(Serialize)]
pub struct MoolreUssdResponse {
    pub message: String,
    pub reply: bool,
}

static USSD_SESSIONS: Lazy<Mutex<HashMap<String, String>>> =
    Lazy::new(|| Mutex::new(HashMap::new()));

#[debug_handler]
pub async fn handle(Json(req): Json<MoolreUssdRequest>) -> Result<Json<MoolreUssdResponse>> {
    tracing::info!("Moolre USSD request: {:?}", req);

    let session_id = req.sessionid.clone();
    let user_input = req.message.trim();
    let is_new_session = req.new;

    // Get or initialize session state
    let mut sessions = USSD_SESSIONS.lock().unwrap();
    let current_state = if is_new_session {
        sessions.insert(session_id.clone(), "main".to_string());
        "main".to_string()
    } else {
        sessions
            .get(&session_id)
            .cloned()
            .unwrap_or_else(|| "main".to_string())
    };

    // Handle the USSD logic using existing state machine
    let (response_text, next_state, is_terminal) = handle_ussd_logic(user_input, &current_state);

    // Update session state if not terminal
    if !is_terminal {
        sessions.insert(session_id, next_state);
    } else {
        sessions.remove(&session_id);
    }

    Ok(Json(MoolreUssdResponse {
        message: response_text,
        reply: !is_terminal, // true if we expect more input, false to end session
    }))
}

fn handle_ussd_logic(input: &str, current_state: &str) -> (String, String, bool) {
    // Main menu
    if input.is_empty() || input == "0" {
        return (
            "MOOLRE SUSU\n1. Join Group\n2. Check Balance\n3. Contribute\n4. My Trust Score\n0. Back".to_string(),
            "main".to_string(),
            false,
        );
    }

    match current_state {
        "main" => {
            match input {
                "1" => (
                    "JOIN GROUP\nEnter group code:\n(MKL for Makola, AOS for Abossey)\n\n0. Back".to_string(),
                    "join_group".to_string(),
                    false,
                ),
                "2" => (
                    "YOUR BALANCE\nCurrent Savings: GHS 0.00\n\n0. Back".to_string(),
                    "main".to_string(),
                    false,
                ),
                "3" => (
                    "CONTRIBUTE\nEnter amount in GHS:\n\n0. Back".to_string(),
                    "contribute_amount".to_string(),
                    false,
                ),
                "4" => (
                    "YOUR TRUST SCORE\nScore: 100%\nRating: EXCELLENT\n\n0. Back".to_string(),
                    "main".to_string(),
                    false,
                ),
                _ => (
                    "Invalid option.\n\n0. Back".to_string(),
                    "main".to_string(),
                    false,
                ),
            }
        }
        "join_group" => {
            if input == "0" {
                (
                    "MOOLRE SUSU\n1. Join Group\n2. Check Balance\n3. Contribute\n4. My Trust Score\n0. Back".to_string(),
                    "main".to_string(),
                    false,
                )
            } else {
                (
                    format!("Group {} joined successfully!\n\n0. Back", input.to_uppercase()),
                    "main".to_string(),
                    false,
                )
            }
        }
        "contribute_amount" => {
            match input.parse::<f64>() {
                Ok(amount) if amount > 0.0 => (
                    format!("Confirm contribution of GHS {:.2}?\n1. Yes\n0. No", amount),
                    "contribute_confirm".to_string(),
                    false,
                ),
                _ => (
                    "Invalid amount.\nEnter amount in GHS:\n\n0. Back".to_string(),
                    "contribute_amount".to_string(),
                    false,
                ),
            }
        }
        "contribute_confirm" => {
            if input == "1" {
                (
                    "Contribution successful! Thank you.\n\n0. Back".to_string(),
                    "main".to_string(),
                    false,
                )
            } else {
                (
                    "Contribution cancelled.\n\n0. Back".to_string(),
                    "main".to_string(),
                    false,
                )
            }
        }
        _ => (
            "MOOLRE SUSU\n1. Join Group\n2. Check Balance\n3. Contribute\n4. My Trust Score\n0. Back".to_string(),
            "main".to_string(),
            false,
        ),
    }
}

pub fn routes() -> Routes {
    Routes::new()
        .prefix("api/ussd/moolre")
        .add("/", post(handle))
}
