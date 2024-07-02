use crate::{error::AppError, AppState};

use super::{ChatUser, Workspace};

impl AppState {
    pub async fn create_workspace(&self, name: &str, owner_id: u64) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            r#"
            INSERT INTO workspaces (name, owner_id)
            VALUES ($1, $2)
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(name)
        .bind(owner_id as i64)
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }

    /// find_by_name 方法
    pub async fn find_workspace_by_name(&self, name: &str) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE name = $1
            "#,
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }

    /// find_by_id 方法
    #[allow(dead_code)]
    pub async fn find_workspace_by_id(&self, id: i64) -> Result<Option<Workspace>, AppError> {
        let ws = sqlx::query_as(
            r#"
            SELECT id, name, owner_id, created_at
            FROM workspaces
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;
        Ok(ws)
    }

    pub async fn update_workspace_owner(
        &self,
        ws: Workspace,
        owner_id: u64,
    ) -> Result<Workspace, AppError> {
        let ws = sqlx::query_as(
            r#"
            UPDATE workspaces
            SET owner_id = $1
            WHERE id = $2
            RETURNING id, name, owner_id, created_at
            "#,
        )
        .bind(owner_id as i64)
        .bind(ws.id)
        .fetch_one(&self.pool)
        .await?;
        Ok(ws)
    }

    pub async fn fetch_all_chat_users(&self, id: i64) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE ws_id = $1
            "#,
        )
        .bind(id)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::{models::UserInput, AppState};

    #[tokio::test]
    async fn workspace_find_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        // test fetch all chat users
        let users = state.fetch_all_chat_users(1).await?;
        assert_eq!(users.len(), 5);
        // test find by name
        let ws = state.find_workspace_by_name("acme").await?;
        assert_eq!(ws.unwrap().name, "acme");
        // test find by id
        let ws = state.find_workspace_by_id(1).await?;
        assert_eq!(ws.unwrap().name, "acme");
        Ok(())
    }

    #[tokio::test]
    async fn workspace_create_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        // 先有 workspace 再有 user
        let ws = state.create_workspace("ws_create", 1).await?;
        let input = UserInput::new(
            "test_for_ws_create",
            "test_for_ws_create@org",
            &ws.name,
            "password",
        );
        let user = state.create_user(&input).await?;

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
        let user = state.create_user(&input).await?;
        let ws = state
            .find_workspace_by_name("ws_create2")
            .await?
            .expect("workspace not found");
        assert_eq!(ws.name, "ws_create2");
        assert_eq!(user.ws_id, ws.id);
        assert_eq!(ws.owner_id, user.id);
        Ok(())
    }
}
