use anyhow::Result;

use crm::{AppConfig, CrmService};
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt as _, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = AppConfig::try_load()?;

    let addr = format!("[::1]:{}", &config.server.port).parse().unwrap();
    info!("Starting CRM server at {}", addr);

    let svc = CrmService::try_new(config).await?.into_server();
    tonic::transport::Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?;
    Ok(())
}
