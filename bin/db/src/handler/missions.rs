use async_trait::async_trait;
use sqlx::types::time::Time;
use synixe_events::missions::db::{Request, Response};
use synixe_meta::missions::MISSION_LIST;
use synixe_model::missions::{Listing, MissionType, Rsvp, ScheduledMission};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::Schedule { mission, date } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::Schedule,
                    "INSERT INTO missions_schedule (mission, start) VALUES ($1, $2)",
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
                    "SELECT EXISTS(SELECT 1 FROM missions_schedule WHERE start = $1) AS value",
                    date,
                )
            }
            Self::Unschedule { scheduled } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::Unschedule,
                    "DELETE FROM missions_schedule WHERE id = $1",
                    scheduled,
                )
            }
            Self::SetScheduledMesssage {
                scheduled,
                message_id,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::SetScheduledMesssage,
                    "UPDATE missions_schedule SET schedule_message_id = $1 WHERE id = $2",
                    message_id.to_string(),
                    scheduled,
                )
            }
            Self::FetchScheduledMessage { message } => {
                fetch_one_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::missions::ScheduledMission,
                    Response::FetchScheduledMessage,
                    "SELECT
                        s.id,
                        s.mission,
                        s.schedule_message_id,
                        s.start,
                        m.name,
                        m.summary,
                        m.description,
                        m.type as \"typ: MissionType\"
                    FROM
                        missions_schedule s
                    INNER JOIN
                        missions m ON m.id = s.mission
                    WHERE schedule_message_id = $1",
                    message.to_string(),
                )?;
                Ok(())
            }
            Self::SetScheduledAar {
                scheduled,
                message_id,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::SetScheduledAar,
                    "UPDATE missions_schedule SET aar_message_id = $1 WHERE id = $2",
                    message_id.to_string(),
                    scheduled,
                )
            }
            Self::FetchScheduledAar { message } => {
                fetch_one_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::missions::ScheduledMission,
                    Response::FetchScheduledAar,
                    "SELECT
                        s.id,
                        s.mission,
                        s.schedule_message_id,
                        s.start,
                        m.name,
                        m.summary,
                        m.description,
                        m.type as \"typ: MissionType\"
                    FROM
                        missions_schedule s
                    INNER JOIN
                        missions m ON m.id = s.mission
                    WHERE aar_message_id = $1",
                    message.to_string(),
                )?;
                Ok(())
            }
            Self::UpcomingSchedule {} => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::missions::ScheduledMission,
                    Response::UpcomingSchedule,
                    "SELECT
                        s.id,
                        s.mission,
                        s.schedule_message_id,
                        s.start,
                        m.name,
                        m.summary,
                        m.description,
                        m.type as \"typ: MissionType\"
                    FROM
                        missions_schedule s
                    INNER JOIN
                        missions m ON m.id = s.mission
                    WHERE
                        start + '2 minutes'::Interval > NOW() ORDER BY start ASC",
                )?;
                Ok(())
            }
            Self::FindScheduledDate { mission, date } => {
                let date = date
                    .with_time(Time::from_hms(0, 0, 0).unwrap())
                    .assume_utc();
                fetch_one_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::missions::ScheduledMission,
                    Response::FindScheduledDate,
                    "SELECT
                        s.id,
                        s.mission,
                        s.schedule_message_id,
                        s.start,
                        m.name,
                        m.summary,
                        m.description,
                        m.type as \"typ: MissionType\"
                    FROM
                        missions_schedule s
                    INNER JOIN
                        missions m ON m.id = s.mission
                    WHERE
                        LOWER(m.name) = LOWER($1) AND
                        (start > $2 and start < $2 + '2 Day'::INTERVAL)",
                    mission,
                    date,
                )?;
                Ok(())
            }
            Self::PayMission {
                scheduled,
                contractors,
                contractor_amount,
                group_amount,
            } => {
                let mut tx = db.begin().await?;
                let scheduled: ScheduledMission = sqlx::query_as!(
                    ScheduledMission,
                    "SELECT
                        s.id,
                        s.mission,
                        s.schedule_message_id,
                        s.start,
                        m.name,
                        m.summary,
                        m.description,
                        m.type as \"typ: MissionType\"
                    FROM
                        missions_schedule s
                    INNER JOIN
                        missions m ON m.id = s.mission
                    WHERE
                        s.id = $1",
                    scheduled,
                )
                .fetch_one(&mut tx)
                .await?;
                let end = scheduled.start + time::Duration::hours(2);
                for contractor in contractors {
                    sqlx::query!(
                        "INSERT INTO gear_bank_deposits (member, amount, reason, id, created) VALUES ($1, $2, $3, $4, $5)",
                        contractor.to_string(),
                        contractor_amount,
                        format!("{}: {}", scheduled.typ.to_string(), scheduled.name),
                        scheduled.id,
                        end,
                    )
                    .execute(&mut tx)
                    .await?;
                }
                sqlx::query!(
                    "INSERT INTO gear_bank_deposits (member, amount, reason, id, created) VALUES ('0', $1, $2, $3, $4)",
                    group_amount,
                    format!("{}: {}", scheduled.typ.to_string(), scheduled.name),
                    scheduled.id,
                    end,
                )
                .execute(&mut tx)
                .await?;
                tx.commit().await?;
                synixe_events::respond!(msg, Response::PayMission(Ok(())))
                    .await
                    .map_err(std::convert::Into::into)
            }
            Self::UpdateMissionList {} => {
                let Ok(response) = reqwest::get(MISSION_LIST).await else {
                    return synixe_events::respond!(
                        msg,
                        Response::UpdateMissionList(
                            Err("Failed to fetch mission list".to_string())
                        )
                    )
                    .await
                    .map_err(std::convert::Into::into);
                };
                sqlx::query!("UPDATE missions SET archived = true")
                    .execute(&*db)
                    .await?;
                sqlx::query!("UPDATE missions_maps SET archived = true")
                    .execute(&*db)
                    .await?;
                match response.json::<Listing>().await {
                    Ok(listing) => {
                        for mission in listing.missions() {
                            let query = sqlx::query!(
                                r#"INSERT INTO missions (id, name, summary, description, type, archived)
                                    VALUES ($1, $2, $3, $4, $5, false)
                                    ON CONFLICT (id) DO UPDATE SET
                                        name = $2,
                                        summary = $3,
                                        description = $4,
                                        type = $5,
                                        archived = false"#,
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
                        for map in listing.maps() {
                            let query = sqlx::query!(
                                r#"INSERT INTO missions_maps (map, archived)
                                    VALUES ($1, false)
                                    ON CONFLICT (map) DO UPDATE SET
                                        archived = false"#,
                                map
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
            }
            Self::FetchMissionList { search } => {
                let search = search.as_deref().unwrap_or("");
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::missions::Mission,
                    Response::FetchMissionList,
                    "SELECT id, name, summary, description, type as \"typ: MissionType\" FROM missions WHERE archived = FALSE AND (LOWER(missions.name) LIKE LOWER($1) OR LOWER(missions.id) LIKE LOWER($1)) ORDER BY name ASC",
                    format!("%{search}%"),
                )?;
                Ok(())
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
                )?;
                Ok(())
            }
            Self::FetchMissionRsvps { scheduled } => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::missions::MissionRsvp,
                    Response::FetchMissionRsvps,
                    "SELECT scheduled, member, state as \"state: Rsvp\", details FROM missions_schedule_rsvp WHERE scheduled = $1",
                    scheduled,
                )?;
                Ok(())
            }
            Self::AddMissionRsvp {
                scheduled,
                member,
                rsvp,
                details,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::AddMissionRsvp,
                    "INSERT INTO missions_schedule_rsvp (scheduled, member, state, details) VALUES ($1, $2, $3, $4) ON CONFLICT (scheduled, member) DO UPDATE SET state = $3, details = $4",
                    scheduled,
                    member,
                    *rsvp as Rsvp,
                    details.as_ref(),
                )
            }
            Self::FetchCurrentMission {} => {
                fetch_one_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::missions::ScheduledMission,
                    Response::FetchCurrentMission,
                    "SELECT
                        s.id,
                        s.mission,
                        s.schedule_message_id,
                        s.start,
                        m.name,
                        m.summary,
                        m.description,
                        m.type as \"typ: MissionType\"
                    FROM
                        missions_schedule s
                    INNER JOIN
                        missions m ON m.id = s.mission
                    WHERE
                        s.start <= NOW() AND s.start + INTERVAL '150 minutes' >= NOW()
                    ORDER BY
                        s.start ASC
                    LIMIT 1",
                )?;
                Ok(())
            }
        }
    }
}
