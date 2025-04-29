use std::sync::Arc;
use sqlx::{MySql, Pool, Transaction};

use crate::models::error::{AppError, AppResult};
use crate::models::questionnaire::{
    CreateQuestionnaireRequest, Question, QuestionResponse, Questionnaire,
    QuestionnaireListItem, QuestionnaireListResponse, QuestionnaireResponse,
};
use crate::config::Config;

pub struct QuestionnaireService {
    db: Arc<Pool<MySql>>,
    config: Arc<Config>,
}

impl QuestionnaireService {
    pub fn new(db: Arc<Pool<MySql>>, config: Arc<Config>) -> Self {
        Self { db, config }
    }

    // 创建问卷
    pub async fn create_questionnaire(
        &self,
        user_id: i32,
        req: CreateQuestionnaireRequest,
    ) -> AppResult<QuestionnaireResponse> {
        let mut tx = self.db.begin().await?;

        // 创建问卷
        let questionnaire_id = sqlx::query!(
            r#"
            INSERT INTO questionnaires (title, description, is_public, creator_id)
            VALUES (?, ?, ?, ?)
            "#,
            req.title,
            req.description,
            req.is_public,
            user_id
        )
        .execute(&mut *tx)
        .await?
        .last_insert_id() as i32;

        // 创建问题和选项
        for (index, question) in req.questions.iter().enumerate() {
            let question_id = sqlx::query!(
                r#"
                INSERT INTO questions 
                (questionnaire_id, title, question_type, required, display_order)
                VALUES (?, ?, ?, ?, ?)
                "#,
                questionnaire_id,
                question.title,
                question.question_type,
                question.required,
                (index + 1) as i32
            )
            .execute(&mut *tx)
            .await?
            .last_insert_id() as i32;

            // 如果是单选或多选题，创建选项
            if question.question_type == "radio" || question.question_type == "checkbox" {
                for (opt_index, option_text) in question.options.iter().enumerate() {
                    sqlx::query!(
                        r#"
                        INSERT INTO question_options
                        (question_id, option_text, display_order)
                        VALUES (?, ?, ?)
                        "#,
                        question_id,
                        option_text,
                        (opt_index + 1) as i32
                    )
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        tx.commit().await?;

        // 返回创建的问卷
        self.get_questionnaire(questionnaire_id).await
    }

    // 更新问卷
    pub async fn update_questionnaire(
        &self,
        user_id: i32,
        questionnaire_id: i32,
        req: CreateQuestionnaireRequest,
    ) -> AppResult<QuestionnaireResponse> {
        // 先检查问卷是否存在且属于该用户
        let questionnaire = sqlx::query!(
            "SELECT creator_id FROM questionnaires WHERE id = ?",
            questionnaire_id
        )
        .fetch_optional(&*self.db)
        .await?
        .ok_or_else(|| AppError::NotFoundError(format!("问卷ID {} 不存在", questionnaire_id)))?;

        if questionnaire.creator_id != user_id {
            return Err(AppError::PermissionError("你无权修改此问卷".to_string()));
        }

        let mut tx = self.db.begin().await?;

        // 更新问卷基本信息
        sqlx::query!(
            r#"
            UPDATE questionnaires
            SET title = ?, description = ?, is_public = ?
            WHERE id = ?
            "#,
            req.title,
            req.description,
            req.is_public,
            questionnaire_id
        )
        .execute(&mut *tx)
        .await?;

        // 删除旧的问题和选项
        sqlx::query!("DELETE FROM questions WHERE questionnaire_id = ?", questionnaire_id)
            .execute(&mut *tx)
            .await?;

        // 创建新的问题和选项
        for (index, question) in req.questions.iter().enumerate() {
            let question_id = sqlx::query!(
                r#"
                INSERT INTO questions 
                (questionnaire_id, title, question_type, required, display_order)
                VALUES (?, ?, ?, ?, ?)
                "#,
                questionnaire_id,
                question.title,
                question.question_type,
                question.required,
                (index + 1) as i32
            )
            .execute(&mut *tx)
            .await?
            .last_insert_id() as i32;

            // 如果是单选或多选题，创建选项
            if question.question_type == "radio" || question.question_type == "checkbox" {
                for (opt_index, option_text) in question.options.iter().enumerate() {
                    sqlx::query!(
                        r#"
                        INSERT INTO question_options
                        (question_id, option_text, display_order)
                        VALUES (?, ?, ?)
                        "#,
                        question_id,
                        option_text,
                        (opt_index + 1) as i32
                    )
                    .execute(&mut *tx)
                    .await?;
                }
            }
        }

        tx.commit().await?;

        // 返回更新后的问卷
        self.get_questionnaire(questionnaire_id).await
    }

    // 获取问卷详情
    pub async fn get_questionnaire(&self, questionnaire_id: i32) -> AppResult<QuestionnaireResponse> {
        // 获取问卷基本信息
        let questionnaire = sqlx::query!(
            r#"
            SELECT id, title, description, is_public, creator_id, 
                   created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                   updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM questionnaires
            WHERE id = ?
            "#,
            questionnaire_id
        )
        .fetch_optional(&*self.db)
        .await?
        .ok_or_else(|| AppError::NotFoundError(format!("问卷ID {} 不存在", questionnaire_id)))?;

        // 获取问题列表
        let title = questionnaire.title.clone();
        let description = questionnaire.description.clone().expect("问卷描述不应为空");
        let is_public = questionnaire.is_public.expect("is_public状态不应为空") != 0;
        let creator_id = questionnaire.creator_id;
        let created_at = questionnaire.created_at.expect("创建时间不应为空");
        let updated_at = questionnaire.updated_at.expect("更新时间不应为空");

        // 转换为问卷模型
        let questionnaire_model = Questionnaire {
            id: questionnaire.id,
            title: title.clone(),
            description: description.clone(),
            is_public,
            creator_id,
            created_at,
            updated_at,
        };

        // 获取问题列表
        let questions = self.get_questionnaire_questions(&questionnaire_model).await?;

        Ok(QuestionnaireResponse {
            id: questionnaire.id,
            title,
            description,
            is_public,
            creator_id,
            created_at,
            updated_at,
            questions,
        })
    }

    // 获取问卷的所有问题和选项
    async fn get_questionnaire_questions(
        &self,
        questionnaire: &Questionnaire,
    ) -> AppResult<Vec<QuestionResponse>> {
        let questions = sqlx::query!(
            r#"
            SELECT id, questionnaire_id, title, question_type, required, display_order, 
                   created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                   updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
            FROM questions
            WHERE questionnaire_id = ?
            ORDER BY display_order
            "#,
            questionnaire.id
        )
        .fetch_all(&*self.db)
        .await?;

        let mut result = Vec::new();

        for question_record in questions {
            let mut options = Vec::new();
            
            // 转换为问题模型
            let question = Question {
                id: question_record.id,
                questionnaire_id: question_record.questionnaire_id,
                title: question_record.title,
                question_type: question_record.question_type.clone(),
                required: question_record.required.expect("必填标志不应为空") != 0,
                display_order: question_record.display_order,
                created_at: question_record.created_at.expect("创建时间不应为空"),
                updated_at: question_record.updated_at.expect("更新时间不应为空"),
            };

            // 如果是单选或多选题，获取选项
            if question.question_type == "radio" || question.question_type == "checkbox" {
                let question_options = sqlx::query!(
                    r#"
                    SELECT id, question_id, option_text, display_order,
                           created_at as "created_at: chrono::DateTime<chrono::Utc>", 
                           updated_at as "updated_at: chrono::DateTime<chrono::Utc>"
                    FROM question_options
                    WHERE question_id = ?
                    ORDER BY display_order
                    "#,
                    question.id
                )
                .fetch_all(&*self.db)
                .await?;

                for option_record in question_options {
                    options.push(option_record.option_text);
                }
            }

            result.push(QuestionResponse {
                id: question.id,
                title: question.title,
                question_type: question.question_type,
                required: question.required,
                options,
            });
        }

        Ok(result)
    }

    // 获取用户的问卷列表
    pub async fn get_user_questionnaires(
        &self,
        user_id: i32,
        page: i64,
        page_size: i64,
    ) -> AppResult<QuestionnaireListResponse> {
        // 获取总数
        let total = sqlx::query!(
            "SELECT COUNT(*) as count FROM questionnaires WHERE creator_id = ?",
            user_id
        )
        .fetch_one(&*self.db)
        .await?
        .count;

        // 获取问卷列表
        let items = sqlx::query!(
            r#"
            SELECT 
                q.id, 
                q.title, 
                q.description, 
                u.username as creator,
                q.created_at as "created_at: chrono::DateTime<chrono::Utc>",
                (SELECT COUNT(*) FROM questionnaire_responses WHERE questionnaire_id = q.id) as response_count
            FROM questionnaires q
            JOIN users u ON q.creator_id = u.id
            WHERE q.creator_id = ?
            ORDER BY q.created_at DESC
            LIMIT ? OFFSET ?
            "#,
            user_id,
            page_size,
            (page - 1) * page_size
        )
        .fetch_all(&*self.db)
        .await?;

        let items = items
            .into_iter()
            .map(|row| QuestionnaireListItem {
                id: row.id.expect("问卷ID不应为空"),
                title: row.title.expect("问卷标题不应为空"),
                description: row.description.expect("问卷描述不应为空"),
                creator: row.creator,
                created_at: row.created_at.expect("创建时间不应为空"),
                response_count: row.response_count.unwrap_or(0) as i32,
            })
            .collect();

        Ok(QuestionnaireListResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    // 获取公开问卷列表
    pub async fn get_public_questionnaires(
        &self,
        search: Option<String>,
        page: i64,
        page_size: i64,
    ) -> AppResult<QuestionnaireListResponse> {
        #[derive(Debug)]
        struct QueryResult {
            id: Option<i32>,
            title: Option<String>,
            description: Option<String>,
            creator: String,
            created_at: Option<chrono::DateTime<chrono::Utc>>,
            response_count: Option<i64>,
        }

        // 获取问卷列表和总数
        let total: i64;
        let raw_items: Vec<QueryResult>;

        if let Some(search_text) = &search {
            let search_pattern = format!("%{}%", search_text);
            
            // 获取带搜索条件的问卷列表
            let query_result = sqlx::query!(
                r#"
                SELECT 
                    q.id, 
                    q.title, 
                    q.description, 
                    u.username as creator,
                    q.created_at as "created_at: chrono::DateTime<chrono::Utc>",
                    (SELECT COUNT(*) FROM questionnaire_responses WHERE questionnaire_id = q.id) as response_count
                FROM questionnaires q
                JOIN users u ON q.creator_id = u.id
                WHERE q.is_public = 1
                AND (q.title LIKE ? OR q.description LIKE ?)
                ORDER BY q.created_at DESC
                LIMIT ? OFFSET ?
                "#,
                search_pattern,
                search_pattern,
                page_size,
                (page - 1) * page_size
            )
            .fetch_all(&*self.db)
            .await?;

            // 转换查询结果
            raw_items = query_result.into_iter()
                .map(|row| QueryResult {
                    id: row.id,
                    title: row.title,
                    description: row.description,
                    creator: row.creator,
                    created_at: row.created_at,
                    response_count: row.response_count,
                })
                .collect();

            // 获取带搜索条件的总数
            total = sqlx::query!(
                "
                SELECT COUNT(*) as count 
                FROM questionnaires q
                WHERE q.is_public = 1
                AND (q.title LIKE ? OR q.description LIKE ?)
                ",
                search_pattern,
                search_pattern
            )
            .fetch_one(&*self.db)
            .await?
            .count;
        } else {
            // 获取不带搜索条件的问卷列表
            let query_result = sqlx::query!(
                r#"
                SELECT 
                    q.id, 
                    q.title, 
                    q.description, 
                    u.username as creator,
                    q.created_at as "created_at: chrono::DateTime<chrono::Utc>",
                    (SELECT COUNT(*) FROM questionnaire_responses WHERE questionnaire_id = q.id) as response_count
                FROM questionnaires q
                JOIN users u ON q.creator_id = u.id
                WHERE q.is_public = 1
                ORDER BY q.created_at DESC
                LIMIT ? OFFSET ?
                "#,
                page_size,
                (page - 1) * page_size
            )
            .fetch_all(&*self.db)
            .await?;

            // 转换查询结果
            raw_items = query_result.into_iter()
                .map(|row| QueryResult {
                    id: row.id,
                    title: row.title,
                    description: row.description,
                    creator: row.creator,
                    created_at: row.created_at,
                    response_count: row.response_count,
                })
                .collect();

            // 获取不带搜索条件的总数
            total = sqlx::query!(
                "
                SELECT COUNT(*) as count 
                FROM questionnaires q
                WHERE q.is_public = 1
                "
            )
            .fetch_one(&*self.db)
            .await?
            .count;
        }

        // 转换为列表项
        let items = raw_items
            .into_iter()
            .map(|row| QuestionnaireListItem {
                id: row.id.expect("问卷ID不应为空"),
                title: row.title.expect("问卷标题不应为空"),
                description: row.description.expect("问卷描述不应为空"),
                creator: row.creator,
                created_at: row.created_at.expect("创建时间不应为空"),
                response_count: row.response_count.unwrap_or(0) as i32,
            })
            .collect();

        Ok(QuestionnaireListResponse {
            items,
            total,
            page,
            page_size,
        })
    }

    // 删除问卷
    pub async fn delete_questionnaire(&self, user_id: i32, questionnaire_id: i32) -> AppResult<()> {
        // 先检查问卷是否存在且属于该用户
        let questionnaire = sqlx::query!(
            "SELECT creator_id FROM questionnaires WHERE id = ?",
            questionnaire_id
        )
        .fetch_optional(&*self.db)
        .await?
        .ok_or_else(|| AppError::NotFoundError(format!("问卷ID {} 不存在", questionnaire_id)))?;

        if questionnaire.creator_id != user_id {
            return Err(AppError::PermissionError("你无权删除此问卷".to_string()));
        }

        // 开始事务
        let mut tx = self.db.begin().await?;

        // 删除问卷及相关数据
        Self::delete_questionnaire_transaction(&mut tx, questionnaire_id).await?;

        // 提交事务
        tx.commit().await?;

        Ok(())
    }

    // 在事务中删除问卷及相关数据
    async fn delete_questionnaire_transaction(
        tx: &mut Transaction<'_, MySql>,
        questionnaire_id: i32,
    ) -> AppResult<()> {
        // 删除问卷的回答
        // 先删除问题回答的选项和文本
        sqlx::query!(
            r#"
            DELETE tr
            FROM text_responses tr
            JOIN question_responses qr ON tr.question_response_id = qr.id
            JOIN questionnaire_responses qnr ON qr.questionnaire_response_id = qnr.id
            WHERE qnr.questionnaire_id = ?
            "#,
            questionnaire_id
        )
        .execute(&mut **tx)
        .await?;

        sqlx::query!(
            r#"
            DELETE opt_resp
            FROM option_responses opt_resp
            JOIN question_responses qr ON opt_resp.question_response_id = qr.id
            JOIN questionnaire_responses qnr ON qr.questionnaire_response_id = qnr.id
            WHERE qnr.questionnaire_id = ?
            "#,
            questionnaire_id
        )
        .execute(&mut **tx)
        .await?;

        // 删除问题回答
        sqlx::query!(
            r#"
            DELETE qr
            FROM question_responses qr
            JOIN questionnaire_responses qnr ON qr.questionnaire_response_id = qnr.id
            WHERE qnr.questionnaire_id = ?
            "#,
            questionnaire_id
        )
        .execute(&mut **tx)
        .await?;

        // 删除问卷回答
        sqlx::query!(
            "DELETE FROM questionnaire_responses WHERE questionnaire_id = ?",
            questionnaire_id
        )
        .execute(&mut **tx)
        .await?;

        // 删除问题选项
        sqlx::query!(
            r#"
            DELETE qo
            FROM question_options qo
            JOIN questions q ON qo.question_id = q.id
            WHERE q.questionnaire_id = ?
            "#,
            questionnaire_id
        )
        .execute(&mut **tx)
        .await?;

        // 删除问题
        sqlx::query!(
            "DELETE FROM questions WHERE questionnaire_id = ?",
            questionnaire_id
        )
        .execute(&mut **tx)
        .await?;

        // 删除问卷
        sqlx::query!(
            "DELETE FROM questionnaires WHERE id = ?",
            questionnaire_id
        )
        .execute(&mut **tx)
        .await?;

        Ok(())
    }
} 