mod auth;
mod request_id;
mod server_time;

use axum::{middleware::from_fn, Router};
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use self::request_id::set_request_id;
use self::server_time::ServerTimeLayer;

pub use self::auth::verify_token;

const REQUEST_ID_HEADER: &str = "x-request-id";
const SERVER_TIME: &str = "x-server-time";

pub(crate) fn set_layer(router: Router) -> Router {
    router.layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .layer(
                CompressionLayer::new()
                    .br(true)
                    .gzip(true)
                    .zstd(true)
                    .deflate(true),
            )
            .layer(from_fn(set_request_id))
            .layer(ServerTimeLayer),
    )
}
