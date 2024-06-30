use axum::{
    body::Body,
    extract::{Multipart, Path, Request, State},
    response::IntoResponse,
    Extension, Json,
};
use tokio::fs;
use tower_http::services::ServeFile;
use tracing::{info, warn};

use crate::{error::AppError, AppState, ChatFile, User};

pub(crate) async fn send_message_handler() -> impl IntoResponse {
    todo!()
}

pub(crate) async fn list_message_handler() -> impl IntoResponse {
    todo!()
}

pub(crate) async fn create_message_handler() -> impl IntoResponse {
    todo!()
}

pub(crate) async fn download_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    Path((ws_id, path)): Path<(i64, String)>,
) -> Result<impl IntoResponse, AppError> {
    if user.ws_id != ws_id {
        return Err(AppError::NotFound(
            "File doesn't exist or you don't have permission".to_string(),
        ));
    }
    let base_dir = state.config.server.base_dir.join(ws_id.to_string());
    let path = base_dir.join(path);
    if !path.exists() {
        return Err(AppError::NotFound("File doesn't exist".to_string()));
    }
    let req = Request::new(Body::empty());
    let res = ServeFile::new(path).try_call(req).await?;
    Ok(res.into_response())
}

pub(crate) async fn upload_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
    mut multipart: Multipart,
) -> Result<impl IntoResponse, AppError> {
    let ws_id = user.ws_id as u64;
    let base_dir = &state.config.server.base_dir;
    let mut files = vec![];
    while let Some(field) = multipart.next_field().await.unwrap() {
        let filename = field.file_name().map(|name| name.to_string());
        let (Some(filename), Ok(data)) = (filename, field.bytes().await) else {
            warn!("Failed to read multipart field");
            continue;
        };

        let file = ChatFile::new(ws_id, &filename, &data);
        let path = file.path(base_dir);
        if path.exists() {
            info!("File {} already exists: {:?}", filename, path);
        } else {
            fs::create_dir_all(path.parent().expect("file path parent should exists")).await?;
            fs::write(path, data).await?;
        }
        files.push(file.url());
    }

    Ok(Json(files))
}
