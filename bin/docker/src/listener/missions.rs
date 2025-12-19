use std::{fs::File, io::Write, time::Duration};

use async_trait::async_trait;
use regex::Regex;
use synixe_events::{
    discord::write::{DiscordContent, DiscordMessage},
    missions::publish::Publish,
};
use synixe_meta::docker::Primary;
use synixe_model::missions::MissionType;
use synixe_proc::events_request_5;

use crate::DOCKER_SERVER;

use super::Listener;

const CONFIG_CONTRACTS: &str = "/arma/config/configs/contracts.cfg";

#[async_trait]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        if let Self::ChangeMission {
            id,
            mission_type,
            reason,
        } = &self
        {
            if *DOCKER_SERVER != "monterey-primary" {
                return Ok(());
            }
            if id.starts_with('$') {
                return Ok(());
            }
            info!("Changing contracts server mission to `{}`", id);
            let Ok(regex) = Regex::new(r"(?m)template = ([^;]+);") else {
                error!("failed to compile regex");
                return Ok(());
            };
            if ![
                MissionType::Contract,
                MissionType::SubContract,
                MissionType::Special,
            ]
            .contains(mission_type)
            {
                return Ok(());
            }
            let current_config = std::fs::read_to_string(CONFIG_CONTRACTS)?;
            let new_config = regex
                .replace_all(&current_config, format!("template = {id};"))
                .to_string();
            let mut file = File::create(CONFIG_CONTRACTS)?;
            file.write_all(new_config.as_bytes())?;
            if let Err(e) = events_request_5!(
                nats,
                synixe_events::discord::write,
                Audit {
                    message: DiscordMessage {
                        content: DiscordContent::Text(format!("Contracts server mission changed to `{id}`, will restart in 60 seconds. ({reason})")),
                        reactions: vec![],
                    }
                }
            )
            .await
            {
                error!("failed to send audit message: {}", e);
            }
            tokio::time::sleep(Duration::from_secs(60)).await;
            if let Err(e) = events_request_5!(
                nats,
                synixe_events::containers::docker,
                Restart {
                    container: Primary::Arma3Contracts.into(),
                    reason: format!("Mission changed to `{id}`"),
                }
            )
            .await
            {
                error!("failed to send restart event: {e}");
            }
        }
        Ok(())
    }
}
