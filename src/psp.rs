use axum::{Json, http::StatusCode};

use uuid::Uuid;

use crate::models::{PSPChargeRequest, PSPChargeResponse};

pub async fn charge(
    Json(payload): Json<PSPChargeRequest>,
) -> Result<Json<PSPChargeResponse>, StatusCode> {
    println!("Mock PSP called token={}", payload.card_token);

    match payload.card_token.as_str() {
        "tok_success" => Ok(Json(PSPChargeResponse {
            success: true,
            psp_ref: format!("psp_{}", Uuid::new_v4()),
            status: "success".to_string(),
        })),

        "tok_insufficient_funds" => Ok(Json(PSPChargeResponse {
            success: false,
            psp_ref: String::new(),
            status: "insufficient_funds".to_string(),
        })),

        "tok_card_declined" => Ok(Json(PSPChargeResponse {
            success: false,
            psp_ref: String::new(),
            status: "card_declined".to_string(),
        })),

        "tok_timeout" => Ok(Json(PSPChargeResponse {
            success: false,
            psp_ref: String::new(),
            status: "timeout".to_string(),
        })),

        "tok_network_error" => Ok(Json(PSPChargeResponse {
            success: false,
            psp_ref: String::new(),
            status: "network_error".to_string(),
        })),

        _ => Ok(Json(PSPChargeResponse {
            success: false,
            psp_ref: String::new(),
            status: "unknown_token".to_string(),
        })),
    }
}
