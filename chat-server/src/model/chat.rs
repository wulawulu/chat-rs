use crate::model::{Chat, ChatType, ChatUser};
use crate::AppError;
use serde::{Deserialize, Serialize};
use sqlx::PgPool;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateChat {
    pub name: Option<String>,
    pub members: Vec<i64>,
    pub public: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateChat {
    pub name: Option<String>,
    pub members: Option<Vec<i64>>,
    pub chat_type: Option<ChatType>,
}

#[allow(dead_code)]
impl Chat {
    pub async fn create(input: CreateChat, ws_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let len = input.members.len();
        if len < 2 {
            return Err(AppError::CreateChatError(
                "Chat must have at least 2 members".to_string(),
            ));
        }

        if len > 8 && input.name.is_none() {
            return Err(AppError::CreateChatError(
                "Group chat with more than 8 members must have a name".to_string(),
            ));
        }

        let user = ChatUser::fetch_by_ids(&input.members, pool).await?;
        if user.len() != len {
            return Err(AppError::CreateChatError(
                "Some members do not exist".to_string(),
            ));
        }

        let chat_type = match (&input.name, len) {
            (None, 2) => ChatType::Single,
            (None, _) => ChatType::Group,
            (Some(_), _) => {
                if input.public {
                    ChatType::PublicChannel
                } else {
                    ChatType::PrivateChannel
                }
            }
        };

        let chat = sqlx::query_as(
            r#"
            INSERT INTO chats (ws_id, name, type, members)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, name, type, members,created_at
                "#,
        )
        .bind(ws_id as i64)
        .bind(input.name)
        .bind(chat_type)
        .bind(&input.members)
        .fetch_one(pool)
        .await?;
        Ok(chat)
    }

    pub async fn fetch_all(ws_id: u64, pool: &PgPool) -> Result<Vec<Self>, AppError> {
        let chats = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE ws_id = $1
                "#,
        )
        .bind(ws_id as i64)
        .fetch_all(pool)
        .await?;

        Ok(chats)
    }

    pub async fn get_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let chat = sqlx::query_as(
            r#"
            SELECT id, ws_id, name, type, members, created_at
            FROM chats
            WHERE id = $1
                "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(chat)
    }

    pub async fn update_by_id(id: u64, input: UpdateChat, pool: &PgPool) -> Result<Self, AppError> {
        let chat = Chat::get_by_id(id, pool).await?;
        let chat = match chat {
            None => return Err(AppError::NotFound(format!("chat id {}", id))),
            Some(chat) => chat,
        };

        let mut name = chat.name;
        if let Some(new_name) = input.name {
            name = Some(new_name);
        }

        let mut members = chat.members;
        if let Some(new_members) = input.members {
            if chat.r#type.eq(&ChatType::Single) {
                return Err(AppError::UpdateChatError(
                    "Cannot update members of a single chat".to_string(),
                ));
            }
            let user = ChatUser::fetch_by_ids(&new_members, pool).await?;
            if user.len() != new_members.len() {
                return Err(AppError::UpdateChatError(
                    "Some members do not exist".to_string(),
                ));
            }
            members = new_members;
        }

        let mut r#type = chat.r#type;
        if let Some(new_type) = input.chat_type {
            if r#type == ChatType::Single || r#type == ChatType::Group {
                return Err(AppError::UpdateChatError(
                    "Cannot update type of a single or group chat".to_string(),
                ));
            }
            r#type = new_type;
        }

        let chat = sqlx::query_as(
            r#"
            UPDATE chats
            SET name = $1, members = $2, type = $3
            WHERE id = $4
            RETURNING id, ws_id, name, type, members, created_at
                "#,
        )
        .bind(name)
        .bind(&members)
        .bind(r#type)
        .bind(id as i64)
        .fetch_one(pool)
        .await?;

        Ok(chat)
    }

    pub async fn delete_by_id(id: u64, pool: &PgPool) -> Result<(), AppError> {
        sqlx::query(
            r#"
            DELETE FROM chats
            WHERE id = $1
                "#,
        )
        .bind(id as i64)
        .execute(pool)
        .await?;

        Ok(())
    }
}

#[cfg(test)]
impl CreateChat {
    pub fn new(name: &str, members: &[i64], public: bool) -> Self {
        let name = if name.is_empty() {
            None
        } else {
            Some(name.to_string())
        };
        Self {
            name,
            members: members.to_vec(),
            public,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_util::get_test_pool;

    #[tokio::test]
    async fn create_single_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let input = CreateChat::new("", &[1, 2], false);
        let chat = Chat::create(input, 1, &pool)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 2);
        assert_eq!(chat.r#type, ChatType::Single);
    }

    #[tokio::test]
    async fn create_public_named_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let input = CreateChat::new("test", &[1, 2, 3], true);
        let chat = Chat::create(input, 1, &pool)
            .await
            .expect("create chat failed");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
    }

    #[tokio::test]
    async fn chat_get_by_id_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let chat = Chat::get_by_id(1, &pool)
            .await
            .expect("get chat by id failed")
            .unwrap();
        assert_eq!(chat.id, 1);
        assert_eq!(chat.name.expect("chat name"), "general");
        assert_eq!(chat.ws_id, 1);
        assert_eq!(chat.members.len(), 5);
    }

    #[tokio::test]
    async fn chat_fetch_all_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let chats = Chat::fetch_all(1, &pool)
            .await
            .expect("fetch all chats failed");

        assert_eq!(chats.len(), 4);
    }

    #[tokio::test]
    async fn delete_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        Chat::delete_by_id(1, &pool)
            .await
            .expect("delete chat failed");
        let chat = Chat::get_by_id(1, &pool)
            .await
            .expect("get chat by id failed");
        assert!(chat.is_none());
    }

    #[tokio::test]
    async fn update_chat_should_work() {
        let (_tdb, pool) = get_test_pool(None).await;
        let input = UpdateChat {
            name: Some("new name".to_string()),
            members: Some(vec![1, 2, 3]),
            chat_type: None,
        };
        let chat = Chat::update_by_id(1, input, &pool)
            .await
            .expect("update chat failed");
        assert_eq!(chat.name.expect("chat name"), "new name");
        assert_eq!(chat.members.len(), 3);
        assert_eq!(chat.r#type, ChatType::PublicChannel);
    }
}
