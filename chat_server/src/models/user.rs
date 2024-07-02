use crate::{error::AppError, AppState, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::mem;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct UserInput {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct ChatUser {
    pub id: i64,
    pub fullname: String,
    pub email: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}
//
impl AppState {
    // find_by_email 方法
    pub async fn find_user_by_email(&self, email: &str) -> Result<Option<User>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(&self.pool)
        .await?;
        Ok(user)
    }

    /// create 方法
    /// TODO: use transaction to ensure workspace binding and user creation are atomic
    pub async fn create_user(&self, user: &UserInput) -> Result<User, AppError> {
        // check if user exists
        if self.find_user_by_email(&user.email).await?.is_some() {
            return Err(AppError::EmailAlreadyExists(user.email.to_string()));
        }
        // check if workspace exists
        let ws = match self.find_workspace_by_name(&user.workspace).await? {
            Some(ws) => ws,
            None => self.create_workspace(&user.workspace, 0).await?,
        };

        // 使用 argon2 生成密码哈希
        let password = hash_password(&user.password)?;
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (ws_id, fullname, email, password_hash)
            VALUES ($1, $2, $3, $4)
            RETURNING id, ws_id, fullname, email, created_at
            "#,
        )
        .bind(ws.id)
        .bind(&user.fullname)
        .bind(&user.email)
        .bind(password)
        .fetch_one(&self.pool)
        .await?;
        if ws.owner_id == 0 {
            self.update_workspace_owner(ws, user.id as _).await?;
        }
        Ok(user)
    }

    /// verify email and password
    pub async fn verify_user(&self, input: &SigninUser) -> Result<Option<User>, AppError> {
        // find user by email
        let user: Option<User> = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(&input.email)
        .fetch_optional(&self.pool)
        .await?;

        // verify password
        match user {
            Some(mut user) => {
                // take password_hash from user
                let password_hash = mem::take(&mut user.password_hash).unwrap_or_default();
                let is_valid = verify_password(&input.password, &password_hash)?;
                if is_valid {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub async fn find_chat_users_by_ids(&self, ids: &[i64]) -> Result<Vec<ChatUser>, AppError> {
        let users = sqlx::query_as(
            r#"
            SELECT id, fullname, email
            FROM users
            WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(&self.pool)
        .await?;
        Ok(users)
    }
}

impl User {
    pub async fn add_to_workspace(&self, ws_id: i64, pool: &PgPool) -> Result<Self, AppError> {
        let user = sqlx::query_as(
            r#"
            UPDATE users
            SET ws_id = $1
            WHERE id = $2
            Returning id, ws_id, fullname, email, created_at
            "#,
        )
        .bind(ws_id)
        .bind(self.id)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }
}

fn hash_password(password: &str) -> Result<String, AppError> {
    let salt = SaltString::generate(&mut OsRng);

    // Argon2 with default params (Argon2id v19)
    let argon2 = Argon2::default();

    // Hash password to PHC string ($argon2id$v=19$...)
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

fn verify_password(password: &str, password_hash: &str) -> Result<bool, AppError> {
    let argon2 = Argon2::default();
    let parsed_hash = PasswordHash::new(password_hash)?;

    // Verify password against PHC string
    let is_valid = argon2
        .verify_password(password.as_bytes(), &parsed_hash)
        .is_ok();
    Ok(is_valid)
}

#[cfg(test)]
impl User {
    pub fn new(id: i64, fullname: &str, email: &str) -> Self {
        Self {
            id,
            ws_id: 0,
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
impl UserInput {
    pub fn new(fullname: &str, email: &str, workspace: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            email: email.to_string(),
            workspace: workspace.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
impl SigninUser {
    pub fn new(email: &str, password: &str) -> Self {
        Self {
            email: email.to_string(),
            password: password.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    use anyhow::Result;

    #[test]
    fn test_hash_password() -> Result<()> {
        let password = "password";
        let password_hash = hash_password(password)?;
        assert!(verify_password(password, &password_hash)?);
        Ok(())
    }

    #[tokio::test]
    async fn create_user_should_work() -> Result<()> {
        // init test database
        let (_tdb, state) = AppState::new_for_test().await?;

        // init test data
        let name = "firstero";
        let email = "firsero@acme.org";
        let workspace = "acme";
        let password = "password";

        let user_input = UserInput::new(name, email, workspace, password);

        let user = state.create_user(&user_input).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, name);
        assert!(user.id > 0);

        let user = state.find_user_by_email(email).await?;
        assert!(user.is_some());

        let right_user = SigninUser::new(email, password);
        assert!(state.verify_user(&right_user).await?.is_some());
        let wrong_user = SigninUser::new(email, "wrong_password");
        assert!(state.verify_user(&wrong_user).await?.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_email_test() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let user = state.find_user_by_email("Alice@test.org").await?;
        assert!(user.is_some());

        let user = state.find_user_by_email("NonExist@test.org").await?;
        assert!(user.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> Result<()> {
        // init test database
        let (_tdb, state) = AppState::new_for_test().await?;
        // init test data
        let name = "Alice";
        let email = "Alice@test.org";
        let workspace = "acme";
        let password = "password";

        let user_input = UserInput::new(name, email, workspace, password);
        // create duplicate user
        let ret = state.create_user(&user_input).await;
        match ret {
            Err(AppError::EmailAlreadyExists(_)) => Ok(()),
            _ => panic!("Expecting EmailAlreadyExists error"),
        }
    }
}
