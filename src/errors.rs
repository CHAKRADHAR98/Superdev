use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[allow(dead_code)]
    #[error("Missing required fields")]
    MissingFields,
    #[error("Invalid input: {0}")]
    InvalidInput(String),
    #[error("Cryptographic error: {0}")]
    CryptoError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let body = Json(json!({
            "success": false,
            "error": self.to_string()
        }));
        
        (StatusCode::BAD_REQUEST, body).into_response()
    }
}