mod user_routes;
mod questionnaire_routes;
mod response_routes;

use axum::Router;
use axum::http::{Method, HeaderName, HeaderValue};
use sqlx::MySqlPool;
use std::sync::Arc;
use tower_http::cors::{CorsLayer, Any};

use crate::config::Config;

pub fn create_router(config: Arc<Config>, db_pool: Arc<MySqlPool>) -> Router {
    // 允许的请求头
    let allowed_headers = vec![
        HeaderName::from_static("authorization"),
        HeaderName::from_static("content-type"),
        HeaderName::from_static("x-requested-with"),
    ];

    // 允许的方法
    let allowed_methods = vec![
        Method::GET,
        Method::POST,
        Method::PUT,
        Method::DELETE,
        Method::OPTIONS,
    ];

    // 配置CORS - 简化为允许任何源，但指定凭证false以避免CORS报错
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(allowed_methods)
        .allow_headers(allowed_headers)
        .allow_credentials(false); // 设置为false以允许使用Any源

    // 主路由
    Router::new()
        .nest("/users", user_routes::routes(config.clone(), db_pool.clone()))
        .nest(
            "/questionnaires",
            questionnaire_routes::routes(config.clone(), db_pool.clone()),
        )
        .nest("/responses", response_routes::routes(config.clone(), db_pool.clone()))
        .layer(cors)
} 