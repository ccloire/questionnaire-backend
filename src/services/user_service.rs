use std::sync::Arc;
use sqlx::{MySql, Pool};

use crate::models::error::{AppError, AppResult};
use crate::models::user::{
    AuthResponse, CreateUserRequest, LoginRequest, User, UserResponse,
};
use crate::utils::auth::{generate_jwt, hash_password, verify_password};
use crate::config::Config;

pub struct UserService {
    db: Arc<Pool<MySql>>,
    config: Arc<Config>,
}

impl UserService {
    pub fn new(db: Arc<Pool<MySql>>, config: Arc<Config>) -> Self {
        Self { db, config }
    }

    pub async fn create_user(&self, req: CreateUserRequest) -> AppResult<UserResponse> {
        // 检查用户名是否已存在
        let existing_user = sqlx::query!(
            "SELECT id FROM users WHERE username = ?",
            req.username
        )
        .fetch_optional(&*self.db)
        .await?;

        if existing_user.is_some() {
            return Err(AppError::ValidationError("用户名已被使用".to_string()));
        }

        // 哈希密码
        let password_hash = hash_password(&req.password)?;

        // 插入用户
        let user_id = sqlx::query!(
            r#"
            INSERT INTO users (username, nickname, password_hash, email)
            VALUES (?, ?, ?, ?)
            "#,
            req.username,
            req.nickname,
            password_hash,
            req.email
        )
        .execute(&*self.db)
        .await?
        .last_insert_id() as i32;

        // 查询并返回创建的用户
        let user = self.get_user_by_id(user_id).await?;
        
        Ok(UserResponse::from(user))
    }

    pub async fn login(&self, req: LoginRequest) -> AppResult<AuthResponse> {
        // 查找用户
        let user = sqlx::query!(
            r#"
            SELECT id, username, nickname, password_hash, email, 
                   created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                   updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM users
            WHERE username = ?
            "#,
            req.username
        )
        .fetch_optional(&*self.db)
        .await?
        .ok_or_else(|| AppError::AuthError("用户名或密码不正确".to_string()))?;

        // 验证密码
        let is_valid = verify_password(&req.password, &user.password_hash)?;
        if !is_valid {
            return Err(AppError::AuthError("用户名或密码不正确".to_string()));
        }

        // 转换为用户模型
        let user = User {
            id: user.id,
            username: user.username,
            nickname: user.nickname,
            password_hash: user.password_hash,
            email: user.email,
            created_at: user.created_at.expect("创建时间不应为空"),
            updated_at: user.updated_at.expect("更新时间不应为空"),
        };

        // 生成JWT令牌
        let token = generate_jwt(&self.config, user.id, None)?;

        // 返回认证信息
        Ok(AuthResponse {
            token,
            user: UserResponse::from(user),
        })
    }

    pub async fn get_user_by_id(&self, user_id: i32) -> AppResult<User> {
        let user = sqlx::query!(
            r#"
            SELECT id, username, nickname, password_hash, email, 
                   created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                   updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM users
            WHERE id = ?
            "#,
            user_id
        )
        .fetch_optional(&*self.db)
        .await?
        .ok_or_else(|| AppError::NotFoundError(format!("未找到ID为{}的用户", user_id)))?;

        Ok(User {
            id: user.id,
            username: user.username,
            nickname: user.nickname,
            password_hash: user.password_hash,
            email: user.email,
            created_at: user.created_at.expect("创建时间不应为空"),
            updated_at: user.updated_at.expect("更新时间不应为空"),
        })
    }

    pub async fn get_user_response_by_id(&self, user_id: i32) -> AppResult<UserResponse> {
        let user = self.get_user_by_id(user_id).await?;
        Ok(UserResponse::from(user))
    }
} 