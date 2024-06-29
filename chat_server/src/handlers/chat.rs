use axum::{
    extract::{Path, State},
    http::StatusCode,
    response::IntoResponse,
    Extension, Json,
};

use crate::{error::AppError, AppState, Chat, ChatInput, User};

pub(crate) async fn list_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::fetch_all(user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::OK, Json(chat)))
}

pub(crate) async fn create_chat_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Json(chat): Json<ChatInput>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(&chat, user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::find_by_id(id, &state.pool).await?;
    chat.map(|chat| Ok((StatusCode::OK, Json(chat))))
        .unwrap_or_else(|| Err(AppError::NotFound(format!("chat id: {}", id))))
}

// TODO: implement the following handlers
pub(crate) async fn update_chat_handler() -> impl IntoResponse {
    todo!()
}

pub(crate) async fn delete_chat_handler() -> impl IntoResponse {
    todo!()
}
