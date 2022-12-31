use async_trait::async_trait;
use serenity::model::prelude::MessageId;
use synixe_events::missions::publish::Publish;
use synixe_meta::discord::channel::SCHEDULE;

use crate::cache_http::Bot;

use super::Listener;

#[async_trait]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::StartingSoon { scheduled, minutes } => {
                let Some(ref message) = scheduled.schedule_message_id else { return Ok(()) };
                let Ok(message_id) = message.parse::<u64>() else {
                    error!("Failed to parse message id");
                    return Ok(());
                };
                if (-1..=1).contains(minutes) {
                    if let Err(e) = SCHEDULE
                        .edit_message(&Bot::get().http, MessageId(message_id), |m| {
                            m.components(|f| f);
                            m
                        })
                        .await
                    {
                        error!("Failed to edit message: {}", e);
                    }
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
}
