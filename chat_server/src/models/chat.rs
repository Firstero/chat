use serde::{Deserialize, Serialize};
use sqlx::PgPool;

use crate::error::AppError;

use super::{Chat, ChatType, ChatUser};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ChatInput {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

impl Chat {
    #[allow(dead_code)]
    pub async fn create(chat: &ChatInput, ws_id: u64, pool: &PgPool) -> Result<Chat, AppError> {
        // check if members length is greater than 1
        let len = chat.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "Members length must be greater than 1".to_string(),
            ));
        }
        // check if group chat has a name, when members length is greater than 8
        if len > 8 && chat.name.is_none() {
            return Err(AppError::CreateChatError(
                "Group chat with more than 8 members must have a name".to_string(),
            ));
        }
        // verify that all members exist
        let users = ChatUser::find_by_ids(&chat.members, pool).await?;
        if users.len() != len {
            return Err(AppError::CreateChatError(
                "Some members do not exist".to_string(),
            ));
        }

        let chat_type = match (&chat.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            _ => {
                if chat.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };

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
        .bind(chat_type)
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

    #[allow(dead_code)]
    pub async fn find_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, members, type, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;
        Ok(chat)
    }

    #[allow(dead_code)]
    pub async fn fetch_all(ws_id: u64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, members, type, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(pool)
        .await?;
        Ok(chats)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_utils::get_test_pool;

    use super::*;

    impl ChatInput {
        fn new(name: &str, members: &[i64], public: bool) -> Self {
            Self {
                name: if name.is_empty() {
                    None
                } else {
                    Some(name.to_string())
                },
                members: members.to_vec(),
                public,
            }
        }
    }

    #[tokio::test]
    async fn create_single_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws_id = 1;
        let chat = ChatInput::new("", &[1, 2], false);
        let chat = Chat::create(&chat, ws_id, &pool)
            .await
            .expect("create chat error");
        assert_eq!(chat.ws_id, ws_id as i64);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws_id = 1;
        let chat = ChatInput::new("group chat", &[1, 2, 3, 4], true);
        let chat = Chat::create(&chat, ws_id, &pool)
            .await
            .expect("create chat error");
        assert_eq!(chat.ws_id, ws_id as i64);
        assert_eq!(chat.members.len(), 4);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
    }

    #[tokio::test]
    async fn create_private_named_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let ws_id = 1;
        let chat = ChatInput::new("group chat", &[1, 2, 3, 4, 5], false);
        let chat = Chat::create(&chat, ws_id, &pool)
            .await
            .expect("create chat error");
        assert_eq!(chat.ws_id, ws_id as i64);
        assert_eq!(chat.members.len(), 5);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);
    }

    #[tokio::test]
    async fn find_by_id_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let chat = Chat::find_by_id(1, &pool).await.unwrap().unwrap();
        assert_eq!(1, chat.id);
        assert_eq!(1, chat.ws_id);
        assert_eq!(2, chat.members.len());
        assert_eq!(Some("Private Channel".to_string()), chat.name);
        assert_eq!(ChatType::PrivateChannel, chat.r#type);
    }

    #[tokio::test]
    async fn chat_fetch_all_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let chats = Chat::fetch_all(1, &pool).await.unwrap();
        assert_eq!(4, chats.len());
    }
}
