#![deny(clippy::pedantic)]
#![warn(clippy::nursery, clippy::all)]

use synixe_events::handler;

#[macro_use]
extern crate log;

#[macro_use]
mod macros;

mod handler;

#[tokio::main]
async fn main() {
    bootstrap::logger::init();
    bootstrap::tracer!("db");

    let nats = bootstrap::NC::get().await;

    let sub = nats.subscribe("synixe.db.>").await.unwrap();
    while let Some(msg) = sub.next().await {
        let nats = nats.clone();
        handler!(
            msg,
            nats,
            synixe_events::recruiting::db::Request,
            synixe_events::missions::db::Request
        );
    }
}
