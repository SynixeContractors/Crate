use async_trait::async_trait;
use synixe_events::discord::db::{Request, Response};

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
            Self::FromSteam { steam_id } => {
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
                            steam_id = $1"#,
                    steam_id,
                )
            }
            Self::SaveSteam { steam_id, member } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::SaveSteam,
                    r#"
                        INSERT INTO
                            members_steam (steam_id, member)
                        VALUES
                            ($1, $2)
                        ON CONFLICT (member)
                        DO UPDATE SET
                            steam_id = $1"#,
                    steam_id,
                    member.to_string(),
                )
            }
        }
    }
}
