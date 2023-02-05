use crate::DOCKER_SERVER;

include!("../../../../lib/common/handler.rs");

mod container;
mod missions;
mod modpack;

#[allow(clippy::cognitive_complexity)]
pub async fn start() {
    let nats = bootstrap::NC::get().await;

    let sub = nats
        .queue_subscribe(
            "synixe.docker.>",
            &format!("synixe-docker-{}", *DOCKER_SERVER),
        )
        .await
        .unwrap();
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        synixe_events::handler!(
            msg,
            nats,
            synixe_events::containers::docker::Request,
            synixe_events::containers::missions::Request,
            synixe_events::containers::modpack::Request,
        );
    }
}
