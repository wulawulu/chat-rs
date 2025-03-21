use crate::{AppError, AppState};
use axum::extract::State;
use axum::response::IntoResponse;
use axum::{Extension, Json};
use chat_core::{ChatUser, User};

#[utoipa::path(
    get,
    path = "/api/users",
    responses(
         (status = 200, description = "Chat users", body = ChatUser),
    ),
    tag="user",
    security(
         ("token" = [])
    )
)]
pub(crate) async fn list_chat_users_handler(
    Extension(user): Extension<User>,
    State(state): State<AppState>,
) -> Result<impl IntoResponse, AppError> {
    let users = state.fetch_chat_users(user.ws_id as _).await?;
    Ok(Json(users))
}
