mod campaigns;
mod certifications;
mod discord;
mod garage;
mod gear;
mod github;
mod missions;
mod recruiting;
mod reputation;
mod reset;
mod servers;
mod surveys;
mod voting;

include!("../../../../lib/common/handler.rs");

#[allow(clippy::cognitive_complexity)]
pub async fn start() {
    let nats = bootstrap::NC::get().await;

    let sub = nats
        .queue_subscribe("synixe.db.>", "synixe-db")
        .await
        .expect("should be able to subscribe to db events");
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
            synixe_events::github::db::Request,
            synixe_events::missions::db::Request,
            synixe_events::recruiting::db::Request,
            synixe_events::reputation::db::Request,
            // synixe_events::reset::db::Request,
            synixe_events::servers::db::Request,
            synixe_events::surveys::db::Request,
            synixe_events::voting::db::Request,
        );
    }
}
