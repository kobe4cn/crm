use std::{ops::Deref, pin::Pin, sync::Arc};
mod abi;
mod config;
use anyhow::Result;
use duckdb::{AccessMode, Config, Connection};
use pb::User;
pub mod pb;
pub use config::AppConfig;
use pb::{
    user_stats_server::{UserStats, UserStatsServer},
    QueryRequest, RawQueryRequest,
};
use sqlx::PgPool;
use tokio::sync::Mutex;
use tonic::{Request, Response, Status};
#[derive(Clone)]
pub struct UserStatsService {
    inner: Arc<UserStateServiceInner>,
}
#[allow(unused)]
pub struct UserStateServiceInner {
    config: AppConfig,
    pool: PgPool,
    duck: Mutex<Connection>,
}
impl UserStatsService {
    pub async fn new(config: AppConfig) -> Result<Self> {
        let pool = PgPool::connect(&config.server.db_url).await?;
        let duck_config = Config::default().access_mode(AccessMode::ReadWrite)?;
        let duck = Connection::open_with_flags(&config.server.duck_db, duck_config)?;

        let inner = UserStateServiceInner {
            config,
            pool,
            duck: Mutex::new(duck),
        };
        Ok(Self {
            inner: Arc::new(inner),
        })
    }

    pub fn into_server(self) -> UserStatsServer<Self> {
        UserStatsServer::new(self)
    }
}
impl Deref for UserStatsService {
    type Target = UserStateServiceInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

use futures::Stream;
type ResponseStream = Pin<Box<dyn Stream<Item = Result<User, Status>> + Send>>;
type ServiceResult<T> = Result<Response<T>, Status>;
#[tonic::async_trait]
#[allow(unused)]
impl UserStats for UserStatsService {
    type QueryStream = ResponseStream;
    type RawQueryStream = ResponseStream;

    async fn query(&self, request: Request<QueryRequest>) -> ServiceResult<Self::QueryStream> {
        // Implement your logic here

        self.query(request.into_inner()).await
    }

    async fn raw_query(
        &self,
        request: Request<RawQueryRequest>,
    ) -> ServiceResult<Self::RawQueryStream> {
        // Implement your logic here
        self.raw_query(request.into_inner()).await
    }
}
