use async_trait::async_trait;
use synixe_events::mods::db::{Request, Response};

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
            Self::GetAllMods {} => {
                fetch_as_and_respond!(
                    msg,
                    *db,
                    cx,
                    synixe_model::mods::Workshop,
                    Response::GetAllMods,
                    "SELECT * FROM mods_workshop",
                )?;
                Ok(())
            }
        }
    }
}
