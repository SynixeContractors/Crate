mod missions;
mod recruiting;

include!("../../../../lib/common/handler.rs");

pub async fn start() {
    // Init NATS connection
    let nats = bootstrap::NC::get().await;

    let sub = nats
        .queue_subscribe("synixe.executor.>", "synixe-executor")
        .await
        .unwrap();
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        synixe_events::handler!(
            msg,
            nats,
            synixe_events::recruiting::executions::Request,
            synixe_events::missions::executions::Request
        );
    }
}
