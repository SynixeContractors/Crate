use std::{fs::File, io::Write, time::Duration};

use async_trait::async_trait;
use regex::Regex;
use synixe_events::missions::publish::Publish;
use synixe_model::missions::MissionType;
use synixe_proc::events_request;

use super::Listener;

#[async_trait]
impl Listener for Publish {
    async fn listen(
        &self,
        _msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        if let Publish::ChangeMission { id, mission_type } = &self {
            let Ok(regex) = Regex::new(r"(?m)template = ([^;]+);") else {
                error!("failed to compile regex");
                return Ok(());
            };
            if [
                MissionType::Contract,
                MissionType::SubContract,
                MissionType::Special,
            ]
            .contains(mission_type)
            {
                return Ok(());
            }
            let current_config = std::fs::read_to_string("/configs/arma-main/main.cfg")?;
            let new_config = regex
                .replace_all(&current_config, format!("template = {id};"))
                .to_string();
            let mut file = File::create("/configs/arma-main/main.cfg")?;
            file.write_all(new_config.as_bytes())?;
            tokio::time::sleep(Duration::from_secs(60)).await;
            if let Err(e) = events_request!(
                nats,
                synixe_events::containers::docker,
                Restart {
                    id: "arma-main".to_string(),
                    reason: format!("mission change: {id}"),
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
