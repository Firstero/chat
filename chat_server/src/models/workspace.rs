use sqlx::PgPool;

use crate::error::AppError;

use super::{ChatUser, Workspace};

impl Workspace {
    pub async fn create(name: &str, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(name)
        .bind(owner_id as i64)
        .fetch_one(pool)
        .await?;
        Ok(ws)
    }

    /// find_by_name 方法
    pub async fn find_by_name(name: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(pool)
        .await?;
        Ok(ws)
    }

    /// find_by_id 方法
    #[allow(dead_code)]
    pub async fn find_by_id(id: i64, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(pool)
        .await?;
        Ok(ws)
    }

    pub async fn update_owner(&self, owner_id: u64, pool: &PgPool) -> Result<Self, AppError> {
        let ws = sqlx::query_as(
            r#"
            UPDATE workspaces
            SET owner_id = $1
            WHERE id = $2
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(owner_id as i64)
        .bind(self.id)
        .fetch_one(pool)
        .await?;
        Ok(ws)
    }

    #[allow(dead_code)]
    pub async fn fetch_all_chat_users(id: i64, pool: &PgPool) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1
            "#,
        )
        .bind(id)
        .fetch_all(pool)
        .await?;
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{
        models::{UserInput, Workspace},
        test_utils::get_test_pool,
        User,
    };

    #[tokio::test]
    async fn workspace_find_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        // test fetch all chat users
        let users = Workspace::fetch_all_chat_users(1, &pool).await?;
        assert_eq!(users.len(), 5);
        // test find by name
        let ws = Workspace::find_by_name("acme", &pool).await?;
        assert_eq!(ws.unwrap().name, "acme");
        // test find by id
        let ws = Workspace::find_by_id(1, &pool).await?;
        assert_eq!(ws.unwrap().name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_create_should_work() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;
        // 先有 workspace 再有 user
        let ws = Workspace::create("ws_create", 1, &pool).await?;
        let input = UserInput::new(
            "test_for_ws_create",
            "test_for_ws_create@org",
            &ws.name,
            "password",
        );
        let user = User::create(&input, &pool).await?;

        assert_eq!(ws.name, "ws_create");
        assert_eq!(user.ws_id, ws.id);
        assert_eq!(ws.owner_id, 1);
        // 直接创建 user，并创建 workspace
        let input = UserInput::new(
            "test_for_ws_create2",
            "test_for_ws_create2@org",
            "ws_create2",
            "password",
        );
        let user = User::create(&input, &pool).await?;
        let ws = Workspace::find_by_name("ws_create2", &pool)
            .await?
            .expect("workspace not found");
        assert_eq!(ws.name, "ws_create2");
        assert_eq!(user.ws_id, ws.id);
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }
}
