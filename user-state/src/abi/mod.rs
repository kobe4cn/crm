use chrono::{DateTime, TimeZone, Utc};
use futures::stream;
use prost_types::Timestamp;
use tonic::{Response, Status};

use crate::{
    pb::{QueryRequest, RawQueryRequest, User},
    ResponseStream, ServiceResult, UserStatsService,
};

impl UserStatsService {
    pub async fn query(&self, query: QueryRequest) -> ServiceResult<ResponseStream> {
        let mut sql = "select email,name from user_stats where 1=1 ".to_string();
        let time_conditions = query
            .timestamps
            .into_iter()
            .map(|(k, v)| timestamp_query(&k, v.before, v.after))
            .collect::<Vec<_>>()
            .join(" ");
        sql.push_str(&time_conditions);
        let id_conditions = query
            .ids
            .into_iter()
            .map(|(k, v)| ids_query(&k, v.ids))
            .collect::<Vec<_>>()
            .join(" ");
        sql.push_str(&id_conditions);

        // Implement your logic here
        self.raw_query(RawQueryRequest { query: sql }).await
    }

    pub async fn raw_query(&self, req: RawQueryRequest) -> ServiceResult<ResponseStream> {
        let Ok(ret) = sqlx::query_as::<_, User>(&req.query)
            .fetch_all(&self.pool)
            .await
        else {
            return Err(Status::internal(format!(
                "Failed to fetch data with query {}",
                req.query
            )));
        };

        // Implement your logic here
        Ok(Response::new(Box::pin(stream::iter(
            ret.into_iter().map(Ok),
        ))))
    }
}

fn ids_query(name: &str, ids: Vec<u32>) -> String {
    if ids.is_empty() {
        return "".to_string();
    }
    format!(" and array{:?} <@ {}", ids, name)
}
fn timestamp_query(name: &str, before: Option<Timestamp>, after: Option<Timestamp>) -> String {
    if before.is_none() && after.is_none() {
        return "".to_string();
    }
    if before.is_none() {
        let after = ts_to_utc(after.unwrap());
        return format!(" and {} <= '{}'", name, after);
    }
    if after.is_none() {
        let before = ts_to_utc(before.unwrap());
        return format!(" and {} >= '{}'", name, before);
    }
    let before = ts_to_utc(before.unwrap());
    let after = ts_to_utc(after.unwrap());
    format!(" and {} between '{}' and '{}'", name, before, after)
}

fn ts_to_utc(ts: Timestamp) -> DateTime<Utc> {
    Utc.timestamp_opt(ts.seconds, ts.nanos as _).unwrap()
}

// #[cfg(test)]
// #[allow(unused)]
// mod tests {
//     use crate::{
//         pb::{IdQuery, QueryRequestBuilder, TimeQuery},
//         AppConfig,
//     };

//     use super::*;
//     use anyhow::{Context, Result};
//     use stream::StreamExt;

//     #[tokio::test]
//     async fn raw_query_should_work() -> Result<()> {
//         let config = AppConfig::try_load().context("Failed to load config")?;
//         let svc = UserStatsService::new(config).await?;
//         let mut stream = svc
//             .raw_query(RawQueryRequest {
//                 query:
//                     "select email, name from user_stats where email='laron.ao5ta8kb@example.net'"
//                         .to_string(),
//             })
//             .await?
//             .into_inner();

//         // let ret = stream.next().await.context("Failed to fetch data")?;
//         // let user = ret?;
//         // assert_eq!(user.email, "laron.ao5ta8kb@example.net");
//         // assert_eq!(user.name, "贺修永");
//         // assert!(stream.next().await.is_none());
//         while let Some(ret) = stream.next().await {
//             let user = ret?;
//             eprintln!(" {:?}", user);
//         }
//         Ok(())
//     }

//     #[tokio::test]
//     async fn query_should_work() -> Result<()> {
//         println!("query_should_work");
//         let config = AppConfig::try_load().context("Failed to load config")?;
//         let svc = UserStatsService::new(config).await?;
//         let query = QueryRequestBuilder::default()
//             .timestamp(("created_at".to_string(), tq(Some(20), None)))
//             .timestamp(("last_visited_at".to_string(), tq(Some(20), None)))
//             .id(("viewed_but_not_started".to_string(), to_ids(&[202371])))
//             .build()?;

//         // let mut stream = svc
//         //     .query(query)
//         //     .await;
//         let mut stream = svc.query(query).await?.into_inner();
//         while let Some(ret) = stream.next().await {
//             let user = ret?;
//             eprintln!(" {:?}", user);
//         }

//         Ok(())
//     }
//     fn to_ids(ids: &[u32]) -> IdQuery {
//         IdQuery { ids: ids.to_vec() }
//     }
//     fn tq(before: Option<i64>, after: Option<i64>) -> TimeQuery {
//         TimeQuery {
//             before: before.map(to_ts),
//             after: after.map(to_ts),
//         }
//     }
//     fn to_ts(days: i64) -> Timestamp {
//         let now = Utc::now();
//         let ts = now - chrono::Duration::days(days);
//         Timestamp {
//             seconds: ts.timestamp(),
//             nanos: ts.timestamp_subsec_nanos() as _,
//         }
//     }
// }
