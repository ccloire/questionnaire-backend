mod config;
mod models;
mod routes;
mod services;
mod utils;

use std::{net::SocketAddr, sync::Arc};

use axum::Server;
use sqlx::mysql::MySqlPoolOptions;
use tokio::signal;
use tower_http::trace::TraceLayer;
use tracing::info;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::config::Config;
use crate::routes::create_router;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 初始化环境变量
    dotenv::dotenv().ok();

    // 初始化日志
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    // 加载配置
    let config = Config::from_env()?;

    // 连接数据库
    let db_pool = MySqlPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    // 设置服务器地址
    let addr = SocketAddr::from(([0, 0, 0, 0], config.server.port));

    // 创建配置和数据库的Arc实例
    let db_pool = Arc::new(db_pool);
    let config = Arc::new(config);

    // 创建路由
    let app = create_router(config, db_pool)
        .layer(TraceLayer::new_for_http());

    // 启动服务器
    info!("Starting server at {}", addr);
    
    // 针对axum 0.6.x的使用方法
    Server::bind(&addr)
        .serve(app.into_make_service())
        .with_graceful_shutdown(shutdown_signal())
        .await?;

    Ok(())
}

// 处理优雅关闭信号
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("Failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("Failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    tokio::select! {
        _ = ctrl_c => {
            info!("Received Ctrl+C, starting graceful shutdown");
        },
        _ = terminate => {
            info!("Received terminate signal, starting graceful shutdown");
        }
    }
} 