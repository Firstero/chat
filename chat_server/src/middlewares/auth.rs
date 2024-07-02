use axum::{
    extract::{FromRequestParts, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::AppState;

pub async fn verify_token(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();
    match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
        Ok(TypedHeader(Authorization(bearer))) => {
            let token = bearer.token();
            match state.pk.verify(token) {
                Ok(user) => {
                    parts.extensions.insert(user);
                    let req = Request::from_parts(parts, body);
                    next.run(req).await
                }
                Err(e) => {
                    let msg = format!("Failed to verify token: {}, {:?}", e, token);
                    tracing::warn!(msg);
                    (StatusCode::FORBIDDEN, msg).into_response()
                }
            }
        }
        Err(e) => {
            let msg = format!("Failed to parse token: {:?}", e);
            tracing::warn!(msg);
            (StatusCode::UNAUTHORIZED, msg).into_response()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::User;

    use super::*;
    use anyhow::Result;
    use axum::{body::Body, http::Request, middleware::from_fn_with_state, routing::get, Router};
    use tower::ServiceExt;

    async fn handler() -> impl IntoResponse {
        (StatusCode::OK, "ok")
    }

    #[tokio::test]
    async fn verify_token_should_work() -> Result<()> {
        let user = User::new(1, "test", "password");
        let (_tdb, state) = AppState::new_for_test().await?;
        let token = state.sk.encode(user)?;
        let app = Router::new()
            .route("/", get(handler))
            .layer(from_fn_with_state(state.clone(), verify_token));

        // good token
        let req = Request::builder()
            .uri("/")
            .header("Authorization", format!("Bearer {}", token))
            .body(Body::empty())?;
        let res = app.clone().oneshot(req).await?;
        assert_eq!(res.status(), StatusCode::OK);

        // no token
        let res = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty())?)
            .await?;
        assert_eq!(res.status(), StatusCode::UNAUTHORIZED);

        // bad token
        let res = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("Authorization", "Bearer bad token")
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(res.status(), StatusCode::FORBIDDEN);
        Ok(())
    }
}
