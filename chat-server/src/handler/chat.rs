use crate::model::{Chat, CreateChat, UpdateChat};
use crate::{AppError, AppState, User};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::{Extension, Json};

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
    Json(input): Json<CreateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::create(input, user.ws_id as _, &state.pool).await?;
    Ok((StatusCode::CREATED, Json(chat)))
}

pub(crate) async fn get_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::get_by_id(id, &state.pool).await?;
    match chat {
        None => Err(AppError::NotFound(format!("chat id {id}"))),
        Some(chat) => Ok(Json(chat)),
    }
}

pub(crate) async fn update_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
    Json(input): Json<UpdateChat>,
) -> Result<impl IntoResponse, AppError> {
    let chat = Chat::update_by_id(id, input, &state.pool).await?;
    Ok(Json(chat))
}

pub(crate) async fn delete_chat_handler(
    State(state): State<AppState>,
    Path(id): Path<u64>,
) -> Result<impl IntoResponse, AppError> {
    Chat::delete_by_id(id, &state.pool).await?;
    Ok(StatusCode::OK)
}
