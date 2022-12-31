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
        .expect("Failed to subscribe to synixe.publish.*");
    while let Some(msg) = sub.next().await {
        synixe_events::listener!(
            msg.clone(),
            nats.clone(),
            synixe_events::certifications::publish::Publish,
            synixe_events::missions::publish::Publish
        );
    }
}
