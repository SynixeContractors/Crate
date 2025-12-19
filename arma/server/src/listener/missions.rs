use async_trait::async_trait;
use synixe_events::missions::publish::Publish;
use synixe_model::missions::MissionType;

use crate::{CONTEXT, SERVER_ID};

use super::Listener;

#[async_trait]
#[allow(clippy::too_many_lines)]
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
                if *SERVER_ID == "primary-main" {
                    #[allow(clippy::cast_precision_loss)]
                    if let Err(e) = context.callback_data(
                        "crate:missions",
                        "set_date",
                        vec![arma_rs::Value::Number(*minutes as f64)],
                    ) {
                        error!("error sending set_date: {e:?}");
                    }
                }
                match minutes {
                    4..=6 | 9..=11 | 14..=16 | 29..=31 | 59..=61 | 89..=91 | 119..=121 => {
                        if let Err(e) = context.callback_data(
                            "crate:global",
                            "brodsky_say",
                            vec![arma_rs::Value::String(format!(
                                "[Mission] {} starts in {minutes} minutes!",
                                scheduled.name
                            ))],
                        ) {
                            error!("error sending brodsky_say: {e:?}");
                        }
                    }
                    -1..=1 => {
                        if let Err(e) = context.callback_data(
                            "crate:global",
                            "brodsky_say",
                            vec![arma_rs::Value::String(format!(
                                "[Mission] {} starting now!",
                                scheduled.name
                            ))],
                        ) {
                            error!("error sending brodsky_say: {e:?}");
                        }
                        if *SERVER_ID == "primary-main" {
                            #[allow(clippy::cast_precision_loss)]
                            if let Err(e) = context.callback_null("crate:missions", "intro_text") {
                                error!("error sending intro_text: {e:?}");
                            }
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
                        if *SERVER_ID != "primary-main" {
                            info!("Ignoring mission change event for non-main server");
                            return Ok(());
                        }
                    }
                    _ => return Ok(()),
                }
                info!("Changing main server mission to `{id}`");
                if let Err(e) = context.callback_data(
                    "crate:global",
                    "brodsky_say",
                    vec![arma_rs::Value::String(format!(
                        "[Mission] You will be disconnected. Server is changing mission: {id}"
                    ))],
                ) {
                    error!("error sending brodsky_say: {e:?}");
                }
                if let Err(e) = context.callback_data(
                    "crate:global",
                    "brodsky_say",
                    vec![arma_rs::Value::String(
                        "[Mission] You have 30 seconds to save any gear and leave the shop."
                            .to_string(),
                    )],
                ) {
                    error!("error sending brodsky_say: {e:?}");
                }
                Ok(())
            }
            Self::WarnChangeMission {
                id,
                mission_type: _,
            } => {
                if let Err(e) = context.callback_data(
                    "crate:global",
                    "brodsky_say",
                    vec![arma_rs::Value::String(format!(
                        "[Mission] Restart in 10 minutes! Mission Starting: {id}"
                    ))],
                ) {
                    error!("error sending brodsky_say: {e:?}");
                }
                Ok(())
            }
        }
    }
}
