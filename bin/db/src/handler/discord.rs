use async_trait::async_trait;
use synixe_events::{
    discord::db::{Request, Response},
    respond,
};

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
                                AND created > NOW() - '3 weeks'::Interval
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
