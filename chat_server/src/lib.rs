mod config;
mod error;
mod handlers;
mod middlewares;
mod models;
mod utils;

use anyhow::Context;
use error::AppError;
use middlewares::{set_layer, verify_token};
use sqlx::PgPool;
use utils::{DecodingKey, EncodingKey};

use std::{fmt, fs, ops::Deref, sync::Arc};

pub use models::{
    Chat, ChatFile, ChatInput, ChatUser, Message, SigninUser, User, UserInput, Workspace,
};

use axum::{
    middleware::from_fn_with_state,
    routing::{get, post},
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
        .route("/users", get(list_all_users_handler))
        .route("/chats", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chats/:id",
            get(get_chat_handler)
                .patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route(
            "/chats/:id/message",
            get(list_message_handler).post(create_message_handler),
        )
        .route("/upload", post(upload_handler))
        .route("/files/:ws_id/*path", get(download_handler))
        .layer(from_fn_with_state(state.clone(), verify_token))
        // routes doesn't need token verification layer
        .route("/signin", post(signin_handler))
        .route("/signup", post(signup_handler));

    let router = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state.clone());

    Ok(set_layer(router))
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
        fs::create_dir_all(&config.server.base_dir).context("create bash dir failed")?;
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

mod test_utils {

    use super::*;
    use anyhow::Result;
    use sqlx::Executor;
    use sqlx_db_tester::TestPg;
    use std::path::Path;

    impl AppState {
        #[cfg(test)]
        pub async fn new_for_test(config: AppConfig) -> Result<(TestPg, Self)> {
            let sk = EncodingKey::load(&config.auth.sk).context("load sk failed")?;
            let pk = DecodingKey::load(&config.auth.pk).context("load pk failed")?;
            let pos = config.server.db_url.rfind('/').expect("invalid db url");
            let server_url = &config.server.db_url[..pos];
            let (tdb, pool) = get_test_pool(Some(server_url)).await;
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

    pub async fn get_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let server_url = url.unwrap_or("postgres://firstero:firstero@localhost:5432");
        let tdb = TestPg::new(server_url.to_string(), Path::new("../migrations"));
        let pool = tdb.get_pool().await;
        // run prepare sql to insert test data
        let sql = include_str!("../fixtures/test.sql").split(';');
        let mut ts = pool.begin().await.expect("begin transaction failed");
        for s in sql {
            if s.trim().is_empty() {
                continue;
            }
            ts.execute(s).await.expect("execute sql failed");
        }

        ts.commit().await.expect("commit transaction failed");

        (tdb, pool)
    }
}
