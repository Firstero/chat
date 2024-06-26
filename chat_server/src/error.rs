use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("argon2 error: {0}")]
    Argon2(#[from] argon2::password_hash::Error),

    #[error("create chat error: {0}")]
    CreateChatError(String),

    #[error("http header parse error: {0}")]
    HttpHeader(#[from] axum::http::header::InvalidHeaderValue),

    #[error("std io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("jwt error: {0}")]
    Jwt(#[from] jwt_simple::Error),

    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("conflict error: {0}")]
    EmailAlreadyExists(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Argon2(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::HttpHeader(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::Jwt(_) => StatusCode::FORBIDDEN,
            AppError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::CreateChatError(_) => StatusCode::BAD_REQUEST,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };
        (status, Json(json!({"error": self.to_string()}))).into_response()
    }
}
