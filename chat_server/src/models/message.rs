use crate::{error::AppError, AppState, ChatFile, Message};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListMessage {
    pub last_id: Option<u64>,
    pub limit: u64,
}

impl AppState {
    #[allow(dead_code)]
    pub async fn create_message(
        &self,
        input: CreateMessage,
        chat_id: u64,
        user_id: u64,
    ) -> Result<Message, AppError> {
        // verify content is not empty
        if input.content.is_empty() {
            return Err(AppError::CreateMessageError("Content is empty".to_string()));
        }
        // verify files exist
        for s in &input.files {
            let chatfile = ChatFile::from_str(s)?;
            let path = chatfile.path(&self.config.server.base_dir);
            if !path.exists() {
                return Err(AppError::CreateMessageError(format!("File not exist: {s}")));
            }
        }

        // create message
        let message = sqlx::query_as(
            r#"
            INSERT INTO messages (chat_id, sender_id, content, files)
            VALUES ($1, $2, $3, $4)
            RETURNING id, chat_id, sender_id, content, files, created_at
            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .bind(input.content)
        .bind(input.files)
        .fetch_one(&self.pool)
        .await?;
        Ok(message)
    }

    pub async fn list_messages(
        &self,
        chat_id: u64,
        input: &ListMessage,
    ) -> Result<Vec<Message>, AppError> {
        let last_id = input.last_id.unwrap_or(i64::MAX as u64);
        let messages = sqlx::query_as(
            r#"
            SELECT id, chat_id, sender_id, content, files, created_at
            FROM messages
            WHERE chat_id = $1 AND id < $2
            ORDER BY id DESC
            LIMIT $3
            "#,
        )
        .bind(chat_id as i64)
        .bind(last_id as i64)
        .bind(input.limit as i64)
        .fetch_all(&self.pool)
        .await?;
        Ok(messages)
    }
}

#[cfg(test)]
mod tests {

    use super::*;
    use crate::AppState;
    use anyhow::Result;

    #[tokio::test]
    async fn create_message_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let input = CreateMessage {
            content: "Hello".to_string(),
            files: vec![],
        };

        let message = state
            .create_message(input, 1, 1)
            .await
            .expect("create message error");
        assert_eq!(message.content, "Hello");
        assert_eq!(message.files.len(), 0);

        // invalid files should return error
        let input = CreateMessage {
            content: "Hello".to_string(),
            files: vec!["/files/1/12345/67890/abcdef.jpg".to_string()],
        };
        let err = state.create_message(input, 1, 1).await.unwrap_err();
        assert_eq!(
            err.to_string(),
            "create message error: File not exist: /files/1/12345/67890/abcdef.jpg"
        );

        // valid files should work
        let fileurl = upload_dummy_file(&state)?;
        let input = CreateMessage {
            content: "Hello".to_string(),
            files: vec![fileurl],
        };
        let message = state
            .create_message(input, 1, 1)
            .await
            .expect("create message error");
        assert_eq!(message.content, "Hello");
        assert_eq!(message.files.len(), 1);
        Ok(())
    }

    #[tokio::test]
    async fn list_messages_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let chat_id = 1;

        let input = ListMessage {
            last_id: None,
            limit: 6,
        };
        let messages = state.list_messages(chat_id, &input).await?;
        assert_eq!(messages.len(), 6);
        let last_id = messages.last().unwrap().id;
        let input = ListMessage {
            last_id: Some(last_id as _),
            limit: 6,
        };

        let messages = state.list_messages(chat_id, &input).await?;
        assert_eq!(messages.len(), 4);
        Ok(())
    }

    fn upload_dummy_file(state: &AppState) -> Result<String> {
        let data = b"hello world";
        let file = ChatFile::new(1, "test.txt", data);
        let path = file.path(&state.config.server.base_dir);
        std::fs::create_dir_all(path.parent().unwrap())?;
        std::fs::write(&path, data)?;
        Ok(file.url())
    }
}
