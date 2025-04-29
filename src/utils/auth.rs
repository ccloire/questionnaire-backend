use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

use axum::{
    extract::{FromRef, FromRequestParts, State},
    http::{header, request::Parts, Request},
    middleware::Next,
    response::Response,
    body::Body,
};
use async_trait::async_trait;
use bcrypt::{hash, verify, DEFAULT_COST};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};

use crate::config::Config;
use crate::models::error::{AppError, AppResult};
use crate::models::user::Claims;

// 密码加密
pub fn hash_password(password: &str) -> AppResult<String> {
    hash(password, DEFAULT_COST)
        .map_err(|e| AppError::InternalServerError(format!("密码加密失败: {}", e)))
}

// 验证密码
pub fn verify_password(password: &str, hash: &str) -> AppResult<bool> {
    verify(password, hash)
        .map_err(|e| AppError::InternalServerError(format!("密码验证失败: {}", e)))
}

// 生成JWT令牌
pub fn generate_jwt(
    config: &Arc<Config>,
    user_id: i32,
    expiration_hours: Option<u64>,
) -> AppResult<String> {
    let expiration = expiration_hours.unwrap_or(24);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("获取当前时间失败")
        .as_secs();
    
    let claims = Claims {
        sub: user_id.to_string(),
        iat: now as usize,
        exp: (now + expiration * 3600) as usize,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(config.jwt.secret.as_bytes()),
    )
    .map_err(|e| AppError::InternalServerError(format!("生成JWT令牌失败: {}", e)))?;

    Ok(token)
}

// 验证JWT令牌
pub fn verify_jwt(config: &Arc<Config>, token: &str) -> AppResult<Claims> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(config.jwt.secret.as_bytes()),
        &Validation::default(),
    )
    .map_err(|e| AppError::AuthError(format!("无效的令牌: {}", e)))?;

    Ok(token_data.claims)
}

// JWT中间件，用于保护需要认证的路由
pub async fn auth_middleware(
    State(config): State<Arc<Config>>,
    request: Request<Body>,
    next: Next<Body>,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    match auth_header {
        Some(auth) if auth.starts_with("Bearer ") => {
            let token = auth.trim_start_matches("Bearer ").trim();
            let _claims = verify_jwt(&config, token)?;
            // 通过验证，继续请求
            let response = next.run(request).await;
            Ok(response)
        }
        _ => Err(AppError::AuthError("请提供有效的认证令牌".to_string())),
    }
}

// 用于从请求中提取当前用户ID的提取器
pub struct CurrentUser(pub i32);

#[async_trait]
impl<S> FromRequestParts<S> for CurrentUser
where
    Arc<Config>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = AppError;

    // 使用axum 0.6.x的签名格式
    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let config = Arc::from_ref(state);

        let auth_header = parts
            .headers
            .get(header::AUTHORIZATION)
            .and_then(|header| header.to_str().ok());

        match auth_header {
            Some(auth) if auth.starts_with("Bearer ") => {
                let token = auth.trim_start_matches("Bearer ").trim();
                let claims = verify_jwt(&config, token)?;
                
                let user_id = claims
                    .sub
                    .parse::<i32>()
                    .map_err(|_| AppError::AuthError("无效的用户标识".to_string()))?;
                
                Ok(CurrentUser(user_id))
            }
            _ => Err(AppError::AuthError("请提供有效的认证令牌".to_string())),
        }
    }
} 