use crate::CRATE_SERVER;

include!("../../../../lib/common/handler.rs");

mod garage;

#[allow(clippy::cognitive_complexity)]
pub async fn start() {
    let nats = bootstrap::NC::get().await;

    let Ok(sub) = nats
        .queue_subscribe("synixe.arma.>", &format!("arma-server-{}", *CRATE_SERVER))
        .await
    else {
        panic!("failed to subscribe to arma events");
    };
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        synixe_events::handler!(msg, nats, synixe_events::garage::arma::Request,);
    }
}
