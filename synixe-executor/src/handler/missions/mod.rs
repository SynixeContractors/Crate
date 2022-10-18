use async_trait::async_trait;
use synixe_events::{
    discord::write::{DiscordContent, DiscordMessage},
    missions::{
        db,
        executions::{Request, Response},
    },
    respond,
};
use synixe_meta::discord::channel::ONTOPIC;
use synixe_proc::events_request;

use super::Handler;

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
                match events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::missions::db,
                    UpcomingSchedule {}
                )
                .await
                {
                    Ok(((db::Response::UpcomingSchedule(Ok(missions)), _), _)) => {
                        let now = chrono::Utc::now().naive_utc();
                        let mut posts = Vec::new();
                        for mission in missions {
                            match mission.start_at.signed_duration_since(now).num_minutes() {
                                178..=182 => {
                                    posts.push(("3 hours!", mission));
                                }
                                58..=62 => {
                                    posts.push(("1 hour!", mission));
                                }
                                28..=32 => {
                                    posts.push(("30 minutes!", mission));
                                }
                                8..=12 => {
                                    posts.push(("10 minutes!", mission));
                                }
                                3..=7 => {
                                    posts.push(("5 minutes!", mission));
                                }
                                _ => {}
                            }
                        }
                        for (text, mission) in posts {
                            if let Ok((
                                (db::Response::FetchMission(Ok(Some(mission_data))), _),
                                _,
                            )) = events_request!(
                                bootstrap::NC::get().await,
                                synixe_events::missions::db,
                                FetchMission {
                                    mission: mission.mission.clone()
                                }
                            )
                            .await
                            {
                                if let Err(e) = events_request!(
                                    bootstrap::NC::get().await,
                                    synixe_events::discord::write,
                                    ChannelMessage {
                                        channel: ONTOPIC,
                                        message: DiscordMessage {
                                            content: DiscordContent::Text(format!(
                                                "**{}** starts in {}",
                                                mission_data.name, text
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
                        }
                        respond!(msg, Response::PostUpcomingMissions(Ok(())))
                            .await
                            .map_err(std::convert::Into::into)
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
