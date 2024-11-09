use anyhow::Result;

use crm_metadata::pb::metadata_client::MetadataClient;
use notification::pb::notification_client::NotificationClient;
use pb::{
    crm_server::{Crm, CrmServer},
    RecallRequest, RecallResponse, RemindRequest, RemindResponse, WelcomeRequest, WelcomeResponse,
};
use tonic::{async_trait, transport::Channel, Request, Response, Status};
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
    pub fn into_server(self) -> CrmServer<Self> {
        CrmServer::new(self)
    }
}

type ServiceResult<T> = Result<Response<T>, Status>;

#[async_trait]
impl Crm for CrmService {
    async fn welcome(&self, request: Request<WelcomeRequest>) -> ServiceResult<WelcomeResponse> {
        self.welcome(request.into_inner()).await
    }
    async fn recall(&self, _request: Request<RecallRequest>) -> ServiceResult<RecallResponse> {
        todo!()
    }
    async fn remind(&self, _request: Request<RemindRequest>) -> ServiceResult<RemindResponse> {
        todo!()
    }
}
