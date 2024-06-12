use crate::{error::AppError, User};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::mem;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow, PartialEq)]
pub struct UserInput {
    pub fullname: String,
    pub email: String,
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
    pub async fn find_by_email(
        email: &str,
        pool: &sqlx::PgPool,
    ) -> Result<Option<Self>, sqlx::Error> {
        let user = sqlx::query_as("SELECT * FROM users WHERE email = $1")
            .bind(email)
            .fetch_optional(pool)
            .await?;

        Ok(user)
    }

    /// create 方法
    pub async fn create(user: &UserInput, pool: &sqlx::PgPool) -> Result<User, AppError> {
        let exists_user = User::find_by_email(&user.email, pool).await?;

        if exists_user.is_some() {
            return Err(AppError::EmailAlreadyExists(user.email.to_string()));
        }
        // 使用 argon2 生成密码哈希
        let password = hash_password(&user.password)?;
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (fullname, email, password_hash)
            VALUES ($1, $2, $3)
            RETURNING id, fullname, email, created_at
            "#,
        )
        .bind(&user.fullname)
        .bind(&user.email)
        .bind(password)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    /// verify email and password
    pub async fn verify(input: &SigninUser, pool: &sqlx::PgPool) -> Result<Option<Self>, AppError> {
        // find user by email
        let user: Option<User> = sqlx::query_as(
            "SELECT id, fullname, email, password_hash, created_at FROM users WHERE email = $1",
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
            fullname: fullname.to_string(),
            email: email.to_string(),
            password_hash: None,
            created_at: chrono::Utc::now(),
        }
    }
}

#[cfg(test)]
impl UserInput {
    pub fn new(fullname: &str, email: &str, password: &str) -> Self {
        Self {
            fullname: fullname.to_string(),
            email: email.to_string(),
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
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    #[test]
    fn test_hash_password() -> Result<()> {
        let password = "password";
        let password_hash = hash_password(password)?;
        assert!(verify_password(password, &password_hash)?);
        Ok(())
    }

    #[tokio::test]
    async fn test_user_create_find_verify() -> Result<()> {
        // init test database
        let server_url = "postgres://firstero:firstero@localhost:5432".to_string();
        let tdb = TestPg::new(server_url, Path::new("../migrations"));
        let pool = tdb.get_pool().await;

        // init test data
        let name = "firstero";
        let email = "firsero@acme.org";
        let password = "password";

        let user_input = UserInput::new(name, email, password);

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
}
