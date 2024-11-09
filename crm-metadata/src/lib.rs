mod abi;
mod config;
pub mod pb;
pub use abi::*;
pub use config::AppConfig;
use futures::Stream;
use pb::{
    metadata_server::{Metadata, MetadataServer},
    Content, MaterializeRequest,
};
use std::pin::Pin;
use tonic::{Request, Response, Status, Streaming};
#[allow(unused)]
pub struct MetaDataService {
    config: AppConfig,
}

impl MetaDataService {
    pub fn new(config: AppConfig) -> Self {
        Self { config }
    }
    pub fn into_server(self) -> MetadataServer<Self> {
        MetadataServer::new(self)
    }
}

type ResponseStream = Pin<Box<dyn Stream<Item = Result<Content, Status>> + Send>>;
type ServiceResult<T> = Result<Response<T>, Status>;
#[tonic::async_trait]
impl Metadata for MetaDataService {
    type MaterializeStream = ResponseStream;
    async fn materialize(
        &self,
        request: Request<Streaming<MaterializeRequest>>,
    ) -> ServiceResult<Self::MaterializeStream> {
        let ret = self.materialize(request.into_inner()).await?;
        Ok(ret)
    }
}
