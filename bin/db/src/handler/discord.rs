use async_trait::async_trait;
use serenity::model::prelude::UserId;
use synixe_events::{
    discord::db::{MemberActivity, Notification, Notifications, Request, Response},
    respond,
};

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: async_nats::message::Message,
        _nats: async_nats::Client,
    ) -> Result<(), anyhow::Error> {
        let db = bootstrap::DB::get().await;
        match &self {
            Self::MemberActivity { member } => {
                match_with_return!(member_activity(member, &db), MemberActivity, msg, cx)
            }
            Self::RecordReachout { member, sent_by } => match_no_return!(
                async {
                    sqlx::query!(
                        "INSERT INTO members_reachouts (member, sent_by) VALUES ($1, $2)",
                        member.to_string(),
                        sent_by.to_string(),
                    )
                    .execute(&*db)
                    .await?;
                    Result::<(), anyhow::Error>::Ok(())
                },
                RecordReachout,
                msg,
                cx
            ),
            Self::GetNotifications { member } => {
                match_with_return!(get_notifications(member, &db), GetNotifications, msg, cx)
            }
            Self::SetNotifications {
                member,
                payday,
                mission_reminder,
            } => match_with_return!(
                set_notifications(member, *payday, *mission_reminder, &db),
                SetNotifications,
                msg,
                cx
            ),
            Self::NotificationSubscribers {
                members,
                notification,
            } => match_with_return!(
                notification_subscribers(members, *notification, &db),
                NotificationSubscribers,
                msg,
                cx
            ),
            Self::FromSteam { steam } => {
                fetch_one_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::FromSteam,
                    r#"
                        SELECT
                            member AS value
                        FROM
                            members_steam
                        WHERE
                            steam = $1"#,
                    steam,
                )
            }
            Self::SaveSteam { steam, member } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::SaveSteam,
                    r#"
                        INSERT INTO
                            members_steam (steam, member)
                        VALUES
                            ($1, $2)
                        ON CONFLICT (member)
                        DO UPDATE SET
                            steam = $1"#,
                    steam,
                    member.to_string(),
                )
            }
            Self::SaveDLC { member, dlc } => {
                #[allow(clippy::cast_possible_wrap)]
                let dlc = dlc.iter().map(|x| *x as i32).collect::<Vec<_>>();
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::SaveDLC,
                    r#"
                        INSERT INTO
                            members_dlc (member, dlc)
                        VALUES
                            ($1, $2)
                        ON CONFLICT (member)
                        DO UPDATE
                            SET dlc = $2"#,
                    member.to_string(),
                    &dlc,
                )
            }
            Self::ActiveMembers {} => respond!(
                msg,
                Response::ActiveMembers(
                    match sqlx::query!(
                        r#"
                            SELECT DISTINCT ON (member)
                                member as value
                            FROM gear_bank_deposits
                            WHERE
                                id != '00000000-0000-0000-0000-000000000000'
                                AND created > NOW() - '30 days'::Interval
                            ORDER BY member, created DESC"#,
                    )
                    .fetch_all(&*db)
                    .await
                    .map(|x| x.into_iter().map(|x| x.value).collect::<Vec<_>>())
                    {
                        Ok(x) => Ok(x),
                        Err(e) => {
                            error!("Failed to get active members: {}", e);
                            Err(e.to_string())
                        }
                    }
                )
            )
            .await
            .map_err(|e| anyhow::anyhow!(e)),
        }
    }
}

async fn get_notifications(
    member: &UserId,
    db: &sqlx::PgPool,
) -> Result<Notifications, anyhow::Error> {
    let row = sqlx::query!(
        "SELECT payday, mission_reminder FROM members_notifications WHERE member = $1",
        member.to_string(),
    )
    .fetch_optional(db)
    .await?;
    Ok(
        row.map_or_else(Notifications::default, |row| Notifications {
            payday: row.payday,
            mission_reminder: row.mission_reminder,
        }),
    )
}

async fn set_notifications(
    member: &UserId,
    payday: Option<bool>,
    mission_reminder: Option<bool>,
    db: &sqlx::PgPool,
) -> Result<Notifications, anyhow::Error> {
    let row = sqlx::query!(
        "INSERT INTO members_notifications (member, payday, mission_reminder)
        VALUES ($1, COALESCE($2, FALSE), COALESCE($3, FALSE))
        ON CONFLICT (member) DO UPDATE SET
            payday = COALESCE($2, members_notifications.payday),
            mission_reminder = COALESCE($3, members_notifications.mission_reminder)
        RETURNING payday, mission_reminder",
        member.to_string(),
        payday,
        mission_reminder,
    )
    .fetch_one(db)
    .await?;
    Ok(Notifications {
        payday: row.payday,
        mission_reminder: row.mission_reminder,
    })
}

async fn notification_subscribers(
    members: &[UserId],
    notification: Notification,
    db: &sqlx::PgPool,
) -> Result<Vec<UserId>, anyhow::Error> {
    let members = members.iter().map(ToString::to_string).collect::<Vec<_>>();
    let subscribed = match notification {
        Notification::Payday => {
            sqlx::query_scalar!(
                "SELECT member FROM members_notifications WHERE member = ANY($1) AND payday",
                &members,
            )
            .fetch_all(db)
            .await?
        }
        Notification::MissionReminder => sqlx::query_scalar!(
            "SELECT member FROM members_notifications WHERE member = ANY($1) AND mission_reminder",
            &members,
        )
        .fetch_all(db)
        .await?,
    };
    subscribed
        .iter()
        .map(|member| Ok(UserId::new(member.parse()?)))
        .collect()
}

async fn member_activity(
    member: &UserId,
    db: &sqlx::PgPool,
) -> Result<MemberActivity, anyhow::Error> {
    let row = sqlx::query!(
        r#"SELECT
            (SELECT COUNT(*) FROM gear_bank_deposits d INNER JOIN missions_schedule s ON s.id = d.id
                WHERE d.member = $1) AS "missions!",
            (SELECT MAX(s.start) FROM gear_bank_deposits d INNER JOIN missions_schedule s ON s.id = d.id
                WHERE d.member = $1) AS last_mission,
            (SELECT MAX(l.created) FROM servers_log l INNER JOIN members_steam m ON m.steam = l.steam
                WHERE m.member = $1 AND l.action = 'connected') AS last_connected,
            (SELECT MAX(created) FROM members_reachouts WHERE member = $1) AS last_reachout"#,
        member.to_string(),
    )
    .fetch_one(db)
    .await?;
    Ok(MemberActivity {
        missions: row.missions,
        last_mission: row.last_mission,
        last_connected: row.last_connected,
        last_reachout: row.last_reachout,
    })
}
