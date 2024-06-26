use axum::{extract::Request, http::HeaderValue, response::Response};
use std::{
    future::Future,
    pin::Pin,
    task::{Context, Poll},
};
use tokio::time::Instant;
use tower::{Layer, Service};

use super::{REQUEST_ID_HEADER, SERVER_TIME};

#[derive(Clone)]
pub(crate) struct ServerTimeLayer;

impl<S> Layer<S> for ServerTimeLayer {
    type Service = ServerTimeMiddleware<S>;

    fn layer(&self, inner: S) -> Self::Service {
        ServerTimeMiddleware { inner }
    }
}

#[derive(Clone)]
pub(crate) struct ServerTimeMiddleware<S> {
    inner: S,
}

impl<S> Service<Request> for ServerTimeMiddleware<S>
where
    S: Service<Request, Response = Response> + Send + 'static,
    S::Future: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future =
        Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send + 'static>>;
    // `BoxFuture` is a type alias for `Pin<Box<dyn Future + Send + 'a>>`

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, request: Request) -> Self::Future {
        let start = Instant::now();
        let future = self.inner.call(request);
        Box::pin(async move {
            let mut res: Response = future.await?;
            let elapsed = format!("{:?}us", start.elapsed().as_micros());
            match HeaderValue::from_str(&elapsed) {
                Ok(v) => {
                    res.headers_mut().insert(SERVER_TIME, v);
                }
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse server time: {}, for request {:?}",
                        e,
                        res.headers().get(REQUEST_ID_HEADER)
                    );
                }
            }
            Ok(res)
        })
    }
}
