include!("../../../../lib/common/listener.rs");

mod missions;

#[allow(clippy::cognitive_complexity)]
pub async fn start() {
    let nats = bootstrap::NC::get().await;

    let mut sub = nats
        .queue_subscribe("synixe.publish.>", String::from("synixe-docker"))
        .await
        .expect("Failed to subscribe to synixe.publish.*");
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        tokio::spawn(async move {
            synixe_events::listener!(
                msg.clone(),
                nats.clone(),
                synixe_events::missions::publish::Publish,
            );
        });
    }
}
