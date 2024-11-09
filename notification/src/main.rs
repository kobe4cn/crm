use anyhow::{Context as _, Result};
use notification::{AppConfig, NotificationService};
use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};
use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt as _, Layer as _,
};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = AppConfig::try_load().context("load config failed")?;
    let addr = format!("[::1]:{}", &config.server.port).parse().unwrap();
    let svc = NotificationService::new(config).into_server();
    info!("Notification Service listening on {}", addr);
    Server::builder().add_service(svc).serve(addr).await?;
    Ok(())
}
