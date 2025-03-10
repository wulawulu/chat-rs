use crate::{AppError, AppState, User};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};

pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_chat_users(user.ws_id as _).await?;
    Ok(Json(users))
}
