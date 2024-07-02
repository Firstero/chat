use serde::{Deserialize, Serialize};

use crate::{error::AppError, AppState};

use super::{Chat, ChatType};

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
pub struct ChatInput {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

impl AppState {
    pub async fn create_chat(&self, chat: &ChatInput, ws_id: u64) -> Result<Chat, AppError> {
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
        let users = self.find_chat_users_by_ids(&chat.members).await?;
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
        .fetch_one(&self.pool)
        .await?;
        Ok(chat)
    }

    #[allow(dead_code)]
    pub async fn find_chat_by_name(&self, name: &str) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, members, type, created_at
            FROM chats
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(chat)
    }

    pub async fn find_chat_by_id(&self, id: u64) -> Result<Option<Chat>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, members, type, created_at
            FROM chats
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(&self.pool)
        .await?;
        Ok(chat)
    }

    pub async fn fetch_all_chat(&self, ws_id: u64) -> Result<Vec<Chat>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, members, type, created_at
            FROM chats
            WHERE ws_id = $1
            "#,
        )
        .bind(ws_id as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(chats)
    }
}

#[cfg(test)]
mod tests {
    use crate::AppState;

    use super::*;
    use anyhow::Result;

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
    async fn create_single_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws_id = 1;
        let chat = ChatInput::new("", &[1, 2], false);
        let chat = state
            .create_chat(&chat, ws_id)
            .await
            .expect("create chat error");
        assert_eq!(chat.ws_id, ws_id as i64);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);
        Ok(())
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws_id = 1;
        let chat = ChatInput::new("group chat", &[1, 2, 3, 4], true);
        let chat = state
            .create_chat(&chat, ws_id)
            .await
            .expect("create chat error");
        assert_eq!(chat.ws_id, ws_id as i64);
        assert_eq!(chat.members.len(), 4);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
        Ok(())
    }

    #[tokio::test]
    async fn create_private_named_chat_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let ws_id = 1;
        let chat = ChatInput::new("group chat", &[1, 2, 3, 4, 5], false);
        let chat = state
            .create_chat(&chat, ws_id)
            .await
            .expect("create chat error");
        assert_eq!(chat.ws_id, ws_id as i64);
        assert_eq!(chat.members.len(), 5);
        assert_eq!(chat.r#type, ChatType::PrivateChannel);
        Ok(())
    }

    #[tokio::test]
    async fn find_by_id_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat = state.find_chat_by_id(1).await?.unwrap();
        assert_eq!(1, chat.id);
        assert_eq!(1, chat.ws_id);
        assert_eq!(2, chat.members.len());
        assert_eq!(Some("Private Channel".to_string()), chat.name);
        assert_eq!(ChatType::PrivateChannel, chat.r#type);
        Ok(())
    }

    #[tokio::test]
    async fn chat_fetch_all_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chats = state.fetch_all_chat(1).await?;
        assert_eq!(4, chats.len());
        Ok(())
    }
}
