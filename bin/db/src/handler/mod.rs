mod certifications;
mod discord;
mod gear;
mod missions;
mod recruiting;

include!("../../../../lib/common/handler.rs");

pub async fn start() {
    // Init NATS connection
    let nats = bootstrap::NC::get().await;

    let sub = nats
        .queue_subscribe("synixe.db.>", "synixe-db")
        .await
        .unwrap();
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        synixe_events::handler!(
            msg,
            nats,
            synixe_events::certifications::db::Request,
            synixe_events::discord::db::Request,
            synixe_events::gear::db::Request,
            synixe_events::missions::db::Request,
            synixe_events::recruiting::db::Request
        );
    }
}
