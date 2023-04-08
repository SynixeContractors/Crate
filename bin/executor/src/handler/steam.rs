use async_trait::async_trait;
use synixe_events::{
    mods::{
        db,
        executions::{Request, Response},
    },
    respond,
};
use synixe_model::mods::Workshop;
use synixe_proc::events_request_5;
use time::OffsetDateTime;
use time_tz::{timezones::db::america::NEW_YORK, OffsetDateTimeExt};

const STEAM_FORUM: &str = "https://steamcommunity.com/app/107410/discussions/21/";

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
            Self::CheckSteamModUpdates {} => {
                #[allow(clippy::cognitive_complexity)]
                match events_request_5!(
                    bootstrap::NC::get().await,
                    synixe_events::mods::db,
                    GetAllMods {}
                )
                .await
                {
                    Ok(Ok((db::Response::GetAllMods(Ok(mods)), _))) => {
                        if mods.is_empty() {
                            return respond!(msg, Response::CheckSteamModUpdates(Ok(())))
                                .await
                                .map_err(std::convert::Into::into);
                        }
                        for m in mods {
                            m.check_mod().await;
                        }
                        respond!(msg, Response::CheckSteamModUpdates(Ok(())))
                            .await
                            .map_err(std::convert::Into::into)
                    }
                    Ok(_) => {
                        error!("unexpected response from db");
                        respond!(
                            msg,
                            Response::CheckSteamModUpdates(Err(String::from(
                                "unexpected response from db"
                            )))
                        )
                        .await
                        .map_err(std::convert::Into::into)
                    }
                    Err(e) => {
                        error!("error getting upcoming missions: {}", e);
                        respond!(msg, Response::CheckSteamModUpdates(Err(e.to_string())))
                            .await
                            .map_err(std::convert::Into::into)
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use time::OffsetDateTime;

    use super::*;

    #[tokio::test]
    async fn test_check_mod() {
        let m = Workshop {
            name: "test".to_string(),
            workshop_id: "https://steamcommunity.com/sharedfiles/filedetails/changelog/463939057"
                .to_string(),
            updated_at: OffsetDateTime::now_utc(),
        };
        m.check_mod().await;
    }
}
