use std::fmt::Display;

use async_trait::async_trait;
use bollard::Docker;
use synixe_events::{
    containers::docker::{Request, Response},
    discord::write::{DiscordContent, DiscordMessage},
    respond,
};
use synixe_meta::docker::Container;
use synixe_proc::events_request;

use crate::DOCKER_SERVER;

use super::Handler;

#[derive(Debug)]
pub enum Action {
    Restart,
    Start,
    Stop,
}

impl Display for Action {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Restart => write!(f, "restart"),
            Self::Start => write!(f, "start"),
            Self::Stop => write!(f, "stop"),
        }
    }
}

#[async_trait]
impl Handler for Request {
    async fn handle(
        &self,
        msg: nats::asynk::Message,
        nats: std::sync::Arc<nats::asynk::Connection>,
    ) -> Result<(), anyhow::Error> {
        // TODO could use a macro for these 3 to reduce some code, but not really worth it
        match self {
            Self::Restart { container, reason } => {
                respond!(msg, Response::Restart(Ok(()))).await?;
                handle(nats, container, Action::Restart, reason).await
            }
            Self::Start { container, reason } => {
                respond!(msg, Response::Start(Ok(()))).await?;
                handle(nats, container, Action::Start, reason).await
            }
            Self::Stop { container, reason } => {
                respond!(msg, Response::Stop(Ok(()))).await?;
                handle(nats, container, Action::Stop, reason).await
            }
        }
    }
}

async fn handle(
    nats: std::sync::Arc<nats::asynk::Connection>,
    container: &Container,
    action: Action,
    reason: &str,
) -> Result<(), anyhow::Error> {
    let docker = Docker::connect_with_socket_defaults().unwrap();
    if container.dc() != *DOCKER_SERVER {
        debug!(
            "ignoring container {} on {}",
            container.id(),
            container.dc()
        );
        return Ok(());
    }
    info!("{} container {} ({})", action, container.key(), reason);
    let res = match action {
        Action::Restart => docker.restart_container(container.id(), None).await,
        Action::Start => docker.start_container::<String>(container.id(), None).await,
        Action::Stop => docker.stop_container(container.id(), None).await,
    };
    let audit = match res {
        Ok(_) => {
            format!("container {}: {} ({})", action, container.key(), reason)
        }
        Err(e) => {
            error!("failed to {} container {}: {}", action, container.key(), e);
            format!("failed to {} container {}: {}", action, container.key(), e)
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
