use std::{sync::Arc, time::Duration};

use chrono::Utc;
use futures::{Stream, StreamExt};
use prost_types::Timestamp;
use tokio::{sync::mpsc, time::sleep};
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};
use tracing::{info, warn};

use crate::{
    pb::{
        notification_server::NotificationServer, send_request::Msg, EmailMessage, InAppMessage,
        SendRequest, SendResponse, SmsMessage,
    },
    AppConfig, NotificationService, NotificationServiceInner, ResponseStream, ServiceResult,
};

pub trait Sender {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status>;
}

impl NotificationService {
    pub async fn send(
        &self,
        mut stream: impl Stream<Item = Result<SendRequest, tonic::Status>> + Send + 'static + Unpin,
    ) -> ServiceResult<ResponseStream> {
        let (tx, rx) = mpsc::channel(1024);
        let self_ = self.clone();
        tokio::spawn(async move {
            while let Some(Ok(req)) = stream.next().await {
                let self_ = self_.clone();
                let res = match req.msg {
                    Some(Msg::Email(email)) => email.send(self_).await,
                    Some(Msg::Sms(sms)) => sms.send(self_).await,
                    Some(Msg::InApp(in_app)) => in_app.send(self_).await,
                    None => {
                        tx.send(Err(Status::invalid_argument("msg is required")))
                            .await
                            .unwrap();
                        continue;
                    }
                };
                tx.send(res).await.unwrap();
            }
        });
        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }

    pub fn new(config: AppConfig) -> Self {
        let sender = dummy_send();
        Self {
            inner: Arc::new(NotificationServiceInner { config, sender }),
        }
    }
    pub fn into_server(self) -> NotificationServer<Self> {
        NotificationServer::new(self)
    }
}

pub fn dummy_send() -> mpsc::Sender<Msg> {
    let (tx, mut rx) = mpsc::channel(1024 * 100);
    tokio::spawn(async move {
        while let Some(req) = rx.recv().await {
            info!("send message {:?}", req);
            sleep(Duration::from_millis(300)).await;
        }
    });
    tx
}
fn to_ts() -> Timestamp {
    let now = Utc::now();
    Timestamp {
        seconds: now.timestamp(),
        nanos: now.timestamp_subsec_nanos() as i32,
    }
}

impl Sender for EmailMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();
        svc.sender.send(Msg::Email(self)).await.map_err(|e| {
            warn!("Failed to send email: {}", e);
            Status::internal(format!("Failed to send email: {}", e))
        })?;

        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}
impl Sender for SmsMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();

        svc.sender.send(Msg::Sms(self)).await.map_err(|e| {
            warn!("Failed to send sms: {}", e);
            Status::internal(format!("Failed to send sms: {}", e))
        })?;

        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}
impl Sender for InAppMessage {
    async fn send(self, svc: NotificationService) -> Result<SendResponse, Status> {
        let message_id = self.message_id.clone();

        svc.sender.send(Msg::InApp(self)).await.map_err(|e| {
            warn!("Failed to send in-app message: {}", e);
            Status::internal(format!("Failed to send in-app message: {}", e))
        })?;

        Ok(SendResponse {
            message_id,
            timestamp: Some(to_ts()),
        })
    }
}

#[cfg(test)]
mod tests {

    use anyhow::Result;
    use fake::{
        faker::{internet::en::SafeEmail, phone_number::ar_sa::PhoneNumber},
        Fake,
    };
    use futures::StreamExt;
    use uuid::Uuid;

    use crate::{
        pb::{send_request::Msg, EmailMessage, InAppMessage, SendRequest, SmsMessage},
        AppConfig, NotificationService,
    };

    impl EmailMessage {
        fn fake() -> Self {
            let buf: [u8; 16] = *b"abcdefghijklmnop";
            EmailMessage {
                message_id: Uuid::new_v8(buf).to_string(),
                sender: SafeEmail().fake(),
                recipients: SafeEmail().fake(),
                subject: "Hello".to_string(),
                body: "Hello".to_string(),
            }
        }
    }
    impl From<EmailMessage> for Msg {
        fn from(email: EmailMessage) -> Self {
            Msg::Email(email)
        }
    }

    impl SmsMessage {
        fn fake() -> Self {
            let buf: [u8; 16] = *b"abcdefghijklmnop";
            SmsMessage {
                message_id: Uuid::new_v8(buf).to_string(),
                sender: PhoneNumber().fake(),
                recipients: PhoneNumber().fake(),
                body: "Hello".to_string(),
            }
        }
    }

    impl InAppMessage {
        fn fake() -> Self {
            let buf: [u8; 16] = *b"abcdefghijklmnop";
            InAppMessage {
                message_id: Uuid::new_v8(buf).to_string(),
                device_id: Uuid::new_v8(buf).to_string(),
                title: "Hello".to_string(),
                body: "Hello".to_string(),
            }
        }
    }

    #[tokio::test]
    async fn send_should_work() -> Result<()> {
        let service = NotificationService::new(AppConfig::try_load()?);
        let email = EmailMessage::fake();

        let request_stream = tokio_stream::iter(vec![
            Ok(SendRequest {
                msg: Some(Msg::Email(email)),
            }),
            Ok(SendRequest {
                msg: Some(Msg::Sms(SmsMessage::fake())),
            }),
            Ok(SendRequest {
                msg: Some(Msg::InApp(InAppMessage::fake())),
            }),
        ]);

        let response = service.send(request_stream).await.unwrap();
        let response = response.into_inner();
        let content = response.collect::<Vec<_>>().await;

        assert_eq!(content.len(), 3);
        Ok(())
    }
}
