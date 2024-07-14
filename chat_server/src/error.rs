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

    #[error("Invalid file path: {0}")]
    ChatFileError(String),

    #[error("create chat error: {0}")]
    CreateChatError(String),

    #[error("create message error: {0}")]
    CreateMessageError(String),

    #[error("http header parse error: {0}")]
    HttpHeader(#[from] axum::http::header::InvalidHeaderValue),

    #[error("std io error: {0}")]
    IOError(#[from] std::io::Error),

    #[error("jwt error: {0}")]
    Jwt(#[from] jwt_simple::Error),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("sqlx error: {0}")]
    Sqlx(#[from] sqlx::Error),

    #[error("conflict error: {0}")]
    EmailAlreadyExists(String),

    #[error("verify chat error: {0}")]
    VerifyChatError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::Argon2(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::ChatFileError(_) => StatusCode::BAD_REQUEST,
            AppError::CreateChatError(_) => StatusCode::BAD_REQUEST,
            AppError::CreateMessageError(_) => StatusCode::BAD_REQUEST,
            AppError::EmailAlreadyExists(_) => StatusCode::CONFLICT,
            AppError::HttpHeader(_) => StatusCode::UNPROCESSABLE_ENTITY,
            AppError::IOError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::Jwt(_) => StatusCode::FORBIDDEN,
            AppError::NotFound(_) => StatusCode::NOT_FOUND,
            AppError::Sqlx(_) => StatusCode::INTERNAL_SERVER_ERROR,
            AppError::VerifyChatError(_) => StatusCode::FORBIDDEN,
        };
        (status, Json(json!({"error": self.to_string()}))).into_response()
    }
}
