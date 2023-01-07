use async_trait::async_trait;
use serenity::model::prelude::{Activity, MessageId};
use synixe_events::missions::{db::Response, publish::Publish};
use synixe_meta::discord::channel::SCHEDULE;
use synixe_proc::events_request;

use crate::{bot::Bot, cache_http::CacheAndHttp};

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
                        .edit_message(&CacheAndHttp::get().http, MessageId(message_id), |m| {
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

pub async fn tick(nats: std::sync::Arc<nats::asynk::Connection>) {
    if let Ok(Ok((Response::FetchCurrentMission(Ok(Some(mission))), _))) =
        events_request!(nats, synixe_events::missions::db, FetchCurrentMission {}).await
    {
        Bot::get().set_activity(Some(Activity::playing(mission.name)));
    } else {
        Bot::get().set_activity(None);
    }
}
