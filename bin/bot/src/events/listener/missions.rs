use std::fmt::Write;

use async_trait::async_trait;
use serenity::{
    builder::{EditMessage, EditThread},
    gateway::ActivityData,
    model::id::UserId,
};
use synixe_events::missions::{db::Response, publish::Publish};
use synixe_meta::discord::channel::LOG;
use synixe_model::missions::Rsvp;
use synixe_proc::events_request_5;

use crate::{bot::Bot, cache_http::CacheAndHttp};

use super::Listener;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::StartingSoon { scheduled, minutes } => {
                let Some((channel, message)) = scheduled.message() else {
                    return Ok(());
                };
                if (-1..=1).contains(minutes) {
                    // Remove RSVP buttons
                    if let Err(e) = channel
                        .edit_message(
                            CacheAndHttp::get().as_ref(),
                            message,
                            EditMessage::default().components(vec![]),
                        )
                        .await
                    {
                        error!("Failed to edit message: {}", e);
                    }
                    let Some(mut thread) = channel
                        .message(CacheAndHttp::get().as_ref(), message)
                        .await?
                        .thread
                    else {
                        return Ok(());
                    };
                    thread
                        .edit_thread(
                            CacheAndHttp::get().as_ref(),
                            EditThread::default().locked(true).archived(true),
                        )
                        .await?;

                    // Send RSVP report to LOG
                    let Ok(Ok((Response::FetchMissionRsvps(Ok(rsvps)), _))) = events_request_5!(
                        bootstrap::NC::get().await,
                        synixe_events::missions::db,
                        FetchMissionRsvps {
                            scheduled: scheduled.id,
                        }
                    )
                    .await
                    else {
                        error!("failed to fetch mission rsvps");
                        return Ok(());
                    };

                    let mut report = String::new();
                    let mut yes = Vec::new();
                    for rsvp in rsvps {
                        if rsvp.state == Rsvp::Yes {
                            yes.push(UserId::new(
                                rsvp.member.parse().expect("only valid ids are stored"),
                            ));
                        } else {
                            writeln!(
                                report,
                                "<@{}>: {} ({})",
                                rsvp.member,
                                rsvp.state,
                                rsvp.details.unwrap_or_default(),
                            )?;
                        }
                    }

                    if report.is_empty() {
                        info!("no rsvps to report");
                        return Ok(());
                    }

                    log(format!("RSVP Report for {}: \n{}", scheduled.name, report)).await;

                    let Ok(Ok((Response::FetchMissionCounts(Ok(counts)), _))) = events_request_5!(
                        bootstrap::NC::get().await,
                        synixe_events::missions::db,
                        FetchMissionCounts { members: yes }
                    )
                    .await
                    else {
                        error!("failed to fetch mission rsvps");
                        return Ok(());
                    };
                    for (member, count) in counts {
                        // Include this mission
                        let count = count + 1;
                        match count {
                            1 => {
                                attending(member, "1st").await;
                            }
                            2 => {
                                attending(member, "2nd").await;
                            }
                            16 => {
                                attending(member, "16th").await;
                            }
                            50 => {
                                attending(member, "50th").await;
                            }
                            100 => {
                                attending(member, "100th").await;
                            }
                            200 => {
                                attending(member, "200th").await;
                            }
                            300 => {
                                attending(member, "300th").await;
                            }
                            _ => {}
                        }
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
        Bot::get().set_activity(Some(ActivityData::playing(mission.name)));
    } else {
        Bot::get().set_activity(None);
    }
}

async fn log(message: String) {
    if let Err(e) = LOG.say(CacheAndHttp::get().as_ref(), message).await {
        error!("failed to send log: {}", e);
    }
}

async fn attending(member: UserId, count: &str) {
    log(format!(
        "<@{member}> will be attending their {count} mission"
    ))
    .await;
}
