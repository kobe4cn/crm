use chrono::Utc;
use prost_types::Timestamp;

use std::{net::SocketAddr, time::Duration};
use tokio::time::sleep;

use anyhow::Result;

use futures::StreamExt as _;
use tonic::transport::Server;
use user_state::{
    pb::{
        user_stats_client::UserStatsClient, IdQuery, QueryRequestBuilder, RawQueryRequestBuilder,
        TimeQuery,
    },
    AppConfig, UserStatsService,
};

#[tokio::test]
async fn raw_query_should_work() -> Result<()> {
    let addr = start_server(50058).await?;
    //addr to url
    let addr = format!("http://{}", addr);
    println!("addr: {}", addr);
    let mut client = UserStatsClient::connect(addr).await?;
    let query = RawQueryRequestBuilder::default()
        .query("SELECT * FROM user_stats limit 50")
        .build()?;
    let stream = client.raw_query(query).await?.into_inner();
    let ret = stream
        .then(|response| async move { response.unwrap() })
        .collect::<Vec<_>>()
        .await;
    assert_eq!(ret.len(), 50);
    Ok(())
}

async fn start_server(port: u32) -> Result<SocketAddr> {
    //rand generate the port

    let config = AppConfig::try_load()?;
    let addr = format!("[::1]:{}", port).parse()?;
    let svc = UserStatsService::new(config).await.unwrap().into_server();
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
async fn query_should_work() -> Result<()> {
    let addr = start_server(50057).await?;
    let addr = format!("http://{}", addr);
    let mut client = UserStatsClient::connect(addr).await?;
    let query = QueryRequestBuilder::default()
        .timestamp(("created_at".to_string(), tq(Some(20), None)))
        .timestamp(("last_visited_at".to_string(), tq(Some(20), None)))
        .id(("viewed_but_not_started".to_string(), to_ids(&[202371])))
        .build()?;
    let stream = client.query(query).await?.into_inner();
    let ret = stream
        .then(|response| async move { response.unwrap() })
        .collect::<Vec<_>>()
        .await;
    assert!(!ret.is_empty());
    Ok(())
}

fn to_ids(ids: &[u32]) -> IdQuery {
    IdQuery { ids: ids.to_vec() }
}
fn tq(before: Option<i64>, after: Option<i64>) -> TimeQuery {
    TimeQuery {
        before: before.map(to_ts),
        after: after.map(to_ts),
    }
}

fn to_ts(days: i64) -> Timestamp {
    let now = Utc::now();
    let ts = now - chrono::Duration::days(days);
    Timestamp {
        seconds: ts.timestamp(),
        nanos: ts.timestamp_subsec_nanos() as _,
    }
}
