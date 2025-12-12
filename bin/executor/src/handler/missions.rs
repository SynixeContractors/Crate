use async_trait::async_trait;
use synixe_events::{
    discord::write::{DiscordContent, DiscordMessage},
    missions::{
        db,
        executions::{Request, Response},
    },
    publish, respond,
};
use synixe_meta::discord::{
    GUILD,
    channel::{LEADERSHIP, ONTOPIC},
};
use synixe_model::missions::Rsvp;
use synixe_proc::events_request_5;

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::PostUpcomingMissions {} => {
                respond!(msg, Response::PostUpcomingMissions(Ok(()))).await?;
                match events_request_5!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    UpcomingSchedule {}
                )
                .await
                {
                    Ok(Ok((db::Response::UpcomingSchedule(Ok(missions)), _))) => {
                        let now = time::OffsetDateTime::now_utc();
                        let mut posts = Vec::new();
                        for mission in missions {
                            let num_minutes = (mission.start - now).whole_minutes() + 1;
                            match num_minutes {
                                1438..=1442 => {
                                    if let Err(e) = events_request_5!(
                                        bootstrap::NC::get().await,
                                        synixe_events::discord::write,
                                        ChannelMessage {
                                            channel: LEADERSHIP,
                                            message: DiscordMessage {
                                                content: DiscordContent::Text(format!(
                                                    "**{}** starts in 24 hours. Remember to pick an Operation Lead if one has not been decided already{}",
                                                    mission.name,
                                                    {
                                                        if let Some((channel, message)) = mission.message() {
                                                            format!("\n\nhttps://discord.com/channels/{GUILD}/{channel}/{message}")
                                                        } else {
                                                            String::new()
                                                        }
                                                    },
                                                )),
                                                reactions: Vec::new(),
                                            },
                                            thread: None,
                                        }
                                    )
                                    .await
                                    {
                                        error!("error posting to reddit: {:?}", e);
                                    }
                                }

                                178..=182 => {
                                    posts.push((Some("3 hours!"), mission, num_minutes));
                                }
                                78..=82 | 68..=72 | -1..=2 => {
                                    posts.push((None, mission, num_minutes));
                                }
                                58..=62 => {
                                    posts.push((Some("1 hour!"), mission, num_minutes));
                                }
                                28..=32 => {
                                    posts.push((Some("30 minutes!"), mission, num_minutes));
                                }
                                8..=12 => {
                                    posts.push((Some("10 minutes!"), mission, num_minutes));
                                }
                                3..=7 => {
                                    posts.push((Some("5 minutes!"), mission, num_minutes));
                                }
                                _ => {}
                            }
                        }
                        for (text, scheduled, minutes) in posts {
                            let (pings, ping_message) = if [180].contains(&minutes) {
                                if let Ok(Ok((
                                    synixe_events::missions::db::Response::FetchMissionRsvps(Ok(
                                        rsvps,
                                    )),
                                    _,
                                ))) = events_request_5!(
                                    bootstrap::NC::get().await,
                                    synixe_events::missions::db,
                                    FetchMissionRsvps {
                                        scheduled: scheduled.id,
                                    }
                                )
                                .await
                                {
                                    (
                                        rsvps
                                            .iter()
                                            .filter_map(|rsvp| {
                                                if rsvp.state == Rsvp::Maybe {
                                                    Some(rsvp.member.clone())
                                                } else {
                                                    None
                                                }
                                            })
                                            .collect(),
                                        String::from("Update your RSVP!\n"),
                                    )
                                } else {
                                    (Vec::new(), String::new())
                                }
                            } else {
                                (Vec::new(), String::new())
                            };
                            if let Some(text) = text {
                                if let Err(e) = events_request_5!(
                                    bootstrap::NC::get().await,
                                    synixe_events::discord::write,
                                    ChannelMessage {
                                        channel: ONTOPIC,
                                        message: DiscordMessage {
                                            content: DiscordContent::Text(format!(
                                                "**{}** starts in {text}{}{}",
                                                scheduled.name,
                                                {
                                                    if let Some((channel, message)) = scheduled.message() {
                                                        format!("\n\nhttps://discord.com/channels/{GUILD}/{channel}/{message}")
                                                    } else {
                                                        String::new()
                                                    }
                                                },
                                                if pings.is_empty() {
                                                    String::new()
                                                } else {
                                                    format!("\n\n{}{}", ping_message, pings.iter().map(|p| format!("<@{p}>")).collect::<Vec<_>>().join(" "))
                                                }
                                            )),
                                            reactions: Vec::new(),
                                        },
                                        thread: None,
                                    }
                                )
                                .await
                                {
                                    error!("error posting to reddit: {:?}", e);
                                }
                            } else if let Some(event) = match minutes {
                                // Warn the mission will change 80 minutes before it starts
                                78..=82 => {
                                    if scheduled.mission.starts_with('$') {
                                        None
                                    } else {
                                        Some(synixe_events::missions::publish::Publish::WarnChangeMission {
                                            id: scheduled.mission.clone(),
                                            mission_type: scheduled.typ,
                                        })
                                    }
                                }
                                // Change the mission 70 minutes before it starts
                                68..=72 => {
                                    if scheduled.mission.starts_with('$') {
                                        None
                                    } else {
                                        Some(synixe_events::missions::publish::Publish::ChangeMission {
                                            id: scheduled.mission.clone(),
                                            mission_type: scheduled.typ,
                                            reason: "Scheduled".to_string(),
                                        })
                                    }
                                }
                                _ => None,
                            } {
                                let nats = bootstrap::NC::get().await;
                                if let Err(e) = publish!(nats, event).await {
                                    error!("Failed to publish discord message: {}", e);
                                }
                            }
                            let nats = bootstrap::NC::get().await;
                            if let Err(e) = publish!(
                                nats,
                                synixe_events::missions::publish::Publish::StartingSoon {
                                    scheduled,
                                    minutes,
                                }
                            )
                            .await
                            {
                                error!("Failed to publish discord message: {}", e);
                            }
                        }
                        Ok(())
                    }
                    Ok(_) => {
                        error!("unexpected response from db");
                        respond!(
                            msg,
                            Response::PostUpcomingMissions(Err(String::from(
                                "unexpected response from db"
                            )))
                        )
                        .await
                        .map_err(std::convert::Into::into)
                    }
                    Err(e) => {
                        error!("error getting upcoming missions: {}", e);
                        respond!(msg, Response::PostUpcomingMissions(Err(e.clone())))
                            .await
                            .map_err(std::convert::Into::into)
                    }
                }
            }
        }
    }
}
