use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;

use super::{Chat, ChatType};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ChatInput {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub r#type: ChatType,
}

impl Chat {
    #[allow(dead_code)]
    pub async fn create(chat: &ChatInput, ws_id: u64, pool: &PgPool) -> Result<Chat, AppError> {
        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, members, type)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, members, type, created_at
            "#,
        )
        .bind(ws_id as i64)
        .bind(&chat.name)
        .bind(&chat.members)
        .bind(&chat.r#type)
        .fetch_one(pool)
        .await?;
        Ok(chat)
    }

    #[allow(dead_code)]
    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, members, type, created_at
            FROM chats
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;
        Ok(chat)
    }
}
