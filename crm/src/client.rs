use anyhow::Result;
use crm::{
    pb::{crm_client::CrmClient, WelcomeRequestBuilder},
    AppConfig,
};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<()> {
    let _config = AppConfig::try_load()?;
    // let addr = format!("http://[::1]:{}", config.server.port.clone());
    let addr = "http://localhost:8080";
    println!("Connecting to CRM server at {}", addr);
    let mut svc = CrmClient::connect(addr).await?;
    let request = WelcomeRequestBuilder::default()
        .id(Uuid::new_v4().to_string())
        .interval(90u32)
        .content_ids(vec![1, 2, 3])
        .build()?;

    let response = svc.welcome(request).await?.into_inner();
    println!("RESPONSE={:?}", response);
    Ok(())
}
