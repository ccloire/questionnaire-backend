use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use serde_json::json;

#[derive(Debug, Serialize)]
pub struct ApiResponse<T>
where
    T: Serialize,
{
    pub code: u16,
    pub success: bool,
    pub message: String,
    pub data: Option<T>,
}

impl<T> ApiResponse<T>
where
    T: Serialize,
{
    pub fn success(data: T, message: impl Into<String>) -> Self {
        Self {
            code: 200,
            success: true,
            message: message.into(),
            data: Some(data),
        }
    }

    pub fn error(status_code: StatusCode, message: impl Into<String>) -> Self {
        Self {
            code: status_code.as_u16(),
            success: false,
            message: message.into(),
            data: None,
        }
    }
}

impl<T> IntoResponse for ApiResponse<T>
where
    T: Serialize,
{
    fn into_response(self) -> Response {
        let status = match self.code {
            200..=299 => StatusCode::OK,
            400 => StatusCode::BAD_REQUEST,
            401 => StatusCode::UNAUTHORIZED,
            403 => StatusCode::FORBIDDEN,
            404 => StatusCode::NOT_FOUND,
            500 => StatusCode::INTERNAL_SERVER_ERROR,
            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        let body = Json(json!({
            "code": self.code,
            "success": self.success,
            "message": self.message,
            "data": self.data,
        }));

        (status, body).into_response()
    }
} 