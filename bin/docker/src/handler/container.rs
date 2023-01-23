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
            Action::Restart => write!(f, "restart"),
            Action::Start => write!(f, "start"),
            Action::Stop => write!(f, "stop"),
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
            Request::Restart { container, reason } => {
                let ret = handle(nats, container, Action::Restart, reason).await;
                if let Err(e) = ret {
                    respond!(msg, Response::Restart(Err(e.clone()))).await?;
                    return Err(anyhow::anyhow!(e));
                }
                respond!(msg, Response::Restart(Ok(()))).await?;
                Ok(())
            }
            Request::Start { container, reason } => {
                let ret = handle(nats, container, Action::Start, reason).await;
                if let Err(e) = ret {
                    respond!(msg, Response::Start(Err(e.clone()))).await?;
                    return Err(anyhow::anyhow!(e));
                }
                respond!(msg, Response::Start(Ok(()))).await?;
                Ok(())
            }
            Request::Stop { container, reason } => {
                let ret = handle(nats, container, Action::Stop, reason).await;
                if let Err(e) = ret {
                    respond!(msg, Response::Stop(Err(e.clone()))).await?;
                    return Err(anyhow::anyhow!(e));
                }
                respond!(msg, Response::Stop(Ok(()))).await?;
                Ok(())
            }
        }
    }
}

async fn handle(
    nats: std::sync::Arc<nats::asynk::Connection>,
    container: &Container,
    action: Action,
    reason: &str,
) -> Result<Option<String>, String> {
    let docker = Docker::connect_with_socket_defaults().unwrap();
    if container.dc() != *DOCKER_SERVER {
        debug!(
            "ignoring container {} on {}",
            container.id(),
            container.dc()
        );
        return Ok(None);
    }
    info!("{} container {} ({})", action, container.id(), reason);
    let res = match action {
        Action::Restart => docker.restart_container(container.id(), None).await,
        Action::Start => docker.start_container::<String>(container.id(), None).await,
        Action::Stop => docker.stop_container(container.id(), None).await,
    };
    let audit = match res {
        Ok(_) => {
            format!("container {}: {} ({})", action, container.id(), reason)
        }
        Err(e) => {
            format!("failed to {} container {}: {}", action, container.id(), e)
        }
    };
    if let Err(e) = events_request!(
        nats,
        synixe_events::discord::write,
        Audit {
            message: DiscordMessage {
                content: DiscordContent::Text(audit.clone()),
                reactions: vec![],
            }
        }
    )
    .await
    {
        error!("failed to send audit message: {}", e);
    }
    Ok(Some(audit))
}
