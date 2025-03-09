use std::str::FromStr;

use serde::{Deserialize, Serialize};

use crate::{AppError, AppState};

use super::{ChatFile, Message};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMessage {
    pub content: String,
    pub files: Vec<String>,
}

#[allow(dead_code)]
impl AppState {
    pub async fn create_message(
        &self,
        input: CreateMessage,
        chat_id: u64,
        user_id: u64,
    ) -> Result<i64, AppError> {
        let base_dir = &self.config.server.base_url;
        if input.content.is_empty() && input.files.is_empty() {
            return Err(AppError::CreateMessageError(
                "Content or files must be provided".to_string(),
            ));
        }
        for s in &input.files {
            let file = ChatFile::from_str(s)?;
            if !file.path(base_dir).exists() {
                return Err(AppError::CreateMessageError(format!(
                    "File {} does not exist",
                    s
                )));
            }
        }
        let message: Message = sqlx::query_as(
            r#"
            INSERT INTO messages (chat_id, sender_id, content, files)
            VALUES ($1, $2, $3, $4)
            RETURNING id, chat_id, sender_id, content, files, created_at
            "#,
        )
        .bind(chat_id as i64)
        .bind(user_id as i64)
        .bind(input.content)
        .bind(&input.files)
        .fetch_one(&self.pool)
        .await?;
        Ok(message.id)
    }
}
