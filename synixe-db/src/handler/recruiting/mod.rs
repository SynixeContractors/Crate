use async_trait::async_trait;
use synixe_events::recruiting::db::{Request, Response};

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
            Self::Seen { url } => {
                execute_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::Seen,
                    "INSERT INTO recruitment_seen (link) VALUES ($1) ON CONFLICT (link) DO UPDATE SET created_at = NOW()",
                    url,
                )
            }
            Self::HasSeen { url } => {
                fetch_one_and_respond!(
                    msg,
                    *db,
                    cx,
                    Response::HasSeen,
                    "SELECT EXISTS(SELECT 1 FROM recruitment_seen WHERE link = $1) as value",
                    url,
                )
            }
        }
    }
}
