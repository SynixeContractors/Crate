mod campaigns;
mod certifications;
mod discord;
mod garage;
mod gear;
mod missions;
mod recruiting;
mod reputation;
mod servers;

include!("../../../../lib/common/handler.rs");

#[allow(clippy::cognitive_complexity)]
pub async fn start() {
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
            synixe_events::campaigns::db::Request,
            synixe_events::certifications::db::Request,
            synixe_events::discord::db::Request,
            synixe_events::garage::db::Request,
            synixe_events::gear::db::Request,
            synixe_events::missions::db::Request,
            synixe_events::recruiting::db::Request,
            synixe_events::reputation::db::Request,
            synixe_events::servers::db::Request,
        );
    }
}
