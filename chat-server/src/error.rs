use axum::response::{IntoResponse, Response};
use axum::{http, Json};
use serde::{Deserialize, Serialize};

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("sqlx error: {0}")]
    SqlxError(#[from] sqlx::Error),

    #[error("email already exists: {0}")]
    EmailAlreadyExists(String),

    #[error("password hash error: {0}")]
    PasswordHashError(#[from] argon2::password_hash::Error),

    #[error("jwt error: {0}")]
    JwtError(#[from] jwt_simple::Error),

    #[error("http header parse error: {0}")]
    HttpHeaderError(#[from] axum::http::header::InvalidHeaderValue),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let status = match &self {
            AppError::SqlxError(_) => http::StatusCode::INTERNAL_SERVER_ERROR,
            AppError::EmailAlreadyExists(_) => http::StatusCode::CONFLICT,
            AppError::PasswordHashError(_) => http::StatusCode::UNPROCESSABLE_ENTITY,
            AppError::JwtError(_) => http::StatusCode::FORBIDDEN,
            AppError::HttpHeaderError(_) => http::StatusCode::UNPROCESSABLE_ENTITY,
        };
        (status, Json(ErrorOutput::new(self.to_string()))).into_response()
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorOutput {
    pub error: String,
}
impl ErrorOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}
