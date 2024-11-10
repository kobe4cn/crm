use std::mem;

use anyhow::Result;

use crm::{AppConfig, CrmService};

use tonic::transport::{Identity, ServerTlsConfig};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let mut config = AppConfig::try_load()?;

    let addr = format!("[::]:{}", &config.server.port).parse().unwrap();
    info!("Starting CRM server at {}", addr);
    let tls = mem::take(&mut config.server.tls);
    let svc = CrmService::try_new(config).await?.into_server()?;
    if let Some(tls) = tls {
        let identity = Identity::from_pem(tls.cert, tls.key);
        tonic::transport::Server::builder()
            .tls_config(ServerTlsConfig::new().identity(identity))?
            .add_service(svc)
            .serve(addr)
            .await?;
    } else {
        tonic::transport::Server::builder()
            .add_service(svc)
            .serve(addr)
            .await?;
    }
    Ok(())
}
