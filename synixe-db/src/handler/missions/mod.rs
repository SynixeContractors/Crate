use async_trait::async_trait;
use synixe_events::missions::db::{Request, Response};
use synixe_meta::missions::MISSION_LIST;
use synixe_model::missions::Mission;

use super::Handler;

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
            Self::UpdateMissionList {} => {
                if let Ok(response) = reqwest::get(MISSION_LIST).await {
                    match response.json::<Vec<Mission>>().await {
                        Ok(missions) => {
                            for mission in missions {
                                let (query, _span) = trace_query!(
                                    cx,
                                    r#"INSERT INTO missions_list (id, name, summary, description, type)
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
                                    mission.typ as i32,
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
        }
    }
}
