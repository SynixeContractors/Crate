use async_trait::async_trait;
use synixe_events::servers::db::{Request, Response};

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
            Self::Log {
                server,
                steam,
                action,
                data,
            } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::Log,
                    "INSERT INTO servers_log (server, steam, action, data) VALUES ($1, $2, $3, $4)",
                    server,
                    steam,
                    action,
                    data,
                )
            }
        }
    }
}
