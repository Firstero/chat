use axum::{extract::State, http::StatusCode, response::IntoResponse, Extension, Json};

use crate::{error::AppError, AppState, User};

pub(crate) async fn list_all_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_all_chat_users(user.ws_id as _).await?;
    Ok((StatusCode::OK, Json(users)))
}
