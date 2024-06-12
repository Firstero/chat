mod config;
mod error;
mod handlers;
mod models;
mod utils;

use anyhow::Context;
use error::AppError;
pub use models::User;
use sqlx::PgPool;
use utils::{DecodingKey, EncodingKey};

use std::{fmt, ops::Deref, sync::Arc};

use axum::{
    routing::{get, patch, post},
    Router,
};
use handlers::*;

pub use config::AppConfig;

#[derive(Debug, Clone)]
pub(crate) struct AppState {
    inner: Arc<AppStateInner>,
}

#[allow(unused)]
pub(crate) struct AppStateInner {
    pub(crate) config: AppConfig,
    // auth jwt config
    pub(crate) sk: EncodingKey,
    pub(crate) pk: DecodingKey,
    // db config
    pub(crate) pool: PgPool,
}

pub async fn get_router(config: AppConfig) -> Result<Router, AppError> {
    let state = AppState::try_new(config).await?;

    let api = Router::new()
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler))
        .route(
            "/chat",
            get(list_chat_handler)
                .post(create_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler),
        )
        .route(
            "/chat/:id",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route(
            "/chat/:id/message",
            get(list_message_handler).post(create_message_handler),
        );

    let router = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(router)
}

// 给 AppState 实现 Dereference trait
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppError> {
        let sk = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;

        let pool = PgPool::connect(&config.server.db_url)
            .await
            .context("connect db failed")?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                sk,
                pk,
                pool,
            }),
        })
    }
}

impl fmt::Debug for AppStateInner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(test)]
impl AppState {
    #[cfg(test)]
    pub async fn new_for_test(config: AppConfig) -> anyhow::Result<(sqlx_db_tester::TestPg, Self)> {
        use std::path::Path;
        let sk = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
        let pk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
        // let server_url = config.server.db_url.split('/').next().unwrap();
        let server_url = "postgresql://firstero:firstero@localhost:5432".to_string();
        let tdb = sqlx_db_tester::TestPg::new(server_url.to_string(), Path::new("../migrations"));

        let pool = tdb.get_pool().await;
        Ok((
            tdb,
            Self {
                inner: Arc::new(AppStateInner {
                    config,
                    sk,
                    pk,
                    pool,
                }),
            },
        ))
    }
}
