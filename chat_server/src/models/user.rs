use crate::{error::AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::{FromRow, PgPool};
use std::mem;

use super::Workspace;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct UserInput {
    pub fullname: String,
    pub email: String,
    pub workspace: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct SigninUser {
    pub email: String,
    pub password: String,
}
//
impl User {
    // find_by_email 方法
    pub async fn find_by_email(email: &str, pool: &PgPool) -> Result<Option<Self>, AppError> {
        let user = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, created_at FROM users WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pool)
        .await?;

        Ok(user)
    }

    /// create 方法
    /// TODO: use transaction to ensure workspace binding and user creation are atomic
    pub async fn create(user: &UserInput, pool: &PgPool) -> Result<User, AppError> {
        // check if user exists
        if User::find_by_email(&user.email, pool).await?.is_some() {
            return Err(AppError::EmailAlreadyExists(user.email.to_string()));
        }
        // check if workspace exists
        let ws = match Workspace::find_by_name(&user.workspace, pool).await? {
            Some(ws) => ws,
            None => Workspace::create(&user.workspace, 0, pool).await?,
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
        .fetch_one(pool)
        .await?;
        ws.trigger_update_owner(user.id as _, pool).await?;
        Ok(user)
    }

    /// add user to workspace
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

    /// verify email and password
    pub async fn verify(input: &SigninUser, pool: &PgPool) -> Result<Option<Self>, AppError> {
        // find user by email
        let user: Option<User> = sqlx::query_as(
            "SELECT id, ws_id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
        )
        .bind(&input.email)
        .fetch_optional(pool)
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

    use crate::test_utils::get_test_pool;

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
        let (_tdb, pool) = get_test_pool(None).await;

        // init test data
        let name = "firstero";
        let email = "firsero@acme.org";
        let workspace = "acme";
        let password = "password";

        let user_input = UserInput::new(name, email, workspace, password);

        let user = User::create(&user_input, &pool).await?;
        assert_eq!(user.email, email);
        assert_eq!(user.fullname, name);
        assert!(user.id > 0);

        let user = User::find_by_email(email, &pool).await?;
        assert!(user.is_some());

        let right_user = SigninUser::new(email, password);
        assert!(User::verify(&right_user, &pool).await?.is_some());
        let wrong_user = SigninUser::new(email, "wrong_password");
        assert!(User::verify(&wrong_user, &pool).await?.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn find_user_by_email_test() -> Result<()> {
        let (_tdb, pool) = get_test_pool(None).await;

        let user = User::find_by_email("Alice@test.org", &pool).await?;
        assert!(user.is_some());

        let user = User::find_by_email("NonExist@test.org", &pool).await?;
        assert!(user.is_none());
        Ok(())
    }

    #[tokio::test]
    async fn create_duplicate_user_should_fail() -> Result<()> {
        // init test database
        let (_tdb, pool) = get_test_pool(None).await;
        // init test data
        let name = "Alice";
        let email = "Alice@test.org";
        let workspace = "acme";
        let password = "password";

        let user_input = UserInput::new(name, email, workspace, password);
        // create duplicate user
        let ret = User::create(&user_input, &pool).await;
        match ret {
            Err(AppError::EmailAlreadyExists(_)) => Ok(()),
            _ => panic!("Expecting EmailAlreadyExists error"),
        }
    }
}
