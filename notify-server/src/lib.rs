mod config;
mod error;
mod notify;
mod sse;

use anyhow::Context;
pub use config::AppConfig;
use std::fmt;
use std::ops::Deref;
use std::sync::Arc;

use crate::error::AppError;
use crate::notify::AppEvent;
use axum::middleware::from_fn_with_state;
use axum::{
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use chat_core::{verify_token, DecodingKey, TokenVerify, User};
use dashmap::DashMap;
use sse::sse_handler;
use tokio::sync::broadcast;

pub use notify::setup_pg_listener;

type UserMap = Arc<DashMap<u64, broadcast::Sender<Arc<AppEvent>>>>;

#[derive(Debug, Clone)]
pub struct AppState(Arc<AppStateInner>);

pub struct AppStateInner {
    pub config: AppConfig,
    pub dk: DecodingKey,
    pub users: UserMap,
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let dk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        let users = Arc::new(DashMap::new());
        Ok(Self(Arc::new(AppStateInner { config, dk, users })))
    }
}

impl TokenVerify for AppState {
    type Error = AppError;

    fn verify(&self, token: &str) -> Result<User, Self::Error> {
        self.dk.verify(token).map_err(AppError::from)
    }
}

impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

const INDEX_HTML: &str = include_str!("../index.html");

pub async fn get_router(config: AppConfig) -> (Router, AppState) {
    let app_state = AppState::try_new(config).await.expect("init failed");
    let router = Router::new()
        .route("/events", get(sse_handler))
        .layer(from_fn_with_state(
            app_state.clone(),
            verify_token::<AppState>,
        ))
        .route("/", get(index_handler))
        .with_state(app_state.clone());
    (router, app_state)
}

async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
