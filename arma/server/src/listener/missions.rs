use std::{fs::File, io::Write};

use async_trait::async_trait;
use regex::Regex;
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
                match minutes {
                    4..=6 | 9..=11 | 14..=16 | 29..=31 | 59..=61 | 89..=91 | 119..=121 => {
                        context.callback_data(
                            "crate",
                            "global_message",
                            vec![arma_rs::Value::String(format!(
                                "[Mission] {} starts in {minutes} minutes!",
                                scheduled.name
                            ))],
                        );
                    }
                    -1..=1 => {
                        context.callback_data(
                            "crate",
                            "global_message",
                            vec![arma_rs::Value::String(format!(
                                "[Mission] {} starting now!",
                                scheduled.name
                            ))],
                        );
                    }
                    _ => {}
                }
                Ok(())
            }
            Self::ChangeMission { id, mission_type } => {
                let Ok(regex) = Regex::new(r"(?m)template = ([^;]+);") else {
                    error!("failed to compile regex");
                    return Ok(());
                };
                match mission_type {
                    MissionType::Contract | MissionType::SubContract | MissionType::Special => {
                        if *SERVER_ID != "primary-contracts" {
                            return Ok(());
                        }
                    }
                    _ => return Ok(()),
                }
                context.callback_data(
                    "crate",
                    "global_message",
                    vec![arma_rs::Value::String(format!(
                        "[Mission] You will be disconnected. Server is changing mission: {id}"
                    ))],
                );
                context.callback_data(
                    "crate",
                    "global_message",
                    vec![arma_rs::Value::String(
                        "[Mission] You have 30 seconds to save any gear and leave the shop."
                            .to_string(),
                    )],
                );
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                context.callback_null("crate", "restart");
                tokio::time::sleep(std::time::Duration::from_secs(30)).await;
                let current_config = std::fs::read_to_string("./configs/main.cfg")?;
                let new_config = regex
                    .replace_all(&current_config, format!("template = {id};"))
                    .to_string();
                let mut file = File::create("./configs/main.cfg")?;
                file.write_all(new_config.as_bytes())?;
                context.callback_null("crate:restart", "restart");
                Ok(())
            }
            Self::WarnChangeMission {
                id,
                mission_type: _,
            } => {
                context.callback_data(
                    "crate",
                    "global_message",
                    vec![arma_rs::Value::String(format!(
                        "[Mission] Restart in 10 minutes! Mission Starting: {id}"
                    ))],
                );
                Ok(())
            }
        }
    }
}
