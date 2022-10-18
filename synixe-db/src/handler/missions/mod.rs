use async_trait::async_trait;
use synixe_events::missions::db::{Request, Response};
use synixe_meta::missions::MISSION_LIST;
use synixe_model::missions::{Mission, MissionType};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
        cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::Schedule { mission, date } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::Schedule,
                    "INSERT INTO missions_schedule (mission, start_at) VALUES ($1, $2)",
                    mission,
                    date,
                )
            }
            Self::IsScheduled { date } => {
                fetch_one_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::IsScheduled,
                    "SELECT EXISTS(SELECT 1 FROM missions_schedule WHERE start_at = $1) as value",
                    date,
                )
            }
            Self::Unschedule { scheduled_mission } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::Unschedule,
                    "DELETE FROM missions_schedule WHERE id = $1",
                    scheduled_mission,
                )
            }
            Self::SetScheduledMesssage {
                scheduled_mission,
                schedule_message_id,
                planning_message_id,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::SetScheduledMesssage,
                    "UPDATE missions_schedule SET schedule_message_id = $1, planning_message_id = $2 WHERE id = $3",
                    schedule_message_id,
                    planning_message_id,
                    scheduled_mission,
                )
            }
            Self::UpcomingSchedule {} => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::missions::ScheduledMission,
                    Response::UpcomingSchedule,
                    "SELECT * FROM missions_schedule WHERE start_at > NOW() ORDER BY start_at ASC",
                )
            }
            Self::UpdateMissionList {} => {
                if let Ok(response) = reqwest::get(MISSION_LIST).await {
                    match response.json::<Vec<Mission>>().await {
                        Ok(missions) => {
                            for mission in missions {
                                let (query, _span) = trace_query!(
                                    cx,
                                    r#"INSERT INTO missions (id, name, summary, description, type)
                                        VALUES ($1, $2, $3, $4, $5)
                                        ON CONFLICT (id) DO UPDATE SET
                                            name = $2,
                                            summary = $3,
                                            description = $4,
                                            type = $5"#,
                                    mission.id,
                                    mission.name,
                                    mission.summary,
                                    mission.description,
                                    mission.typ as MissionType,
                                );
                                if let Err(e) = query.execute(&*db).await {
                                    error!("{:?}", e);
                                    synixe_events::respond!(
                                        msg,
                                        Response::UpdateMissionList(Err(e.to_string()))
                                    )
                                    .await?;
                                }
                            }
                            synixe_events::respond!(msg, Response::UpdateMissionList(Ok(())))
                                .await
                                .map_err(std::convert::Into::into)
                        }
                        Err(e) => {
                            error!("Failed to parse mission list: {}", e);
                            synixe_events::respond!(
                                msg,
                                Response::UpdateMissionList(Err(e.to_string()))
                            )
                            .await
                            .map_err(std::convert::Into::into)
                        }
                    }
                } else {
                    synixe_events::respond!(
                        msg,
                        Response::UpdateMissionList(
                            Err("Failed to fetch mission list".to_string())
                        )
                    )
                    .await
                    .map_err(std::convert::Into::into)
                }
            }
            Self::FetchMissionList {} => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::missions::Mission,
                    Response::FetchMissionList,
                    "SELECT id, name, summary, description, type as \"typ: MissionType\" FROM missions ORDER BY name ASC",
                )
            }
            Self::FetchMission { mission } => {
                fetch_one_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::missions::Mission,
                    Response::FetchMission,
                    "SELECT id, name, summary, description, type as \"typ: MissionType\" FROM missions WHERE id = $1",
                    mission,
                )
            }
        }
    }
}
