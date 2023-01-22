use async_trait::async_trait;
use bollard::Docker;
use synixe_events::{
    containers::docker::{Request, Response},
    discord::write::{DiscordContent, DiscordMessage},
    respond,
};
use synixe_proc::events_request;

use super::Handler;

#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        let docker = Docker::connect_with_socket_defaults().unwrap();
        let audit = match self {
            Request::Restart { id, reason } => {
                if docker.restart_container(id, None).await.is_ok() {
                    if let Err(e) = respond!(msg, Response::Restart(Ok(()))).await {
                        error!("failed to respond to restart request: {}", e);
                    }
                    format!("Restarted container {id}: {reason}")
                } else {
                    if let Err(e) = respond!(
                        msg,
                        Response::Restart(Err("Failed to restart container".to_string()))
                    )
                    .await
                    {
                        error!("failed to respond to restart request: {}", e);
                    }
                    format!("Failed to restart container {id}: {reason}")
                }
            }
            Request::Start { id, reason } => {
                if docker.start_container::<String>(id, None).await.is_ok() {
                    if let Err(e) = respond!(msg, Response::Start(Ok(()))).await {
                        error!("failed to respond to start request: {}", e);
                    }
                    format!("Started container {id}: {reason}")
                } else {
                    if let Err(e) = respond!(
                        msg,
                        Response::Start(Err("Failed to start container".to_string()))
                    )
                    .await
                    {
                        error!("failed to respond to start request: {}", e);
                    }
                    format!("Failed to start container {id}: {reason}")
                }
            }
            Request::Stop { id, reason } => {
                if docker.stop_container(id, None).await.is_ok() {
                    if let Err(e) = respond!(msg, Response::Stop(Ok(()))).await {
                        error!("failed to respond to start request: {}", e);
                    }
                    format!("Stopped container {id}: {reason}")
                } else {
                    if let Err(e) = respond!(
                        msg,
                        Response::Stop(Err("Failed to stop container".to_string()))
                    )
                    .await
                    {
                        error!("failed to respond to start request: {}", e);
                    }
                    format!("Failed to stop container {id}: {reason}")
                }
            }
        };
        if let Err(e) = events_request!(
            nats,
            synixe_events::discord::write,
            Audit {
                message: DiscordMessage {
                    content: DiscordContent::Text(audit),
                    reactions: vec![],
                }
            }
        )
        .await
        {
            error!("failed to send audit message: {}", e);
        }
        Ok(())
    }
}
