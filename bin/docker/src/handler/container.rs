use std::fmt::Display;

use async_trait::async_trait;
use bollard::Docker;
use synixe_events::{
    containers::docker::{Request, Response},
    discord::write::{DiscordContent, DiscordMessage},
    respond,
};
use synixe_meta::docker::ArmaServer;
use synixe_proc::events_request_5;

use crate::{CRATE_CONTAINER, CRATE_SERVER};

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
        match self {
            Self::Restart { server, reason } => {
                respond!(msg, Response::Restart(Ok(()))).await?;
                handle(nats, server, Action::Restart, reason).await
            }
            Self::Start { server, reason } => {
                respond!(msg, Response::Start(Ok(()))).await?;
                handle(nats, server, Action::Start, reason).await
            }
            Self::Stop { server, reason } => {
                respond!(msg, Response::Stop(Ok(()))).await?;
                handle(nats, server, Action::Stop, reason).await
            }
        }
    }
}

#[allow(clippy::cognitive_complexity)]
async fn handle(
    nats: std::sync::Arc<nats::asynk::Connection>,
    server: &ArmaServer,
    action: Action,
    reason: &str,
) -> Result<(), anyhow::Error> {
    if server != &CRATE_SERVER.1 {
        return Ok(());
    }
    let docker =
        Docker::connect_with_socket_defaults().expect("should be able to connect to docker");
    info!("{action} container {:?} ({reason})", &CRATE_CONTAINER);
    let res = match action {
        Action::Restart => {
            docker
                .restart_container(
                    &CRATE_CONTAINER,
                    None::<bollard::query_parameters::RestartContainerOptions>,
                )
                .await
        }
        Action::Start => {
            docker
                .start_container(
                    &CRATE_CONTAINER,
                    None::<bollard::query_parameters::StartContainerOptions>,
                )
                .await
        }
        Action::Stop => {
            docker
                .stop_container(
                    &CRATE_CONTAINER,
                    None::<bollard::query_parameters::StopContainerOptions>,
                )
                .await
        }
    };
    let audit = match res {
        Ok(()) => {
            format!("container {action}: {:?} ({reason})", &CRATE_CONTAINER)
        }
        Err(e) => {
            error!("failed to {action} container {:?}: {e}", &CRATE_CONTAINER);
            format!("failed to {action} container {:?}: {e}", &CRATE_CONTAINER)
        }
    };
    if let Err(e) = events_request_5!(
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
