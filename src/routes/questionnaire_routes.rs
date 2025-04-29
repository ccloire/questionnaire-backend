use std::sync::Arc;

use axum::{
    extract::{FromRef, Path, Query, State},
    middleware,
    routing::{delete, get, post, put},
    Json, Router,
};
use serde::Deserialize;
use sqlx::MySqlPool;
use validator::Validate;

use crate::config::Config;
use crate::models::error::{AppError, AppResult};
use crate::models::questionnaire::CreateQuestionnaireRequest;
use crate::services::questionnaire_service::QuestionnaireService;
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
    search: Option<String>,
}

// 创建问卷
async fn create_questionnaire(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Json(req): Json<CreateQuestionnaireRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // 创建问卷
    let service = QuestionnaireService::new(state.db, state.config);
    let questionnaire = service.create_questionnaire(current_user.0, req).await?;

    Ok(ApiResponse::success(questionnaire, "问卷创建成功"))
}

// 更新问卷
async fn update_questionnaire(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i32>,
    Json(req): Json<CreateQuestionnaireRequest>,
) -> AppResult<impl axum::response::IntoResponse> {
    // 验证请求
    req.validate()
        .map_err(|e| AppError::ValidationError(e.to_string()))?;

    // 更新问卷
    let service = QuestionnaireService::new(state.db, state.config);
    let questionnaire = service.update_questionnaire(current_user.0, id, req).await?;

    Ok(ApiResponse::success(questionnaire, "问卷更新成功"))
}

// 获取问卷详情
async fn get_questionnaire(
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> AppResult<impl axum::response::IntoResponse> {
    let service = QuestionnaireService::new(state.db, state.config);
    let questionnaire = service.get_questionnaire(id).await?;

    Ok(ApiResponse::success(questionnaire, "获取问卷成功"))
}

// 获取我的问卷列表
async fn get_my_questionnaires(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Query(query): Query<PaginationQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let service = QuestionnaireService::new(state.db, state.config);
    let questionnaires = service
        .get_user_questionnaires(current_user.0, page, page_size)
        .await?;

    Ok(ApiResponse::success(questionnaires, "获取我的问卷列表成功"))
}

// 获取公开问卷列表
async fn get_public_questionnaires(
    State(state): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> AppResult<impl axum::response::IntoResponse> {
    let page = query.page.unwrap_or(1);
    let page_size = query.page_size.unwrap_or(10);

    let service = QuestionnaireService::new(state.db, state.config);
    let questionnaires = service
        .get_public_questionnaires(query.search, page, page_size)
        .await?;

    Ok(ApiResponse::success(questionnaires, "获取公开问卷列表成功"))
}

// 删除问卷
async fn delete_questionnaire(
    State(state): State<AppState>,
    current_user: CurrentUser,
    Path(id): Path<i32>,
) -> AppResult<impl axum::response::IntoResponse> {
    let service = QuestionnaireService::new(state.db, state.config);
    service.delete_questionnaire(current_user.0, id).await?;

    Ok(ApiResponse::success(
        serde_json::json!({"id": id}),
        "问卷删除成功",
    ))
}

// 创建问卷路由
pub fn routes(config: Arc<Config>, db: Arc<MySqlPool>) -> Router {
    let state = AppState { config: config.clone(), db: db.clone() };
    
    // 需要认证的路由
    let authenticated_routes = Router::new()
        .route("/", post(create_questionnaire))
        .route("/:id", put(update_questionnaire))
        .route("/my", get(get_my_questionnaires))
        .route("/:id", delete(delete_questionnaire))
        .route_layer(middleware::from_fn_with_state(
            config.clone(),
            auth_middleware,
        ))
        .with_state(state.clone());

    // 不需要认证的路由
    let public_routes = Router::new()
        .route("/:id", get(get_questionnaire))
        .route("/public", get(get_public_questionnaires))
        .with_state(state);

    // 合并路由
    Router::new()
        .merge(authenticated_routes)
        .merge(public_routes)
} 