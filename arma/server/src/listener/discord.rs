use async_trait::async_trait;
use synixe_events::discord::publish::Publish;

use crate::{CONTEXT, STEAM_CACHE};

use super::Listener;

#[async_trait]
#[deny(clippy::unwrap_used)]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let context_store = CONTEXT.read().await;
        let Some(context) = context_store.as_ref() else {
            error!("event received before context was initialized");
            return Ok(());
        };
        match &self {
            Self::MemberUpdate { member } => {
                if let Some(steam) = STEAM_CACHE.read().await.get(&member.user.id.to_string()) {
                    if let Err(e) = context.callback_data(
                        "crate:discord",
                        "member:get:ok",
                        vec![
                            arma_rs::Value::String(steam.to_string()),
                            arma_rs::Value::String(member.user.id.to_string()),
                            arma_rs::Value::Array(
                                member
                                    .roles
                                    .iter()
                                    .map(std::string::ToString::to_string)
                                    .map(arma_rs::Value::String)
                                    .collect(),
                            ),
                        ],
                    ) {
                        error!("error sending member:get:ok: {:?}", e);
                    }
                }
                Ok(())
            }
        }
    }
}
