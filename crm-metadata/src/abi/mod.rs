use chrono::{DateTime, Days, Utc};
use fake::{
    faker::{lorem::zh_cn::Sentence, name::zh_cn::Name},
    Fake, Faker,
};
use futures::{Stream, StreamExt};
use prost_types::Timestamp;
use rand::Rng;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Response, Status};

use crate::{
    pb::{Content, MaterializeRequest, Publisher},
    MetaDataService, ResponseStream, ServiceResult,
};

impl MetaDataService {
    pub async fn materialize(
        &self,
        mut stream: impl Stream<Item = Result<MaterializeRequest, Status>> + Send + 'static + Unpin,
    ) -> ServiceResult<ResponseStream> {
        //使用mpsc通过发送数据，tx负责往通道发送数据，ReceiverStream::new 将rx进行包装之后再使用Box::pin进行包装，返回客户端。
        //客户段 stream.next().await Ok(rx) 获取到rx，然后进行数据的接收
        let (tx, rx) = tokio::sync::mpsc::channel(1024);
        tokio::spawn(async move {
            while let Some(req) = stream.next().await {
                match req {
                    Ok(req) => {
                        let content = Content::materialize(req.id);
                        tx.send(Ok(content)).await.unwrap()
                    }
                    Err(e) => tx
                        .send(Err(Status::internal(format!(
                            "Failed to fetch data with query {}",
                            e
                        ))))
                        .await
                        .unwrap(),
                }
            }
        });

        let stream = ReceiverStream::new(rx);
        Ok(Response::new(Box::pin(stream)))
    }
}

impl Content {
    pub fn materialize(id: u32) -> Self {
        let mut rng = rand::thread_rng();
        Content {
            id,
            name: Name().fake(),
            description: Sentence(3..20).fake(),
            created_at: create_at(),
            publishers: (1..rng.gen_range(2..10))
                .map(|_| Publisher::new())
                .collect(),
            url: "https://placehold.co/400x400".to_string(),
            images: "https://placehold.co/400x400".to_string(),
            content_type: Faker.fake(),
            views: rng.gen_range(123412..1000000000),
            likes: rng.gen_range(123333..12313213213),
            dislikes: rng.gen_range(121313..131212121121),
        }
    }
}
impl Publisher {
    pub fn new() -> Self {
        Publisher {
            id: (10000..200000).fake(),
            name: Name().fake(),
            avatar: "https://placehold.co/400x400".to_string(),
        }
    }
}
fn before(days: u64) -> DateTime<Utc> {
    Utc::now().checked_sub_days(Days::new(days)).unwrap()
}

fn create_at() -> Option<Timestamp> {
    Some(Timestamp {
        seconds: before(100).timestamp(),
        nanos: 0,
    })
}

#[cfg(test)]
mod tests {
    use anyhow::Result;

    use crate::AppConfig;

    use super::*;

    #[tokio::test]
    async fn test_materialize() -> Result<()> {
        let service = MetaDataService::new(AppConfig::try_load()?);

        let request_stream = tokio_stream::iter(vec![
            Ok(MaterializeRequest { id: 1 }),
            Ok(MaterializeRequest { id: 2 }),
            Ok(MaterializeRequest { id: 3 }),
        ]);

        let response = service.materialize(request_stream).await.unwrap();
        let response = response.into_inner();
        let content = response.collect::<Vec<_>>().await;
        assert_eq!(content.len(), 3);
        Ok(())
    }
}
