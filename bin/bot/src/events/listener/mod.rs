include!("../../../../../lib/common/listener.rs");

mod certifications;
mod missions;

#[allow(clippy::cognitive_complexity)]
pub async fn start() {
    // Init NATS connection
    let nats = bootstrap::NC::get().await;

    let sub = nats
        .queue_subscribe("synixe.publish.>", "synixe-bot")
        .await
        .unwrap();
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        synixe_events::listener!(
            msg,
            nats,
            synixe_events::certifications::publish::Publish,
            synixe_events::missions::publish::Publish
        );
    }
}
