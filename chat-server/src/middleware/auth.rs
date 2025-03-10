use crate::AppState;
use axum::{
    extract::Request,
    extract::State,
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::headers::{HeaderMap, HeaderMapExt};
use axum_extra::{headers::authorization::Bearer, headers::Authorization};
use tracing::warn;

pub async fn verify_token(
    State(state): State<AppState>,
    headers: HeaderMap,
    mut req: Request,
    next: Next,
) -> Response {
    let option = headers.typed_get::<Authorization<Bearer>>();
    let Some(Authorization(bearer)) = option else {
        let msg = "missing Authorization header";
        warn!(msg);
        return (StatusCode::UNAUTHORIZED, msg).into_response();
    };
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::User;
    use anyhow::Result;
    use axum::body::Body;
    use axum::http::Request;
    use axum::middleware::from_fn_with_state;
    use axum::routing::get;
    use axum::Router;
    use tower::ServiceExt;

    async fn handle() -> impl IntoResponse {
        (StatusCode::OK, "OK")
    }

    #[tokio::test]
    async fn test_verify_token() -> Result<()> {
        let (_tdb, state) = AppState::new_for_test().await?;

        let user = User::new(1, "wu", "wu@github.org");
        let token = state.ek.sign(user)?;

        let app = Router::new()
            .route("/", get(handle))
            .layer(from_fn_with_state(state.clone(), verify_token))
            .with_state(state);

        let response = app
            .clone()
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("Authorization", format!("Bearer {}", token))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::OK);

        let response = app
            .clone()
            .oneshot(Request::builder().uri("/").body(Body::empty())?)
            .await?;
        assert_eq!(response.status(), StatusCode::UNAUTHORIZED);

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/")
                    .header("Authorization", format!("Bearer {}", "bad token"))
                    .body(Body::empty())?,
            )
            .await?;
        assert_eq!(response.status(), StatusCode::FORBIDDEN);

        Ok(())
    }
}
