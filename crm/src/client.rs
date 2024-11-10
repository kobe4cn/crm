use std::mem;

use anyhow::Result;
use crm::{
    pb::{crm_client::CrmClient, WelcomeRequestBuilder},
    AppConfig,
};
use tonic::{
    metadata::MetadataValue,
    transport::{Certificate, Channel, ClientTlsConfig},
    Request,
};
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

        let token: MetadataValue<_> = "eyJhbGciOiJFZERTQSIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE3MzEyMjE0NjYsImV4cCI6MTczMzgxMzQ2NiwibmJmIjoxNzMxMjIxNDY2LCJpc3MiOiJjaGF0X3NlcnZlciIsImF1ZCI6ImNoYXRfd2ViIiwiaWQiOjEsIndzX2lkIjoxLCJmdWxsbmFtZSI6ImtldmluIHlhbmciLCJlbWFpbCI6ImtldmluLnlhbmcueGd6QGdtYWlsLmNvbSIsImNyZWF0ZWRfYXQiOiIyMDI0LTExLTEwVDA2OjUxOjA2LjQyMjM5MloifQ.KKTXNplpkU84MAZT_8KpTk8gv-gafEoAVt2berf4QA-9bDXeTq_-4GppYJuOFoMLGUB2UpWSdydHgoFnbxA4Bw".parse()?;
        let mut svc = CrmClient::with_interceptor(channel, move |mut req: Request<()>| {
            req.metadata_mut().insert("authorization", token.clone());
            Ok(req)
        });
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
