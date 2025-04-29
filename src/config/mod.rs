use serde::Deserialize;
use std::env;
use anyhow::Result;

#[derive(Clone, Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Clone, Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        // 尝试加载.env文件，如果存在的话
        let _ = dotenv::dotenv();

        let server = ServerConfig {
            host: env::var("SERVER_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: env::var("SERVER_PORT")
                .unwrap_or_else(|_| "3000".to_string())
                .parse()
                .unwrap_or(3000),
        };

        let database = DatabaseConfig {
            url: env::var("DATABASE_URL").expect("DATABASE_URL must be set"),
            max_connections: env::var("MAX_CONNECTIONS")
                .unwrap_or_else(|_| "10".to_string())
                .parse()
                .unwrap_or(10),
        };

        let jwt = JwtConfig {
            secret: env::var("JWT_SECRET").expect("JWT_SECRET must be set"),
            expiration: env::var("JWT_EXPIRATION").unwrap_or_else(|_| "24h".to_string()),
        };

        Ok(Config {
            server,
            database,
            jwt,
        })
    }
} 