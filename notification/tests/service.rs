use std::{net::SocketAddr, time::Duration};

use anyhow::Result;

use notification::{
    pb::{notification_client::NotificationClient, send_request::Msg, EmailMessage, SendRequest},
    AppConfig, NotificationService,
};
use tokio::time::sleep;
use tonic::{transport::Server, Request};

use fake::{faker::internet::en::SafeEmail, Fake};
use futures::StreamExt;
use uuid::Uuid;

fn email_fake() -> EmailMessage {
    let buf: [u8; 16] = *b"abcdefghijklmnop";
    EmailMessage {
        message_id: Uuid::new_v8(buf).to_string(),
        sender: SafeEmail().fake(),
        recipients: SafeEmail().fake(),
        subject: "Hello".to_string(),
        body: "Hello".to_string(),
    }
}

#[tokio::test]
async fn send_should_work() -> Result<()> {
    let addr = start_server().await?;
    let addr = format!("http://{}", addr);
    let mut client = NotificationClient::connect(addr).await?;
    let request_stream = tokio_stream::iter(vec![SendRequest {
        msg: Some(Msg::Email(email_fake())),
    }]);
    let request = Request::new(request_stream);

    let response = client.send(request).await?;
    let response = response.into_inner();
    let content = response
        .then(|res| async { res.unwrap() })
        .collect::<Vec<_>>()
        .await;

    assert_eq!(content.len(), 1);
    Ok(())
}

async fn start_server() -> Result<SocketAddr> {
    let config = AppConfig::try_load()?;
    let addr = format!("[::1]:{}", &config.server.port).parse()?;
    let svc = NotificationService::new(config).into_server();
    tokio::spawn(async move {
        Server::builder()
            .add_service(svc)
            .serve(addr)
            .await
            .unwrap();
    });
    sleep(Duration::from_micros(1)).await;
    Ok(addr)
}
