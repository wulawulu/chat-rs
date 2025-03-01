use crate::AppState;
use axum::{
    extract::Request,
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{headers::authorization::Bearer, headers::Authorization, TypedHeader};
use tracing::warn;

pub async fn auth(
    State(state): State<AppState>,
    TypedHeader(Authorization(bearer)): TypedHeader<Authorization<Bearer>>,
    mut req: Request,
    next: Next,
) -> Response {
    match state.dk.verify(bearer.token()) {
        Ok(user) => {
            req.extensions_mut().insert(user);
        }
        Err(e) => {
            let msg = format!("verify token failed: {}", e);
            warn!(msg);
            return (StatusCode::FORBIDDEN, msg).into_response();
        }
    }

    next.run(req).await
}
