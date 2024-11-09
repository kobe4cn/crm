use anyhow::Result;
use crm::{
    pb::{crm_client::CrmClient, WelcomeRequestBuilder},
    AppConfig,
};

#[tokio::main]
async fn main() -> Result<()> {
    let config = AppConfig::try_load()?;
    let addr = format!("http://[::1]:{}", config.server.port.clone());
    println!("Connecting to CRM server at {}", addr);
    let mut svc = CrmClient::connect(addr).await?;
    let request = WelcomeRequestBuilder::default()
        .id("1")
        .interval(90u32)
        .content_ids(vec![1, 2, 3])
        .build()?;

    let response = svc.welcome(request).await?.into_inner();
    println!("RESPONSE={:?}", response);
    Ok(())
}
