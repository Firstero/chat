use anyhow::Result;

use notify_server::get_router;
use tokio::net::TcpListener;
use tracing::{info, level_filters::LevelFilter, warn};
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};
#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    warn!("Prepare Starting to load {} config", env!("CARGO_PKG_NAME"));
    let addr = format!("0.0.0.0:{}", 6687);

    let listener = TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);

    let app = get_router();

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
