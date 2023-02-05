use arma_rs::IntoArma;
use async_trait::async_trait;
use synixe_events::garage::arma::Request;
use uuid::Uuid;

use crate::{CONTEXT, SERVER_ID};

use super::Handler;

#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        if *SERVER_ID != "arma-main" && *SERVER_ID != "test_brett_yehuda" {
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
                let pending = ctx.state().get::<crate::commands::garage::PendingSpawn>();
                let id = Uuid::new_v4();
                pending.as_ref().write().await.insert(id, msg);
                ctx.callback_data(
                    "crate:garage",
                    "spawn",
                    vec![
                        id.to_arma(),
                        plate.to_arma(),
                        class.to_arma(),
                        state.to_arma(),
                    ],
                );
                Ok(())
            }
        }
    }
}
