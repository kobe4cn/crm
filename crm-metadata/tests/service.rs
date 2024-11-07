use std::{net::SocketAddr, time::Duration};

use anyhow::Result;
use crm_metadata::{
    pb::{metadata_client::MetadataClient, MaterializeRequest},
    AppConfig, MetaDataService,
};
use futures::StreamExt;
use tokio::time::sleep;
use tonic::transport::Server;

async fn start_server() -> Result<SocketAddr> {
    let config = AppConfig::try_load()?;
    let addr = format!("[::1]:{}", &config.server.port).parse()?;
    let svc = MetaDataService::new(config).into_server();
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

#[tokio::test]
async fn get_metadata_should_work() -> Result<()> {
    let addr = start_server().await?;
    let addr = format!("http://{}", addr);
    let mut client = MetadataClient::connect(addr).await?;
    let request_stream = tokio_stream::iter(vec![
        MaterializeRequest { id: 1 },
        MaterializeRequest { id: 2 },
        MaterializeRequest { id: 3 },
    ]);
    let request = tonic::Request::new(request_stream);
    let response = client.materialize(request).await?;
    let response = response.into_inner();
    let content = response
        .then(|res| async { res.unwrap() })
        .collect::<Vec<_>>()
        .await;
    assert_eq!(content.len(), 3);
    Ok(())
}
