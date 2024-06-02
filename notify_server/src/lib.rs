mod sse;

use axum::{routing::get, Router};

use sse::sse_handler;

pub fn get_router() -> Router {
    // build our application with a route
    Router::new().route("/events", get(sse_handler))
}
