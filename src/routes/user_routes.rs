use std::sync::Arc;

use axum::{
    extract::{FromRef, State},
    routing::{get, post},
    Json, Router,
};
use sqlx::MySqlPool;
use validator::Validate;

use crate::config::Config;
use crate::models::error::{AppError, AppResult};
use crate::models::user::{CreateUserRequest, LoginRequest};
use crate::services::user_service::UserService;
use crate::utils::auth::CurrentUser;
use crate::utils::response::ApiResponse;

// 定义应用程序状态
#[derive(Clone)]
pub struct AppState {
    pub config: Arc<Config>,
    pub db: Arc<MySqlPool>,
}

// 为AppState实现FromRef，使CurrentUser可以从中提取Config
impl FromRef<AppState> for Arc<Config> {
    fn from_ref(state: &AppState) -> Self {
        state.config.clone()
    }
}

// 用户注册
async fn register(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // 创建用户
    let user_service = UserService::new(state.db, state.config);
    let user = user_service.create_user(req).await?;

    Ok(ApiResponse::success(user, "用户注册成功"))
}

// 用户登录
async fn login(
    State(state): State<AppState>,
    Json(req): Json<LoginRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // 登录
    let user_service = UserService::new(state.db, state.config);
    let auth = user_service.login(req).await?;

    Ok(ApiResponse::success(auth, "登录成功"))
}

// 获取当前用户信息
async fn get_current_user(
    State(state): State<AppState>,
    current_user: CurrentUser,
) -> AppResult<impl axum::response::IntoResponse> {
    let user_service = UserService::new(state.db, state.config);
    let user = user_service.get_user_response_by_id(current_user.0).await?;

    Ok(ApiResponse::success(user, "获取用户信息成功"))
}

// 创建用户路由
pub fn routes(config: Arc<Config>, db: Arc<MySqlPool>) -> Router {
    let state = AppState { config, db };
    
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
        .route("/me", get(get_current_user))
        .with_state(state)
} 