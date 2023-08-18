use async_trait::async_trait;
use serenity::model::prelude::Activity;
use synixe_events::missions::{db::Response, publish::Publish};
use synixe_meta::discord::channel::LOG;
use synixe_model::missions::Rsvp;
use synixe_proc::events_request_5;

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
                    // Remove RSVP buttons
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

                    // Send RSVP report to LOG
                    let Ok(Ok((Response::FetchMissionRsvps(Ok(rsvps)), _))) = events_request_5!(
                        bootstrap::NC::get().await,
                        synixe_events::missions::db,
                        FetchMissionRsvps {
                            scheduled: scheduled.id,
                        }
                    ).await else {
                        error!("failed to fetch mission rsvps");
                        return Ok(());
                    };

                    let mut report = String::new();
                    for rsvp in rsvps {
                        if rsvp.state != Rsvp::Yes {
                            report.push_str(&format!(
                                "<@{}>: {} ({})\n",
                                rsvp.member,
                                rsvp.state,
                                rsvp.details.unwrap_or_default(),
                            ));
                        }
                    }

                    if report.is_empty() {
                        info!("no rsvps to report");
                        return Ok(());
                    }

                    if let Err(e) = LOG
                        .send_message(&CacheAndHttp::get().http, |m| {
                            m.content(format!("RSVP Report for {}: \n{}", scheduled.name, report))
                        })
                        .await
                    {
                        error!("failed to send RSVP report: {}", e);
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
        events_request_5!(nats, synixe_events::missions::db, FetchCurrentMission {}).await
    {
        Bot::get().set_activity(Some(Activity::playing(mission.name)));
    } else {
        Bot::get().set_activity(None);
    }
}
