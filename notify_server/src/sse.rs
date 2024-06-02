use std::{convert::Infallible, time::Duration};

use axum::response::{
    sse::{Event, Sse},
    Html, IntoResponse,
};

use axum_extra::{headers, TypedHeader};
use futures::{self, stream, Stream};
use tokio_stream::StreamExt as _;
use tracing::info;

const INDEX_HTML: &str = include_str!("../index.html");

pub(crate) async fn sse_handler(
    TypedHeader(user_agent): TypedHeader<headers::UserAgent>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    info!("`{}` connected", user_agent.as_str());

    // A `Stream` that repeats an event every second
    //
    // You can also create streams from tokio channels using the wrappers in
    // https://docs.rs/tokio-stream
    let stream = stream::repeat_with(|| Event::default().data("hello, sse client!"))
        .map(Ok)
        .throttle(Duration::from_secs(1));

    Sse::new(stream).keep_alive(
        axum::response::sse::KeepAlive::new()
            .interval(Duration::from_secs(1))
            .text("keep-alive-text"),
    )
}

pub(crate) async fn index_handler() -> impl IntoResponse {
    Html(INDEX_HTML)
}
