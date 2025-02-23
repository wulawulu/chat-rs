mod auth;
mod chat;
mod message;

pub(crate) use auth::*;
pub(crate) use chat::*;
pub(crate) use message::*;

pub(crate) async fn index_handler() -> &'static str {
    "index"
}
