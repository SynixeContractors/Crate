use arma_rs::{ContextState, IntoArma};
use async_trait::async_trait;
use synixe_events::garage::arma::Request;
use synixe_meta::docker::ArmaServer;
use uuid::Uuid;

use crate::{CONTEXT, CRATE_SERVER};

use super::Handler;

#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        if !ArmaServer::is_contracts(&CRATE_SERVER) {
            return Ok(());
        }
        match self {
            Self::Spawn {
                class,
                state,
                plate,
            } => {
                let ctxguard = CONTEXT.read().await;
                let ctx = ctxguard.as_ref().expect("Unable to get context");
                let pending = ctx
                    .global()
                    .get::<crate::commands::garage::PendingSpawn>()
                    .expect("Unable to get pending spawns");
                let id = Uuid::new_v4();
                pending.as_ref().write().await.insert(id, msg);
                if let Err(e) = ctx.callback_data(
                    "crate:garage",
                    "spawn",
                    vec![
                        id.to_arma(),
                        plate.to_arma(),
                        class.to_arma(),
                        state.to_arma(),
                    ],
                ) {
                    error!("error sending spawn: {e:?}");
                }
                Ok(())
            }
        }
    }
}
