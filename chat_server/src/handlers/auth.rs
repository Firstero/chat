use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::{Deserialize, Serialize};

use crate::{error::AppError, models::SigninUser, models::UserInput, AppState, User};
#[derive(Debug, Deserialize, Serialize)]
pub struct AuthOutput {
    token: String,
}

pub(crate) async fn signup_handler(
    State(state): State<AppState>,
    Json(input): Json<UserInput>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::create(&input, &state.pool).await?;
    let token = state.sk.encode(user)?;
    // let mut header = HeaderMap::new();
    // header.insert("X-token", HeaderValue::from_str(&token)?);
    // Ok((StatusCode::CREATED, header))
    let body = Json(AuthOutput { token });
    Ok((StatusCode::CREATED, body))
}

pub(crate) async fn signin_handler(
    State(state): State<AppState>,
    Json(input): Json<SigninUser>,
) -> Result<impl IntoResponse, AppError> {
    let user = User::verify(&input, &state.pool).await?;
    match user {
        Some(user) => {
            let token = state.sk.encode(user)?;
            Ok((StatusCode::OK, Json(AuthOutput { token })).into_response())
        }
        None => Ok((StatusCode::FORBIDDEN, "Invalid email or password").into_response()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::AppConfig;
    use anyhow::Result;
    use http_body_util::BodyExt;

    #[tokio::test]
    pub async fn signup_should_work() -> Result<()> {
        let config = AppConfig::try_load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        let input = UserInput::new("firsteor", "firstero@email", "acme", "password");
        let ret = signup_handler(State(state), Json(input))
            .await?
            .into_response();
        // check status
        assert_eq!(ret.status(), StatusCode::CREATED);
        // check token
        let body = ret.into_body().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");
        Ok(())
    }

    #[tokio::test]
    pub async fn signin_should_work() -> Result<()> {
        let config = AppConfig::try_load()?;
        let (_tdb, state) = AppState::new_for_test(config).await?;
        // init input
        let name = "firstero";
        let email = "firstero@email.org";
        let workspace = "acme";
        let password = "password";
        let input = UserInput::new(name, email, workspace, password);
        User::create(&input, &state.pool).await?;

        let sign_input = SigninUser::new(email, password);
        let ret = signin_handler(State(state), Json(sign_input))
            .await?
            .into_response();
        assert_eq!(ret.status(), StatusCode::OK);
        let body = ret.into_body().into_response().collect().await?.to_bytes();
        let ret: AuthOutput = serde_json::from_slice(&body)?;
        assert_ne!(ret.token, "");
        Ok(())
    }
}
