use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Questionnaire {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub is_public: bool,
    pub creator_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct Question {
    pub id: i32,
    pub questionnaire_id: i32,
    pub title: String,
    pub question_type: String,
    pub required: bool,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct QuestionOption {
    pub id: i32,
    pub question_id: i32,
    pub option_text: String,
    pub display_order: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

// API请求和响应模型
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct CreateQuestionnaireRequest {
    #[validate(length(min = 1, max = 100, message = "问卷标题不能为空且长度不能超过100"))]
    pub title: String,
    
    #[validate(length(max = 1000, message = "问卷描述长度不能超过1000"))]
    pub description: String,
    
    pub is_public: bool,
    
    pub questions: Vec<QuestionRequest>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionRequest {
    pub id: Option<i32>,
    pub title: String,
    #[serde(rename = "type")]
    pub question_type: String, // "text", "radio", "checkbox"
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionnaireResponse {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub is_public: bool,
    pub creator_id: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub questions: Vec<QuestionResponse>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionResponse {
    pub id: i32,
    pub title: String,
    #[serde(rename = "type")]
    pub question_type: String,
    pub required: bool,
    pub options: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionnaireListItem {
    pub id: i32,
    pub title: String,
    pub description: String,
    pub creator: String,
    pub created_at: DateTime<Utc>,
    pub response_count: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionnaireListResponse {
    pub items: Vec<QuestionnaireListItem>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
} 