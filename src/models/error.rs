use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AppError {
    #[error("认证错误: {0}")]
    AuthError(String),

    #[error("数据库错误: {0}")]
    DatabaseError(#[from] sqlx::Error),

    #[error("验证错误: {0}")]
    ValidationError(String),

    #[error("找不到资源: {0}")]
    NotFoundError(String),

    #[error("权限错误: {0}")]
    PermissionError(String),

    #[error("内部服务器错误: {0}")]
    InternalServerError(String),

    #[error("无效的请求: {0}")]
    BadRequestError(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, error_message) = match self {
            Self::AuthError(message) => (StatusCode::UNAUTHORIZED, message),
            Self::ValidationError(message) => (StatusCode::BAD_REQUEST, message),
            Self::NotFoundError(message) => (StatusCode::NOT_FOUND, message),
            Self::PermissionError(message) => (StatusCode::FORBIDDEN, message),
            Self::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                format!("数据库错误: {}", e),
            ),
            Self::InternalServerError(message) => (StatusCode::INTERNAL_SERVER_ERROR, message),
            Self::BadRequestError(message) => (StatusCode::BAD_REQUEST, message),
        };

        let body = Json(json!({
            "code": status.as_u16(),
            "success": false,
            "message": error_message,
        }));

        (status, body).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>; 