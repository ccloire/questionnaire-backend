use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use validator::Validate;

// 数据库模型
#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct QuestionnaireResponse {
    pub id: i32,
    pub questionnaire_id: i32,
    pub respondent_id: Option<i32>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct QuestionResponse {
    pub id: i32,
    pub questionnaire_response_id: i32,
    pub question_id: i32,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct TextResponse {
    pub id: i32,
    pub question_response_id: i32,
    pub text_value: String,
}

#[derive(Debug, Serialize, Deserialize, FromRow)]
pub struct OptionResponse {
    pub id: i32,
    pub question_response_id: i32,
    pub option_id: i32,
}

// API请求和响应模型
#[derive(Debug, Serialize, Deserialize, Validate)]
pub struct SubmitResponseRequest {
    pub questionnaire_id: i32,
    pub answers: Vec<QuestionAnswer>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionAnswer {
    pub question_id: i32,
    pub answer_type: String, // "text", "option", "options"
    pub text_value: Option<String>,
    pub option_ids: Option<Vec<i32>>,
    pub option_values: Option<Vec<String>>, // 用于前端提交选项文本而非ID
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitResponseResponse {
    pub id: i32,
    pub questionnaire_id: i32,
    pub success: bool,
    pub created_at: DateTime<Utc>,
}

// 统计相关
#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionnaireStatistics {
    pub questionnaire_id: i32,
    pub title: String,
    pub response_count: i32,
    pub questions: Vec<QuestionStatistics>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct QuestionStatistics {
    pub question_id: i32,
    pub title: String,
    pub question_type: String,
    pub text_responses: Option<Vec<String>>,
    pub option_counts: Option<Vec<OptionCount>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct OptionCount {
    pub option_id: i32,
    pub option_text: String,
    pub count: i32,
    pub percentage: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseListItem {
    pub id: i32,
    pub questionnaire_id: i32,
    pub respondent: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ResponseDetails {
    pub id: i32,
    pub questionnaire_id: i32,
    pub questionnaire_title: String,
    pub respondent: Option<String>,
    pub created_at: DateTime<Utc>,
    pub answers: Vec<AnswerDetail>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AnswerDetail {
    pub question_id: i32,
    pub question_title: String,
    pub question_type: String,
    pub text_value: Option<String>,
    pub selected_options: Option<Vec<String>>,
} 