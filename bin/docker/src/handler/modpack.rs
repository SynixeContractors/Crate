use async_trait::async_trait;
use synixe_events::{
    containers::modpack::{Request, Response},
    respond,
};
use synixe_meta::docker::ArmaServer;
use synixe_proc::events_request_5;
use tokio::process::Command;

use crate::CRATE_CONTAINER;

use super::Handler;

#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        respond!(msg, Response::Updated(Ok(()))).await?;
        let command = Command::new("rsync")
            .arg("-ur")
            .arg("--delete-after")
            .arg("download@192.168.1.111:/")
            .arg(".")
            .current_dir(format!("/arma/{}/mods", *CRATE_CONTAINER))
            .status()
            .await?;
        if !command.success() {
            error!("Failed to update mods");
            return Err(anyhow::anyhow!("Failed to update mods"));
        }
        if let Err(e) = events_request_5!(
            nats,
            synixe_events::containers::docker,
            Restart {
                server: ArmaServer::Arma3Contracts,
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
                server: ArmaServer::Arma3Training,
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
