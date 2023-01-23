use crate::DOCKER_SERVER;

include!("../../../../lib/common/handler.rs");

mod container;

#[allow(clippy::cognitive_complexity)]
pub async fn start() {
    // Init NATS connection
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
        synixe_events::handler!(msg, nats, synixe_events::containers::docker::Request,);
    }
}
