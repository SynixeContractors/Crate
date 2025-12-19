use crate::CRATE_SERVER;

include!("../../../../lib/common/listener.rs");

mod discord;
mod missions;

pub async fn start() {
    let nats = bootstrap::NC::get().await;

    let Ok(sub) = nats
        .queue_subscribe(
            "synixe.publish.>",
            &format!("arma-server-{}", *CRATE_SERVER),
        )
        .await
    else {
        panic!("failed to subscribe to publish events");
    };
    while let Some(msg) = sub.next().await {
        synixe_events::listener!(
            msg.clone(),
            nats.clone(),
            synixe_events::discord::publish::Publish,
            synixe_events::missions::publish::Publish,
        );
    }
}
