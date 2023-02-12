use async_trait::async_trait;
use synixe_events::missions::publish::Publish;
use synixe_model::missions::MissionType;

use crate::{CONTEXT, SERVER_ID};

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
            Self::StartingSoon { scheduled, minutes } => {
                if *SERVER_ID == "arma-main" {
                    #[allow(clippy::cast_precision_loss)]
                    context.callback_data(
                        "crate:missions",
                        "set_date",
                        vec![arma_rs::Value::Number(*minutes as f64)],
                    );
                }
                match minutes {
                    4..=6 | 9..=11 | 14..=16 | 29..=31 | 59..=61 | 89..=91 | 119..=121 => {
                        context.callback_data(
                            "crate:global",
                            "brodsky_say",
                            vec![arma_rs::Value::String(format!(
                                "[Mission] {} starts in {minutes} minutes!",
                                scheduled.name
                            ))],
                        );
                    }
                    -1..=1 => {
                        context.callback_data(
                            "crate:global",
                            "brodsky_say",
                            vec![arma_rs::Value::String(format!(
                                "[Mission] {} starting now!",
                                scheduled.name
                            ))],
                        );
                        if *SERVER_ID == "arma-main" {
                            #[allow(clippy::cast_precision_loss)]
                            context.callback_null("crate:missions", "intro_text");
                        }
                    }
                    _ => {}
                }
                Ok(())
            }
            Self::ChangeMission {
                id,
                mission_type,
                reason: _,
            } => {
                match mission_type {
                    MissionType::Contract | MissionType::SubContract | MissionType::Special => {
                        if *SERVER_ID != "arma-main" {
                            info!("Ignoring mission change event for non-main server");
                            return Ok(());
                        }
                    }
                    _ => return Ok(()),
                }
                info!("Changing main server mission to `{id}`");
                context.callback_data(
                    "crate:global",
                    "brodsky_say",
                    vec![arma_rs::Value::String(format!(
                        "[Mission] You will be disconnected. Server is changing mission: {id}"
                    ))],
                );
                context.callback_data(
                    "crate:global",
                    "brodsky_say",
                    vec![arma_rs::Value::String(
                        "[Mission] You have 30 seconds to save any gear and leave the shop."
                            .to_string(),
                    )],
                );
                Ok(())
            }
            Self::WarnChangeMission {
                id,
                mission_type: _,
            } => {
                context.callback_data(
                    "crate:global",
                    "brodsky_say",
                    vec![arma_rs::Value::String(format!(
                        "[Mission] Restart in 10 minutes! Mission Starting: {id}"
                    ))],
                );
                Ok(())
            }
        }
    }
}
