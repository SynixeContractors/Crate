use async_trait::async_trait;
use synixe_events::{
    containers::missions::{Request, Response},
    respond,
};
use tokio::process::Command;

use crate::DOCKER_SERVER;

use super::Handler;

#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        _nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        // All arma servers are on worker-primary
        if *DOCKER_SERVER != "monterey-primary" {
            return Ok(());
        }
        respond!(msg, Response::UpdateMissionList(Ok(()))).await?;
        let command = Command::new("git")
            .arg("pull")
            .current_dir("/arma/contracts/mpmissions")
            .status()
            .await?;
        if !command.success() {
            error!("Failed to update mission list");
            return Err(anyhow::anyhow!("Failed to update mission list"));
        }
        Ok(())
    }
}
