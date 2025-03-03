use crate::model::{ChatUser, Workspace};
use crate::AppError;
use sqlx::PgPool;

impl Workspace {
    pub async fn create(name: &str, user_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let row = sqlx::query_as(
            r#"
            INSERT INTO workspace(name,owner_id)
            VALUES($1,$2)
            RETURNING id,name,owner_id,created_at
            "#,
        )
        .bind(name)
        .bind(user_id as i64)
        .fetch_one(pool)
        .await?;

        Ok(row)
    }

    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            UPDATE workspace
            SET owner_id = $1
            WHERE id = $2 AND (SELECT ws_id FROM users WHERE id = $1) = $2
            RETURNING id,name,owner_id,created_at
            "#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;

        Ok(ws)
    }

    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id,name,owner_id,created_at
            FROM workspace
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn find_by_id(id: u64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id,name,owner_id,created_at
            FROM workspace
            WHERE id = $1
            "#,
        )
        .bind(id as i64)
        .fetch_optional(pool)
        .await?;

        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn fetch_all_chat_users(id: u64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1 ORDER BY id
            "#,
        )
        .bind(id as i64)
        .fetch_all(pool)
        .await?;

        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    use crate::model::CreateUser;
    use crate::User;
    use anyhow::Result;
    use sqlx_db_tester::TestPg;

    ///TODO: Add more tests
    #[tokio::test]
    async fn workspace_should_create_and_set_owner() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let ws = Workspace::create("test", 0, &pool)
            .await
            .expect("cannot create workspace");
        assert_eq!(ws.name, "test");

        let user = CreateUser::new(&ws.name, "wu", "wu@github.con", "123456");
        let user = User::create(&user, &pool)
            .await
            .expect("cannot create user");

        assert_eq!(user.ws_id, ws.id);

        let Some(ws) = Workspace::find_by_id(ws.id as u64, &pool)
            .await
            .expect("cannot find workspace")
        else {
            panic!("cannot find workspace")
        };
        assert_eq!(ws.owner_id, user.id);

        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_find_by_name() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let _ws = Workspace::create("test", 0, &pool)
            .await
            .expect("cannot create workspace");

        let ws = Workspace::find_by_name("test", &pool)
            .await
            .expect("cannot find workspace");
        assert_eq!(ws.expect("cannot find by name").name, "test");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_should_fetch_all_chat_users() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://postgres:postgres@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let ws = Workspace::create("test", 0, &pool)
            .await
            .expect("cannot create workspace");

        let input = CreateUser::new(&ws.name, "wu", "wu@github.con", "123456");
        let user1 = User::create(&input, &pool).await?;
        let input = CreateUser::new(&ws.name, "wang", "wang@github.con", "123456");
        let user2 = User::create(&input, &pool).await?;

        let users = Workspace::fetch_all_chat_users(ws.id as _, &pool)
            .await
            .expect("cannot fetch all chat users");
        assert_eq!(users.len(), 2);
        assert_eq!(users[0].id, user1.id);
        assert_eq!(users[1].id, user2.id);

        Ok(())
    }
}
