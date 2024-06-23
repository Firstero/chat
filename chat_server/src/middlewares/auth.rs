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
