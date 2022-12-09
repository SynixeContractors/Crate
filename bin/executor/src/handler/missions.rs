use async_trait::async_trait;
use synixe_events::{
    discord::write::{DiscordContent, DiscordMessage},
    missions::{
        db,
        executions::{Request, Response},
    },
    publish, respond,
};
use synixe_meta::discord::channel::ONTOPIC;
use synixe_proc::events_request;

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
        _cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::PostUpcomingMissions {} => {
                respond!(msg, Response::PostUpcomingMissions(Ok(()))).await?;
                match events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    UpcomingSchedule {}
                )
                .await
                {
                    Ok(((db::Response::UpcomingSchedule(Ok(missions)), _), _)) => {
                        let now = time::OffsetDateTime::now_utc();
                        let mut posts = Vec::new();
                        for mission in missions {
                            let num_minutes = (mission.start - now).whole_minutes() + 1;
                            match num_minutes {
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
                            if let Ok(((db::Response::FetchMission(Ok(Some(mission))), _), _)) =
                                events_request!(
                                    bootstrap::NC::get().await,
                                    synixe_events::missions::db,
                                    FetchMission {
                                        mission: scheduled.mission.clone()
                                    }
                                )
                                .await
                            {
                                if let Some(text) = text {
                                    if let Err(e) = events_request!(
                                        bootstrap::NC::get().await,
                                        synixe_events::discord::write,
                                        ChannelMessage {
                                            channel: ONTOPIC,
                                            message: DiscordMessage {
                                                content: DiscordContent::Text(format!(
                                                    "**{}** starts in {text}",
                                                    mission.name
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
                                } else if let Err(e) = publish!(
                                    bootstrap::NC::get().await,
                                    match minutes {
                                        // Warn the mission will change 80 minutes before it starts
                                        78..=82 => synixe_events::missions::publish::Publish::WarnChangeMission {
                                            id: scheduled.mission.clone(),
                                            mission_type: mission.typ
                                        },
                                        // Change the mission 70 minutes before it starts
                                        68..=72 => synixe_events::missions::publish::Publish::ChangeMission {
                                            id: scheduled.mission.clone(),
                                            mission_type: mission.typ
                                        },
                                        _ => unreachable!(),
                                    }
                                )
                                .await
                                {
                                    error!("Failed to publish discord message: {}", e);
                                }
                                if let Err(e) = publish!(
                                    bootstrap::NC::get().await,
                                    synixe_events::missions::publish::Publish::StartingSoon {
                                        mission,
                                        scheduled,
                                        minutes,
                                    }
                                )
                                .await
                                {
                                    error!("Failed to publish discord message: {}", e);
                                }
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
                        respond!(msg, Response::PostUpcomingMissions(Err(e.to_string())))
                            .await
                            .map_err(std::convert::Into::into)
                    }
                }
            }
        }
    }
}
