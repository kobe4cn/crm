mod config;
pub mod pb;
use std::{ops::Deref, pin::Pin, sync::Arc};
mod abi;

pub use config::AppConfig;
use futures::Stream;
pub use pb::{
    notification_server::Notification, send_request::Msg, EmailMessage, InAppMessage, SendRequest,
    SendResponse, SmsMessage,
};
use tokio::sync::mpsc;
use tonic::{Request, Response, Status, Streaming};

#[derive(Clone)]
pub struct NotificationService {
    inner: Arc<NotificationServiceInner>,
}
#[allow(unused)]
pub struct NotificationServiceInner {
    config: AppConfig,
    sender: mpsc::Sender<Msg>,
}

impl Deref for NotificationService {
    type Target = NotificationServiceInner;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
type ResponseStream = Pin<Box<dyn Stream<Item = Result<SendResponse, Status>> + Send>>;
type ServiceResult<T> = Result<Response<T>, Status>;
#[tonic::async_trait]
impl Notification for NotificationService {
    type SendStream = ResponseStream;

    async fn send(
        &self,
        request: Request<Streaming<SendRequest>>,
    ) -> ServiceResult<Self::SendStream> {
        let ret = self.send(request.into_inner()).await?;
        Ok(ret)
    }
}
