use actix_web::{error::ResponseError, http::StatusCode, HttpResponse};
use derive_more::Display;
use serde::Serialize;

#[derive(Debug, Display)]
pub enum ApiError {
    #[display(fmt = "Internal Server Error")]
    InternalServerError,

    #[display(fmt = "Bad Request: {}", _0)]
    BadRequest(String),

    #[display(fmt = "Unauthorized: {}", _0)]
    Unauthorized(String),

    #[display(fmt = "Not Found: {}", _0)]
    NotFound(String),

    #[display(fmt = "Insufficient Balance: {}", _0)]
    InsufficientBalance(String),

    #[display(fmt = "Invalid Transaction: {}", _0)]
    InvalidTransaction(String),

    #[display(fmt = "Buyback Error: {}", _0)]
    BuybackError(String),

    #[display(fmt = "Reward Distribution Error: {}", _0)]
    RewardError(String),

    #[display(fmt = "Solana Error: {}", _0)]
    SolanaError(String),
}

#[derive(Serialize)]
struct ErrorResponse {
    code: u16,
    message: String,
}

impl ResponseError for ApiError {
    fn error_response(&self) -> HttpResponse {
        let status_code = self.status_code();
        let error_response = ErrorResponse {
            code: status_code.as_u16(),
            message: self.to_string(),
        };
        HttpResponse::build(status_code).json(error_response)
    }

    fn status_code(&self) -> StatusCode {
        match self {
            ApiError::InternalServerError => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::BadRequest(_) => StatusCode::BAD_REQUEST,
            ApiError::Unauthorized(_) => StatusCode::UNAUTHORIZED,
            ApiError::NotFound(_) => StatusCode::NOT_FOUND,
            ApiError::InsufficientBalance(_) => StatusCode::BAD_REQUEST,
            ApiError::InvalidTransaction(_) => StatusCode::BAD_REQUEST,
            ApiError::BuybackError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::RewardError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            ApiError::SolanaError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}