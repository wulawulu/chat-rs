mod auth;
mod request_id;
mod server_time;

use crate::middleware::server_time::ServerTimeLayer;
pub use auth::auth;
use axum::Router;
use tower::ServiceBuilder;
use tower_http::trace::TraceLayer;
use tower_http::trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse};
use tower_http::{compression::CompressionLayer, LatencyUnit};
use tracing::Level;

#[allow(unused)]
const REQUEST_ID_HEADER: &str = "x-request-id";
const SERVER_TIME_HEADER: &str = "x-server-time";

pub fn set_layer(app: Router) -> Router {
    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(DefaultMakeSpan::new().include_headers(true))
        .on_request(DefaultOnRequest::new().level(Level::INFO))
        .on_response(
            DefaultOnResponse::new()
                .level(Level::INFO)
                .latency_unit(LatencyUnit::Micros),
        );
    app.layer(
        ServiceBuilder::new()
            .layer(tracing_layer)
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(ServerTimeLayer),
    )
}
