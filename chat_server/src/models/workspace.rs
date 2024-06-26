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
            SET owner_id = $2
            WHERE id = $1 and (select ws_id from users where id = $1) = $2
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(self.id)
        .bind(owner_id as i64)
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
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    use crate::{
        models::{UserInput, Workspace},
        User,
    };

    #[tokio::test]
    async fn workspace_curd_should_work() -> Result<()> {
        let tdb = TestPg::new(
            "postgres://firstero:firstero@localhost:5432".to_string(),
            Path::new("../migrations"),
        );
        let pool = tdb.get_pool().await;
        let input = UserInput::new("test", "test@org.com", "ws_name", "password");
        let user = User::create(&input, &pool).await?;
        let ws0 = Workspace::find_by_name("ws_name", &pool).await?.unwrap();
        assert_eq!(ws0.name, "ws_name");
        assert_eq!(ws0.owner_id, user.id);
        assert_eq!(user.ws_id, ws0.id);

        let ws = Workspace::find_by_id(user.ws_id, &pool).await?.unwrap();
        assert_eq!(ws0, ws);

        Ok(())
    }
}
