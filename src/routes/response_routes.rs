use std::sync::Arc;

use axum::{
    extract::{FromRef, Path, Query, State},
    middleware,
    routing::{get, post},
    Json, Router,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use validator::Validate;

use crate::config::Config;
use crate::models::error::{AppError, AppResult};
use crate::models::response::SubmitResponseRequest;
use crate::services::response_service::ResponseService;
use crate::utils::auth::{auth_middleware, CurrentUser};
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

#[derive(Debug, Deserialize)]
struct PaginationQuery {
    page: Option<i64>,
    page_size: Option<i64>,
}

// 提交问卷回答
async fn submit_response(
    State(state): State<AppState>,
    Json(req): Json<SubmitResponseRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // 提交回答 - 匿名回答无需用户ID
    let service = ResponseService::new(state.db, state.config);
    let response = service.submit_response(None, req).await?;

    Ok(ApiResponse::success(response, "问卷提交成功"))
}

// 提交问卷回答 - 已认证用户
async fn submit_response_auth(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<SubmitResponseRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // 提交回答 - 使用认证用户ID
    let service = ResponseService::new(state.db, state.config);
    let response = service.submit_response(Some(current_user.0), req).await?;

    Ok(ApiResponse::success(response, "问卷提交成功"))
}

// 获取问卷的统计信息
async fn get_questionnaire_statistics(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(questionnaire_id): Path<i32>,
) -> AppResult<impl axum::response::IntoResponse> {
    let service = ResponseService::new(state.db, state.config);
    let stats = service
        .get_questionnaire_statistics(current_user.0, questionnaire_id)
        .await?;

    Ok(ApiResponse::success(stats, "获取问卷统计信息成功"))
}

// 获取问卷的回答列表
async fn get_questionnaire_responses(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(questionnaire_id): Path<i32>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let service = ResponseService::new(state.db, state.config);
    let responses = service
        .get_questionnaire_responses(current_user.0, questionnaire_id, page, page_size)
        .await?;

    Ok(ApiResponse::success(responses, "获取问卷回答列表成功"))
}

// 获取回答详情
async fn get_response_detail(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(response_id): Path<i32>,
) -> AppResult<impl axum::response::IntoResponse> {
    let service = ResponseService::new(state.db, state.config);
    let detail = service.get_response_detail(current_user.0, response_id).await?;

    Ok(ApiResponse::success(detail, "获取回答详情成功"))
}

// 创建问卷回答路由
pub fn routes(config: Arc<Config>, db: Arc<MySqlPool>) -> Router {
    let state = AppState { config: config.clone(), db: db.clone() };
    
    // 不需要认证的路由
    let public_routes = Router::new()
        .route("/submit", post(submit_response))
        .with_state(state.clone());

    // 需要认证的路由
    let authenticated_routes = Router::new()
        .route("/submit/auth", post(submit_response_auth))
        .route("/questionnaires/:id/statistics", get(get_questionnaire_statistics))
        .route("/questionnaires/:id/responses", get(get_questionnaire_responses))
        .route("/:id", get(get_response_detail))
        .route_layer(middleware::from_fn_with_state(
            config.clone(),
            auth_middleware,
        ))
        .with_state(state);

    // 合并路由
    Router::new()
        .merge(public_routes)
        .merge(authenticated_routes)
} 