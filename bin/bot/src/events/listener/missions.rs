use async_trait::async_trait;
use serenity::model::prelude::Activity;
use synixe_events::missions::{db::Response, publish::Publish};
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
                let Some((channel, message)) = scheduled.message() else { return Ok(()) };
                if (-1..=1).contains(minutes) {
                    if let Err(e) = channel
                        .edit_message(&CacheAndHttp::get().http, message, |m| m.components(|f| f))
                        .await
                    {
                        error!("Failed to edit message: {}", e);
                    }
                    let Some(thread) = channel.message(&CacheAndHttp::get().http, message).await?.thread else {
                        return Ok(());
                    };
                    thread
                        .edit_thread(&CacheAndHttp::get().http, |t| t.locked(true).archived(true))
                        .await?;
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
