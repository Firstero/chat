mod sse;

use axum::{routing::get, Router};

use sse::{index_handler, sse_handler};

pub fn get_router() -> Router {
    // build our application with a route
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}
