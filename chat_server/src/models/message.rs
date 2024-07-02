use crate::{error::AppError, AppState, ChatFile, Message};
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MessageInput {
    pub content: String,
    pub files: Vec<String>,
}

impl AppState {
    #[allow(dead_code)]
    pub async fn create_message(
        &self,
        input: MessageInput,
        chat_id: u64,
        user_id: u64,
    ) -> Result<Message, AppError> {
        // verify content is not empty
        if input.content.is_empty() {
            return Err(AppError::CreateMessageError("Content is empty".to_string()));
        }
        // verify files exist
        for s in &input.files {
            let s = ChatFile::from_str(s)?;
            let path = s.path(&self.config.server.base_dir);
            if !path.exists() {
                return Err(AppError::CreateMessageError(format!(
                    "File not exist: {:?}",
                    path
                )));
            }
        }
        // create message
        let message = sqlx::query_as(
            r#"
            INSERT INTO messages (chat_id, user_id, content, files)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, user_id, content, files, created_at
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
}
