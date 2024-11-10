use abi::DecodingKey;
use anyhow::{Ok, Result};

use crm_metadata::pb::metadata_client::MetadataClient;

use notification::pb::notification_client::NotificationClient;
use pb::{
    crm_server::{Crm, CrmServer},
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};
use tonic::{
    async_trait, service::interceptor::InterceptedService, transport::Channel, Request, Response,
    Status,
};
use user_state::pb::user_stats_client::UserStatsClient;
mod abi;

mod config;
pub mod pb;

pub use config::AppConfig;

pub struct CrmService {
    config: AppConfig,
    user_state: UserStatsClient<Channel>,
    notification: NotificationClient<Channel>,
    metadata: MetadataClient<Channel>,
}

impl CrmService {
    pub async fn try_new(config: AppConfig) -> Result<Self> {
        let user_state = UserStatsClient::connect(config.server.user_stats.clone()).await?;
        let notification = NotificationClient::connect(config.server.notification.clone()).await?;
        let metadata = MetadataClient::connect(config.server.metadata.clone()).await?;
        Ok(Self {
            config,
            user_state,
            notification,
            metadata,
        })
    }
    pub fn into_server(self) -> Result<InterceptedService<CrmServer<CrmService>, DecodingKey>> {
        let dk = DecodingKey::load(&self.config.auth.pk)?;
        let svc: tonic::service::interceptor::InterceptedService<
            CrmServer<CrmService>,
            DecodingKey,
        > = CrmServer::with_interceptor(self, dk);
        Ok(svc)
    }
}

type ServiceResult<T> = Result<Response<T>, Status>;

#[async_trait]
impl Crm for CrmService {
    async fn welcome(&self, request: Request<WelcomeRequest>) -> ServiceResult<WelcomeResponse> {
        self.welcome(request.into_inner()).await
    }
    async fn recall(&self, request: Request<RecallRequest>) -> ServiceResult<RecallResponse> {
        self.recall(request.into_inner()).await
    }
    async fn remind(&self, request: Request<RemindRequest>) -> ServiceResult<RemindResponse> {
        self.remind(request.into_inner()).await
    }
}
