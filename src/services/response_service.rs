use std::sync::Arc;
use sqlx::{MySql, Pool};

use crate::models::error::{AppError, AppResult};
use crate::models::response::{
    AnswerDetail, OptionCount, QuestionStatistics, QuestionnaireStatistics,
    ResponseDetails, ResponseListItem, SubmitResponseRequest, SubmitResponseResponse,
};
use crate::config::Config;

pub struct ResponseService {
    db: Arc<Pool<MySql>>,
    config: Arc<Config>,
}

impl ResponseService {
    pub fn new(db: Arc<Pool<MySql>>, config: Arc<Config>) -> Self {
        Self { db, config }
    }

    // 提交问卷回答
    pub async fn submit_response(
        &self,
        user_id: Option<i32>,
        req: SubmitResponseRequest,
    ) -> AppResult<SubmitResponseResponse> {
        // 检查问卷是否存在
        let _questionnaire = sqlx::query!(
            "SELECT id, is_public FROM questionnaires WHERE id = ?",
            req.questionnaire_id
        )
        .fetch_optional(&*self.db)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(format!("问卷ID {} 不存在", req.questionnaire_id))
        })?;
        
        // 如果是已登录用户，检查是否已经提交过该问卷
        if let Some(uid) = user_id {
            let existing_response = sqlx::query!(
                r#"
                SELECT id FROM questionnaire_responses 
                WHERE questionnaire_id = ? AND respondent_id = ?
                "#,
                req.questionnaire_id,
                uid
            )
            .fetch_optional(&*self.db)
            .await?;
            
            // 如果已经提交过，返回错误
            if existing_response.is_some() {
                return Err(AppError::ValidationError(
                    "您已经提交过该问卷，不能重复提交".to_string()
                ));
            }
        }

        // 开始事务
        let mut tx = self.db.begin().await?;

        // 创建问卷回答记录
        let questionnaire_response_id = sqlx::query!(
            r#"
            INSERT INTO questionnaire_responses (questionnaire_id, respondent_id)
            VALUES (?, ?)
            "#,
            req.questionnaire_id,
            user_id
        )
        .execute(&mut *tx)
        .await?
        .last_insert_id() as i32;

        // 处理每个问题的回答
        for answer in &req.answers {
            // 创建问题回答记录
            let question_response_id = sqlx::query!(
                r#"
                INSERT INTO question_responses (questionnaire_response_id, question_id)
                VALUES (?, ?)
                "#,
                questionnaire_response_id,
                answer.question_id
            )
            .execute(&mut *tx)
            .await?
            .last_insert_id() as i32;

            // 根据问题类型保存回答内容
            match answer.answer_type.as_str() {
                "text" => {
                    if let Some(text_value) = &answer.text_value {
                        sqlx::query!(
                            r#"
                            INSERT INTO text_responses (question_response_id, text_value)
                            VALUES (?, ?)
                            "#,
                            question_response_id,
                            text_value
                        )
                        .execute(&mut *tx)
                        .await?;
                    }
                }
                "option" | "options" => {
                    // 处理选项回答
                    if let Some(option_ids) = &answer.option_ids {
                        for option_id in option_ids {
                            sqlx::query!(
                                r#"
                                INSERT INTO option_responses (question_response_id, option_id)
                                VALUES (?, ?)
                                "#,
                                question_response_id,
                                option_id
                            )
                            .execute(&mut *tx)
                            .await?;
                        }
                    } else if let Some(option_values) = &answer.option_values {
                        // 如果提交的是选项文本而非ID，需要查找对应的选项ID
                        for option_value in option_values {
                            // 查找选项ID
                            let option = sqlx::query!(
                                r#"
                                SELECT id FROM question_options 
                                WHERE question_id = ? AND option_text = ?
                                "#,
                                answer.question_id,
                                option_value
                            )
                            .fetch_optional(&mut *tx)
                            .await?;

                            if let Some(option_row) = option {
                                sqlx::query!(
                                    r#"
                                    INSERT INTO option_responses (question_response_id, option_id)
                                    VALUES (?, ?)
                                    "#,
                                    question_response_id,
                                    option_row.id
                                )
                                .execute(&mut *tx)
                                .await?;
                            }
                        }
                    }
                }
                _ => {
                    return Err(AppError::ValidationError(format!(
                        "不支持的回答类型: {}",
                        answer.answer_type
                    )));
                }
            }
        }

        // 提交事务
        tx.commit().await?;

        Ok(SubmitResponseResponse {
            id: questionnaire_response_id,
            questionnaire_id: req.questionnaire_id,
            success: true,
            created_at: chrono::Utc::now(),
        })
    }

    // 获取问卷的统计信息
    pub async fn get_questionnaire_statistics(
        &self,
        user_id: i32,
        questionnaire_id: i32,
    ) -> AppResult<QuestionnaireStatistics> {
        // 检查问卷是否存在且用户是否有权限查看
        let questionnaire = sqlx::query!(
            "SELECT id, title, creator_id FROM questionnaires WHERE id = ?",
            questionnaire_id
        )
        .fetch_optional(&*self.db)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(format!("问卷ID {} 不存在", questionnaire_id))
        })?;

        // 检查权限
        if questionnaire.creator_id != user_id {
            return Err(AppError::PermissionError(
                "你无权查看此问卷的统计信息".to_string(),
            ));
        }

        // 获取问卷回答总数
        let response_count = sqlx::query!(
            "SELECT COUNT(*) as count FROM questionnaire_responses WHERE questionnaire_id = ?",
            questionnaire_id
        )
        .fetch_one(&*self.db)
        .await?
        .count as i32;

        // 获取问卷的所有问题
        let questions = sqlx::query!(
            r#"
            SELECT id, title, question_type
            FROM questions
            WHERE questionnaire_id = ?
            ORDER BY display_order
            "#,
            questionnaire_id
        )
        .fetch_all(&*self.db)
        .await?;

        let mut question_stats = Vec::new();

        for question in questions {
            match question.question_type.as_str() {
                "text" => {
                    // 获取文本回答
                    let text_responses = sqlx::query!(
                        r#"
                        SELECT tr.text_value
                        FROM text_responses tr
                        JOIN question_responses qr ON tr.question_response_id = qr.id
                        JOIN questionnaire_responses qnr ON qr.questionnaire_response_id = qnr.id
                        WHERE qnr.questionnaire_id = ? AND qr.question_id = ?
                        "#,
                        questionnaire_id,
                        question.id
                    )
                    .fetch_all(&*self.db)
                    .await?
                    .into_iter()
                    .map(|row| row.text_value)
                    .collect();

                    question_stats.push(QuestionStatistics {
                        question_id: question.id,
                        title: question.title,
                        question_type: question.question_type,
                        text_responses: Some(text_responses),
                        option_counts: None,
                    });
                }
                "radio" | "checkbox" => {
                    // 获取选项及其选择次数
                    let options = sqlx::query!(
                        r#"
                        SELECT 
                            qo.id as option_id, 
                            qo.option_text,
                            COUNT(opt_resp.id) as count
                        FROM question_options qo
                        LEFT JOIN option_responses opt_resp ON qo.id = opt_resp.option_id
                        LEFT JOIN question_responses qr ON opt_resp.question_response_id = qr.id
                        WHERE qo.question_id = ?
                        GROUP BY qo.id, qo.option_text
                        ORDER BY qo.display_order
                        "#,
                        question.id
                    )
                    .fetch_all(&*self.db)
                    .await?;

                    let total_responses = options
                        .iter()
                        .map(|opt| opt.count as i32)
                        .sum::<i32>();

                    let option_counts = options
                        .into_iter()
                        .map(|opt| {
                            let count = opt.count as i32;
                            let percentage = if total_responses > 0 {
                                (count as f64 / total_responses as f64) * 100.0
                            } else {
                                0.0
                            };

                            OptionCount {
                                option_id: opt.option_id,
                                option_text: opt.option_text,
                                count,
                                percentage,
                            }
                        })
                        .collect();

                    question_stats.push(QuestionStatistics {
                        question_id: question.id,
                        title: question.title,
                        question_type: question.question_type,
                        text_responses: None,
                        option_counts: Some(option_counts),
                    });
                }
                _ => {} // 其他类型暂不处理
            }
        }

        Ok(QuestionnaireStatistics {
            questionnaire_id,
            title: questionnaire.title,
            response_count,
            questions: question_stats,
        })
    }

    // 获取问卷的回答列表
    pub async fn get_questionnaire_responses(
        &self,
        user_id: i32,
        questionnaire_id: i32,
        page: i64,
        page_size: i64,
    ) -> AppResult<Vec<ResponseListItem>> {
        // 检查问卷是否存在且用户是否有权限查看
        let questionnaire = sqlx::query!(
            "SELECT creator_id FROM questionnaires WHERE id = ?",
            questionnaire_id
        )
        .fetch_optional(&*self.db)
        .await?
        .ok_or_else(|| {
            AppError::NotFoundError(format!("问卷ID {} 不存在", questionnaire_id))
        })?;

        // 检查权限
        if questionnaire.creator_id != user_id {
            return Err(AppError::PermissionError(
                "你无权查看此问卷的回答".to_string(),
            ));
        }

        // 获取问卷回答列表
        let responses = sqlx::query!(
            r#"
            SELECT 
                qr.id, 
                qr.questionnaire_id,
                qr.created_at as "created_at: chrono::DateTime<chrono::Utc>",
                u.username as respondent
            FROM questionnaire_responses qr
            LEFT JOIN users u ON qr.respondent_id = u.id
            WHERE qr.questionnaire_id = ?
            ORDER BY qr.created_at DESC
            LIMIT ? OFFSET ?
            "#,
            questionnaire_id,
            page_size,
            (page - 1) * page_size
        )
        .fetch_all(&*self.db)
        .await?
        .into_iter()
        .map(|row| ResponseListItem {
            id: row.id,
            questionnaire_id: row.questionnaire_id,
            respondent: row.respondent,
            created_at: row.created_at.expect("创建时间不应为空"),
        })
        .collect();

        Ok(responses)
    }

    // 获取回答详情
    pub async fn get_response_detail(
        &self,
        user_id: i32,
        response_id: i32,
    ) -> AppResult<ResponseDetails> {
        // 获取回答基本信息
        let response = sqlx::query!(
            r#"
            SELECT 
                qr.id, 
                qr.questionnaire_id,
                qr.created_at as "created_at: chrono::DateTime<chrono::Utc>",
                q.title as questionnaire_title,
                q.creator_id,
                u.username as respondent
            FROM questionnaire_responses qr
            JOIN questionnaires q ON qr.questionnaire_id = q.id
            LEFT JOIN users u ON qr.respondent_id = u.id
            WHERE qr.id = ?
            "#,
            response_id
        )
        .fetch_optional(&*self.db)
        .await?
        .ok_or_else(|| AppError::NotFoundError(format!("回答ID {} 不存在", response_id)))?;

        // 检查权限
        if response.creator_id != user_id {
            return Err(AppError::PermissionError(
                "你无权查看此回答".to_string(),
            ));
        }

        // 获取回答详情
        let answers = sqlx::query!(
            r#"
            SELECT 
                q.id as question_id,
                q.title as question_title,
                q.question_type,
                tr.text_value,
                GROUP_CONCAT(qo.option_text SEPARATOR '||') as selected_options
            FROM question_responses qr
            JOIN questions q ON qr.question_id = q.id
            LEFT JOIN text_responses tr ON tr.question_response_id = qr.id
            LEFT JOIN option_responses orsp ON orsp.question_response_id = qr.id
            LEFT JOIN question_options qo ON orsp.option_id = qo.id
            WHERE qr.questionnaire_response_id = ?
            GROUP BY q.id, q.title, q.question_type, tr.text_value
            "#,
            response_id
        )
        .fetch_all(&*self.db)
        .await?
        .into_iter()
        .map(|row| {
            let selected_options = row
                .selected_options
                .map(|opts| {
                    opts.split("||")
                        .map(|s| s.to_string())
                        .filter(|s| !s.is_empty())
                        .collect()
                });

            AnswerDetail {
                question_id: row.question_id,
                question_title: row.question_title,
                question_type: row.question_type,
                text_value: row.text_value,
                selected_options,
            }
        })
        .collect();

        Ok(ResponseDetails {
            id: response.id,
            questionnaire_id: response.questionnaire_id,
            questionnaire_title: response.questionnaire_title,
            respondent: response.respondent,
            created_at: response.created_at.expect("创建时间不应为空"),
            answers,
        })
    }
} 