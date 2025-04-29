use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub nickname: String,
    pub password_hash: String,
    pub email: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateUserRequest {
    #[validate(length(min = 3, max = 20, message = "用户名长度必须在3-20之间"))]
    pub username: String,
    
    #[validate(length(min = 1, max = 50, message = "昵称不能为空且长度不能超过50"))]
    pub nickname: String,
    
    #[validate(length(min = 6, message = "密码长度不能少于6个字符"))]
    pub password: String,
    
    #[validate(email(message = "邮箱格式不正确"))]
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct LoginRequest {
    #[validate(length(min = 3, max = 20, message = "用户名长度必须在3-20之间"))]
    pub username: String,
    
    #[validate(length(min = 6, message = "密码长度不能少于6个字符"))]
    pub password: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct UserResponse {
    pub id: i32,
    pub username: String,
    pub nickname: String,
    pub email: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthResponse {
    pub token: String,
    pub user: UserResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String, // 用户ID
    pub exp: usize,  // 过期时间
    pub iat: usize,  // 颁发时间
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            nickname: user.nickname,
            email: user.email,
        }
    }
} 