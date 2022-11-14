use async_trait::async_trait;
use synixe_events::discord::publish::Publish;

use crate::{CONTEXT, STEAM_CACHE};

use super::Listener;

#[async_trait]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
        _cx: opentelemetry::Context,
    ) -> Result<(), anyhow::Error> {
        match &self {
            Self::ReactionAdd { reaction: _ } => todo!(),
            Self::ReactionRemove { reaction: _ } => todo!(),
            Self::MemberUpdate { member } => {
                if let Some(steam) = STEAM_CACHE.read().await.get(&member.user.id.to_string()) {
                    CONTEXT.read().await.as_ref().unwrap().callback_data(
                        "crate:discord",
                        "member",
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
                    );
                }
                Ok(())
            }
        }
    }
}
