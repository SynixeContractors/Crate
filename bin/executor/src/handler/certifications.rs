use async_trait::async_trait;
use synixe_events::{
    certifications::{
        db,
        executions::{Request, Response},
    },
    respond,
};
use synixe_proc::events_request;

use super::Handler;

#[allow(clippy::too_many_lines)]
#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
        _cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::CheckExpiries {  } => {
                respond!(msg, Response::CheckExpiries(Ok(()))).await?;
                let Ok(((db::Response::Expiring(Ok(expiring)), _), _)) = events_request!(
                    bootstrap::NC::get().await,
                    synixe_events::certifications::db,
                    Expiring { days: 30 }
                ).await else {
                    return Ok(());
                };
                for trail in expiring {
                    println!("Expiring: {:?}", trail);
                }
                Ok(())
            },
        }
    }
}
