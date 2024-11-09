use std::collections::HashSet;

use crate::{
    pb::{WelcomeRequest, WelcomeResponse},
    CrmService,
};
use anyhow::Result;
use chrono::{Duration, Utc};
use crm_metadata::{
    pb::{Content, MaterializeRequest},
    Tpl,
};
use futures::{StreamExt, TryStreamExt};
use notification::{EmailMessage, Msg, SendRequest};
use prost_types::Timestamp;

use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};
use tracing::warn;
use user_state::pb::{QueryRequestBuilder, TimeQuery, User};
use uuid::Uuid;

impl CrmService {
    pub async fn welcome(
        &self,
        request: WelcomeRequest,
    ) -> Result<Response<WelcomeResponse>, Status> {
        let request_id = request.id.clone();
        let date = Utc::now() - Duration::days(request.interval as _);
        //convert to timestamp
        let timestamp = Timestamp {
            seconds: date.timestamp(),
            nanos: date.timestamp_subsec_nanos() as _,
        };
        //build query request
        let query = QueryRequestBuilder::default()
            .timestamp((
                "created_at".to_string(),
                TimeQuery {
                    before: Some(timestamp),
                    after: Some(timestamp),
                },
            ))
            .build()
            .expect("Failed to build query");
        let user_res: Response<tonic::Streaming<user_state::pb::User>> =
            self.user_state.clone().query(query).await?;
        let mut user_stream = user_res.into_inner();

        //materialize request
        let content_ids = request.content_ids.clone();
        let metarequest: HashSet<_> = content_ids
            .iter()
            .map(|x| MaterializeRequest { id: *x })
            .collect();
        let request_stream = tokio_stream::iter(metarequest.into_iter());
        let request = Request::new(request_stream);

        let meta_res = self.metadata.clone().materialize(request).await?;
        let meta_stream = meta_res.into_inner();
        //stream to vec
        let contents = meta_stream
            .try_collect::<Vec<_>>()
            .await
            .map_err(|e| Status::internal(e.to_string()))?;
        // let contents = Arc::new(contents);
        let mut notification = self.notification.clone();

        // while let Some(Ok(user)) = user_stream.next().await {
        //     let req = gen_send_request(
        //         "Welcome".to_string(),
        //         self.config.server.sender_email.clone(),
        //         user,
        //         &contents,
        //     );
        //     let req = tokio_stream::iter(vec![req]);
        //     notification.send(req).await?;
        // }

        let (tx, rx) = mpsc::channel(1024);
        let sender_email = self.config.server.sender_email.clone();
        tokio::spawn(async move {
            while let Some(Ok(user)) = user_stream.next().await {
                let req =
                    gen_send_request("Welcome".to_string(), sender_email.clone(), user, &contents);
                if let Err(e) = tx.send(req).await {
                    warn!("Failed to send message: {:?}", e);
                };
            }
        });
        let req = ReceiverStream::new(rx);
        notification.send(req).await?;

        let ret = WelcomeResponse { id: request_id };
        Ok(Response::new(ret))
    }
}

fn gen_send_request(
    subject: String,
    sender: String,
    user: User,
    content: &[Content],
) -> SendRequest {
    let tpl = Tpl(content);
    SendRequest {
        msg: Some(Msg::Email(EmailMessage {
            message_id: Uuid::new_v4().to_string(),
            sender,
            recipients: user.email,
            subject,
            body: tpl.to_body(),
        })),
    }
}
