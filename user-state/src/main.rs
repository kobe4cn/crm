use anyhow::{Context, Result};

use tonic::transport::Server;
use tracing::{info, level_filters::LevelFilter};

use tracing_subscriber::{
    fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt as _, Layer as _,
};
use user_state::{AppConfig, UserStatsService};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();
    let config = AppConfig::try_load().context("load config failed")?;
    let addr: std::net::SocketAddr = format!("[::1]:{}", &config.server.port).parse().unwrap();
    let service = UserStatsService::new(config).await?.into_server();

    info!("UserStatsService listening on {}", addr);
    Server::builder().add_service(service).serve(addr).await?;
    Ok(())
}
