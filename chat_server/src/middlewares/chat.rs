use axum::{
    extract::{FromRequestParts, Path, Request, State},
    middleware::Next,
    response::{IntoResponse, Response},
};

use crate::{error::AppError, AppState, User};

pub async fn verify_chat(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();

    let Path(chat_id) = Path::<u64>::from_request_parts(&mut parts, &state)
        .await
        .unwrap();
    let user = parts.extensions.get::<User>().unwrap();

    if !state
        .is_chat_member(chat_id, user.id as _)
        .await
        .unwrap_or_default()
    {
        let err =
            AppError::VerifyChatError(format!("User {} is not member of chat {chat_id}", user.id));
        return err.into_response();
    }
    let req = Request::from_parts(parts, body);
    next.run(req).await
}

#[cfg(test)]
mod tests {
    use crate::middlewares::verify_token;

    use super::*;
    use anyhow::Result;
    use axum::{
        body::Body,
        http::{Request, StatusCode},
        middleware::from_fn_with_state,
        routing::get,
        Router,
    };
    use tower::ServiceExt;

    async fn handler() -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }

    #[tokio::test]
    async fn verify_chat_middleware_should_work() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;
        let user = state.find_user_by_id(1).await?.expect("user should exists");
        let token = state.sk.encode(user)?;
        let app = Router::new()
            .route("/chats/:id/messages", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_chat))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state);

        // user is member of chat
        let req = Request::builder()
            .uri("/chats/1/messages")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        // user is not member of chat
        let req = Request::builder()
            .uri("/chats/5/messages")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        Ok(())
    }
}
