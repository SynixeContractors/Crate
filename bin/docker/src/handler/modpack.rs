use async_trait::async_trait;
use synixe_events::{
    containers::modpack::{Request, Response},
    respond,
};
use synixe_meta::docker::Primary;
use synixe_proc::events_request_5;
use tokio::process::Command;

use crate::DOCKER_SERVER;

use super::Handler;

#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        if *DOCKER_SERVER != "monterey-primary" {
            return Ok(());
        }
        respond!(msg, Response::Updated(Ok(()))).await?;
        let command = Command::new("rsync")
            .arg("-ur")
            .arg("--delete-after")
            .arg("moddownload@192.168.1.111:/home/download/mods")
            .arg(".")
            .current_dir("/arma/contracts/mods")
            .status()
            .await?;
        if !command.success() {
            error!("Failed to update mission list");
            return Err(anyhow::anyhow!("Failed to update mission list"));
        }
        if let Err(e) = events_request_5!(
            nats,
            synixe_events::containers::docker,
            Restart {
                container: Primary::Arma3Main.into(),
                reason: "modpack updated".to_string(),
            }
        )
        .await
        {
            error!("failed to send restart event for main: {e}");
        }
        tokio::time::sleep(std::time::Duration::from_secs(60)).await;
        if let Err(e) = events_request_5!(
            nats,
            synixe_events::containers::docker,
            Restart {
                container: Primary::Arma3Training.into(),
                reason: "modpack updated".to_string(),
            }
        )
        .await
        {
            error!("failed to send restart event for main: {e}");
        }
        Ok(())
    }
}
