use std::mem;

use anyhow::Result;
use crm::{
    pb::{crm_client::CrmClient, WelcomeRequestBuilder},
    AppConfig,
};
use tonic::transport::{Certificate, Channel, ClientTlsConfig};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    // let layer = tracing_subscriber::fmt::Layer::new();
    let layer = Layer::new().with_filter(LevelFilter::INFO);

    tracing_subscriber::registry().with(layer).init();
    let mut config = AppConfig::try_load()?;
    let tls = mem::take(&mut config.server.tls);
    let addr = format!("https://[::]:{}", config.server.port.clone());
    info!("Starting CRM client at {}", addr);
    let request = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(90u32)
        .content_ids(vec![1, 2, 3])
        .build()?;
    if let Some(tls) = tls {
        //mkcert -CAROOT ca查询
        let ca = Certificate::from_pem(tls.ca);
        let tls = ClientTlsConfig::new().ca_certificate(ca).domain_name("::");
        let addr_static: &'static str = Box::leak(addr.into_boxed_str());
        let channel = Channel::from_static(addr_static)
            .tls_config(tls)?
            .connect()
            .await?;

        let mut svc = CrmClient::new(channel);
        let response = svc.welcome(request).await?.into_inner();
        println!("RESPONSE={:?}", response);
    } else {
        let mut svc = CrmClient::connect(addr).await?;
        let response = svc.welcome(request).await?.into_inner();
        println!("RESPONSE={:?}", response);
    }

    // let addr = "http://localhost:8080";

    //

    Ok(())
}
